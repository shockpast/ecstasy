use std::{collections::VecDeque, time::Instant};

use cfspeedtest::speedtest::test_download;
use tracing::info;

use crate::config::MirrorType;

pub async fn benchmark() {
  let client = reqwest::blocking::Client::new();

  // 10mb, 25mb, 50mb (general speedtest)
  let mut general_speed: Vec<f64> = vec![];
  general_speed.push(test_download(&client, 10_000_000, cfspeedtest::OutputFormat::None));
  general_speed.push(test_download(&client, 25_000_000, cfspeedtest::OutputFormat::None));
  general_speed.push(test_download(&client, 50_000_000, cfspeedtest::OutputFormat::None));

  // 40mb (mirror speedtest)
  let mut file_size: f64 = 0.0;
  let mut mirror_speed: VecDeque<f64> = VecDeque::new();
  
  for mirror_type in MirrorType::ALL {
    let mirror = mirror_type.get_mirror();

    let start = Instant::now();

    let file = mirror.get_file(1030499).await.unwrap();
    file_size = file.len() as f64;

    mirror_speed.push_front(start.elapsed().as_secs_f64());
  }

  // results
  info!("general speedtest");
  info!("10MB = {:.2}Mb/s | 25MB = {:.2}Mb/s | 50MB = {:.2}Mb/s\n", general_speed[0], general_speed[1], general_speed[2]);

  info!("mirror speedtest (beatmapset: 1030499 | size: {:.2}MB)", file_size / 1024.0 / 1024.0);

  for mirror_type in MirrorType::ALL {
    let mirror = mirror_type.get_mirror();

    info!("{} = {:.2}Mb/s", mirror.get_name(), (file_size * 8.0 / 1_000_000.0) / mirror_speed.pop_front().unwrap());
  }
}