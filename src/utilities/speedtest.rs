use std::{collections::VecDeque, time::Instant};

use tracing::{debug, info};

use crate::config::MirrorType;

async fn test_download(client: &reqwest::Client, payload_size_bytes: usize) -> f64 {
    let req = client.get(format!(
        "https://speed.cloudflare.com/__down?bytes={}",
        payload_size_bytes
    ));

    let resp = req.send().await.unwrap();

    let start = Instant::now();
    let _ = resp.bytes().await.unwrap();
    let elapsed = start.elapsed().as_secs_f64();

    (payload_size_bytes as f64 * 8.0 / 1_000_000.0) / elapsed
}

pub async fn benchmark() {
    let client = reqwest::Client::new();

    // 10mb, 25mb, 50mb (general speedtest)
    info!("running speedtest (general) for 10MB, 25MB and 50MB.");

    let general_speed: Vec<f64> = vec![
        test_download(&client, 10_000_000).await,
        test_download(&client, 25_000_000).await,
        test_download(&client, 50_000_000).await,
    ];

    // 40mb (mirror speedtest)
    info!("running speedtest (mirror)\n");

    let mut file_size: f64 = 0.0;
    let mut mirror_speed: VecDeque<f64> = VecDeque::new();

    for (index, mirror_type) in MirrorType::ALL.iter().enumerate() {
        let mirror = mirror_type.get_mirror();

        let start = Instant::now();

        let file = mirror.get_file(1030499).await.unwrap();
        if file.len() as f64 <= 0.0 {
            continue;
        }

        file_size = file.len() as f64;

        debug!("{} - {}", mirror.get_name(), start.elapsed().as_secs_f64());
        info!(
            "{} done, {} to go.",
            mirror.get_name(),
            MirrorType::ALL.len() - (index + 1)
        );

        mirror_speed.push_front(start.elapsed().as_secs_f64());
    }

    // results
    info!("general speedtest");
    info!(
        "10MB = {:.2}Mb/s | 25MB = {:.2}Mb/s | 50MB = {:.2}Mb/s\n",
        general_speed[0], general_speed[1], general_speed[2]
    );

    info!(
        "mirror speedtest (beatmapset: 1030499 | size: {:.2}MB)",
        file_size / 1024.0 / 1024.0
    );

    for mirror_type in MirrorType::ALL {
        let mirror = mirror_type.get_mirror();
        let mirror_speed = mirror_speed.pop_front();

        if mirror_speed.is_none() {
            continue;
        }

        info!(
            "{} = {:.2}Mb/s",
            mirror.get_name(),
            (file_size * 8.0 / 1_000_000.0) / mirror_speed.unwrap()
        );
    }
}
