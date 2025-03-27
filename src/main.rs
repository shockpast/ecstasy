use std::{
    cell::RefCell,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicI32, Ordering},
    },
};

use clap::Parser;
use futures::{StreamExt, stream};
use osu_db::{CollectionList, Listing, collection};
use tokio::{
    sync::{Mutex, RwLock, Semaphore},
    task,
};
use tracing::{info, warn};
use tracing_subscriber;
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
        utilities::speedtest::benchmark().await;
        return;
    }

    static CONFIG: LazyLock<Arc<config::Config>> =
        std::sync::LazyLock::new(|| Arc::new(config::init()));
    let mirror = Arc::new(CONFIG.user.mirror_type.get_mirror());

    let remote_collection_info = collector::get_info(CONFIG.collector.id)
        .await
        .expect("osu!collector's Collection couldn't be found at this ID!");
    let remote_collection_beatmaps = Arc::new(
        collector::get_beatmaps(CONFIG.collector.id)
            .await
            .expect("osu!collector's Collection couldn't be found at this ID!"),
    );

    let collection_buffer = CollectionList::from_file(&CONFIG.osu.collection_path)
        .expect("Local Collection couldn't be parsed from a provided path to 'collection.db'");
    let listing_buffer = Listing::from_file(&CONFIG.osu.osu_path)
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
    let downloaded: Arc<AtomicI32> = Arc::new(AtomicI32::new(0));
    let tasks_count = Arc::new(AtomicI32::new(0));
    let semaphore = Arc::new(Semaphore::new(CONFIG.user.concurrent_downloads));

    for beatmap in remote_collection_info.beatmapsets {
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

        tokio::task::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            println!("Downloading {}", beatmap.id);
            tasks_count.fetch_add(1, Ordering::SeqCst);
            println!("Task {:?} spawned", tasks_count);
            if let Ok(contents) = mirror.get_file(beatmap.id).await {
                let file_path = format!("{}/{}.osz", CONFIG.osu.songs_path, beatmap.id);
                tokio::fs::write(file_path, contents).await.unwrap();

                for b in beatmap.beatmaps {
                    let mut collection = collection_buffer.write().await;
                    println!("Locked");

                    let hashes = collection
                        .collections
                        .iter_mut()
                        .find(|c| {
                            c.name.as_ref().unwrap_or(&"".to_string()) == &*local_collection_name
                        })
                        .unwrap();

                    hashes.beatmap_hashes.push(Some(b.checksum));
                    println!("Pushed");
                }

                collection_buffer
                    .read()
                    .await
                    .to_file(CONFIG.osu.collection_path.clone())
                    .unwrap();

                downloaded.fetch_add(1, Ordering::SeqCst);

                let beatmapset = &remote_collection_beatmaps
                    .beatmapsets
                    .iter()
                    .find(|s| s.id == beatmap.id)
                    .unwrap();

                info!(
                    "({:?}/{}) {} - {}",
                    downloaded,
                    remote_collection_info.beatmap_count,
                    beatmapset.artist,
                    beatmapset.title,
                );
                drop(_permit);
            }
        });
    }
    loop {}
    /*
        //
        stream::iter(remote_collection_beatmaps.beatmaps.iter())
            .for_each_concurrent(config.user.concurrent_downloads, |beatmap| async {
                let listing_buffer = listing_buffer_mutex.lock().unwrap();
                let mut collection_buffer = collection_buffer_mutex.lock().unwrap();

                let local_collection = get_or_create_collection(&mut collection_buffer, &local_collection_name, None);

                if listing_buffer.beatmaps.iter().find(|b| b.beatmapset_id == beatmap.beatmapset_id).is_some() {
                    warn!("'bs:{}' already downloaded, adding to collection and skipping download process..", beatmap.beatmapset_id);

                    local_collection.beatmap_hashes.push(Some(beatmap.checksum.clone()));
                    collection_buffer.to_file(config.osu.collection_path.clone()).unwrap();

                    return;
                }

                if let Ok(contents) = mirror.get_file(beatmap.beatmapset_id).await {
                    let file_path = format!("{}/{}.osz", config.osu.songs_path, beatmap.beatmapset_id);
                    tokio::fs::write(file_path, contents).await.unwrap();

                    local_collection.beatmap_hashes.push(Some(beatmap.checksum.clone()));
                    collection_buffer.to_file(config.osu.collection_path.clone()).unwrap();

                    downloaded.fetch_add(1, Ordering::SeqCst);

                    let beatmapset = remote_collection_beatmaps
                        .beatmapsets
                        .iter()
                        .find(|s| s.id == beatmap.beatmapset_id)
                        .unwrap();

                    info!("({:?}/{}) {} - {} [{}]", downloaded, remote_collection_info.beatmap_count, beatmapset.artist, beatmapset.title, beatmap.version)
                }
            })
            .await;
    */
}
