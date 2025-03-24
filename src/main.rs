use std::sync::{Arc, Mutex};

use clap::Parser;
use futures::{stream, StreamExt};
use tracing::{info, warn};
use tracing_subscriber;
use osu_db::{CollectionList, Listing};
use utilities::collection::{format_collection_name, get_or_create_collection};

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
    if args.speedtest { utilities::speedtest::benchmark().await; return; }

    let config = config::init();
    let mirror = config.user.mirror_type.get_mirror();

    let remote_collection_info = collector::get_info(config.collector.id)
        .await
        .expect("osu!collector's Collection couldn't be found at this ID!");
    let remote_collection_beatmaps = collector::get_beatmaps(config.collector.id)
        .await
        .expect("osu!collector's Collection couldn't be found at this ID!");

    let collection_buffer = CollectionList::from_file(config.osu.collection_path.clone())
        .expect("Local Collection couldn't be parsed from a provided path to 'collection.db'");
    let listing_buffer = Listing::from_file(config.osu.osu_path.clone())
        .expect("Local Collection couldn't be parsed from a provided path to 'collection.db'");

    let collection_buffer_mutex = Arc::new(Mutex::new(collection_buffer.clone()));
    let listing_buffer_mutex = Arc::new(Mutex::new(listing_buffer.clone()));

    let local_collection_name = format_collection_name(&config.user.collection_name_format, &remote_collection_info);

    info!("{} by {} (with {} beatmaps)", remote_collection_info.name.trim(), remote_collection_info.uploader.username, remote_collection_info.beatmapCount);

    // parallel download
    let downloaded = Arc::new(Mutex::new(0));

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

                let mut downloaded_lock = downloaded.lock().unwrap();
                *downloaded_lock = *downloaded_lock + 1;

                let beatmapset = remote_collection_beatmaps
                    .beatmapsets
                    .iter()
                    .find(|s| s.id == beatmap.beatmapset_id)
                    .unwrap();

                info!("({}/{}) {} - {} [{}]", downloaded_lock, remote_collection_info.beatmapCount, beatmapset.artist, beatmapset.title, beatmap.version)
            }
        })
        .await;
}
