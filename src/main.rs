use std::{
    sync::{
        Arc, LazyLock,
        atomic::{AtomicI32, Ordering},
    },
    time::Duration,
};

use clap::Parser;
use mirrors::Ratelimiter;
use osu_db::CollectionList;
use sanitise_file_name::sanitise;
use tokio::sync::{RwLock, Semaphore};
use tracing::{error, info};

use utilities::{
    collection::{add_to_collection, create_collection, format_collection_name},
    osu::find_beatmap,
};

mod collector;
mod config;
mod mirrors;
mod utilities;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Run a SpeedTest against all osu! mirrors
    #[arg(short)]
    pub speedtest: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    if args.speedtest {
        let _ = tokio::task::spawn(utilities::speedtest::benchmark()).await;
        return;
    }

    static CONFIG: LazyLock<config::Config> = std::sync::LazyLock::new(config::init);
    let mirror = Arc::new(CONFIG.user.mirror_type.get_mirror());

    let remote_collection_info = collector::get_info(CONFIG.collector.id)
        .await
        .expect("osu!collector's Collection Info couldn't be found at this ID!");
    let remote_collection_beatmaps = Arc::new(
        collector::get_beatmaps(CONFIG.collector.id)
            .await
            .expect("osu!collector's Collection Beatmaps couldn't be found at this ID!"),
    );

    let collection_buffer = CollectionList::from_file(&CONFIG.osu.collection_path)
        .expect("Local Collection couldn't be parsed from a provided path to 'collection.db'");
    let collection_buffer = Arc::new(RwLock::new(collection_buffer));

    let local_collection_name = Arc::new(format_collection_name(
        &CONFIG.user.collection_name_format,
        &remote_collection_info,
    ));

    info!(
        "{} by {} (with {} beatmaps)",
        remote_collection_info.name.trim(),
        remote_collection_info.uploader.username,
        remote_collection_info.beatmap_count
    );

    let downloaded = Arc::new(AtomicI32::new(1));
    let beatmap_count = Arc::new(AtomicI32::new(remote_collection_info.beatmap_count as i32));

    let rate_limiter = Arc::new(Ratelimiter::default());
    let semaphore = Arc::new(Semaphore::new(CONFIG.user.concurrent_downloads));

    for beatmapset in remote_collection_info.beatmapsets {
        create_collection(
            Arc::clone(&collection_buffer),
            &local_collection_name,
            &CONFIG.osu.collection_path,
        )
        .await;

        if find_beatmap(&CONFIG.osu.songs_path, beatmapset.id)
            .await
            .is_some()
        {
            for beatmap in &beatmapset.beatmaps {
                add_to_collection(
                    &collection_buffer,
                    &local_collection_name,
                    &beatmap.checksum,
                )
                .await;

                downloaded
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| x.checked_add(1))
                    .expect("Overflow");
            }

            continue;
        }

        let collection_buffer = Arc::clone(&collection_buffer);
        let local_collection_name = Arc::clone(&local_collection_name);
        let mirror = Arc::clone(&mirror);
        let downloaded = Arc::clone(&downloaded);
        let remote_collection_beatmaps = Arc::clone(&remote_collection_beatmaps);
        let semaphore = Arc::clone(&semaphore);
        let rate_limiter = Arc::clone(&rate_limiter);
        let beatmap_count = Arc::clone(&beatmap_count);

        tokio::task::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let _rate_limiter: &Ratelimiter = &rate_limiter;

            match mirror.get_file(beatmapset.id, _rate_limiter).await {
                Ok(bytes) => {
                    let beatmapset_entity = &remote_collection_beatmaps
                        .beatmapsets
                        .iter()
                        .find(|s| s.id == beatmapset.id)
                        .unwrap();

                    let file_name = format!(
                        "{} {} - {}",
                        beatmapset.id, beatmapset_entity.artist, beatmapset_entity.title
                    );
                    let file_path = format!(
                        "{}/{}.osz",
                        CONFIG.osu.songs_path,
                        sanitise(file_name.as_str())
                    );

                    tokio::fs::write(file_path, bytes).await.unwrap();

                    for beatmap in beatmapset.beatmaps {
                        add_to_collection(
                            &collection_buffer,
                            &local_collection_name,
                            &beatmap.checksum,
                        )
                        .await;

                        let beatmap_entity = remote_collection_beatmaps
                            .beatmaps
                            .iter()
                            .find(|b| b.checksum == beatmap.checksum)
                            .unwrap();

                        info!(
                            "({}/{}) {} - {} [{}]",
                            downloaded.load(Ordering::SeqCst),
                            beatmap_count.load(Ordering::SeqCst),
                            beatmapset_entity.artist,
                            beatmapset_entity.title,
                            beatmap_entity.version
                        );
                    }

                    collection_buffer
                        .read()
                        .await
                        .to_file(&CONFIG.osu.collection_path)
                        .unwrap();

                    downloaded
                        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| x.checked_add(1))
                        .expect("Overflow");

                    drop(_permit);
                }
                Err(error) => {
                    error!("{} ({}): {}", mirror.get_name(), beatmapset.id, error);

                    beatmap_count
                        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| x.checked_sub(1))
                        .expect("Overflow");

                    drop(_permit);
                }
            };
        });
    }

    while downloaded.load(Ordering::SeqCst) < beatmap_count.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_secs(1));
    }

    info!(
        "{} - {} is downloaded/merged, have fun!",
        remote_collection_info.uploader.username, remote_collection_info.name
    );
}
