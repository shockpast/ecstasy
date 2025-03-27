use std::{
    sync::{
        Arc, LazyLock,
        atomic::{AtomicI32, Ordering},
    },
    time::Duration,
};

use clap::Parser;
use osu_db::CollectionList;
use tokio::sync::{RwLock, Semaphore};
use tracing::info;
use utilities::collection::{create_collection, format_collection_name};

mod collector;
mod config;
mod mirrors;
mod utilities;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Run a SpeedTest against all osu! mirrors
    #[arg(short)]
    speedtest: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    if args.speedtest {
        let _ = tokio::task::spawn(utilities::speedtest::benchmark()).await;
        return;
    }

    static CONFIG: LazyLock<Arc<config::Config>> =
        std::sync::LazyLock::new(|| Arc::new(config::init()));
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

    // parallel download
    let downloaded = Arc::new(AtomicI32::new(0));
    let tasks_count = Arc::new(AtomicI32::new(0));
    let semaphore = Arc::new(Semaphore::new(CONFIG.user.concurrent_downloads));

    for beatmapset in remote_collection_info.beatmapsets {
        create_collection(
            Arc::clone(&collection_buffer),
            &Arc::clone(&local_collection_name),
            CONFIG.osu.collection_path.clone(),
        )
        .await;

        let collection_buffer = Arc::clone(&collection_buffer);
        let local_collection_name = Arc::clone(&local_collection_name);
        let mirror = Arc::clone(&mirror);
        let downloaded = Arc::clone(&downloaded);
        let remote_collection_beatmaps = Arc::clone(&remote_collection_beatmaps);
        let tasks_count = Arc::clone(&tasks_count);
        let semaphore = Arc::clone(&semaphore);

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            tasks_count.fetch_add(1, Ordering::SeqCst);

            if let Ok(contents) = mirror.get_file(beatmapset.id).await {
                let file_path = format!("{}/{}.osz", CONFIG.osu.songs_path, beatmapset.id);
                tokio::fs::write(file_path, contents).await.unwrap();

                let beatmapset_entity = &remote_collection_beatmaps
                    .beatmapsets
                    .iter()
                    .find(|s| s.id == beatmapset.id)
                    .unwrap();

                for beatmap in beatmapset.beatmaps {
                    collection_buffer
                        .write()
                        .await
                        .collections
                        .iter_mut()
                        .find(|c| {
                            c.name.as_ref().unwrap_or(&"".to_string()) == &*local_collection_name
                        })
                        .unwrap()
                        .beatmap_hashes
                        .push(Some(beatmap.checksum));

                    info!(
                        "({:?}/{}) {} - {}",
                        downloaded,
                        remote_collection_info.beatmap_count,
                        beatmapset_entity.artist,
                        beatmapset_entity.title
                    );
                }

                collection_buffer
                    .read()
                    .await
                    .to_file(CONFIG.osu.collection_path.clone())
                    .unwrap();

                downloaded.fetch_add(1, Ordering::SeqCst);
                tasks_count.fetch_sub(1, Ordering::SeqCst);

                drop(_permit);
            }
        });
    }

    std::thread::sleep(Duration::from_secs(999));
    // std::thread::sleep(dur);
}
