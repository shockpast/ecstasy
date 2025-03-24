use std::collections::HashMap;

use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
  pub id: i32,
  pub name: String,
  pub description: Option<String>,
  pub uploader: CollectionUploader,
  date_uploaded: CollectionDate,
  date_last_modified: CollectionDate,
  pub beatmap_count: i16,
  favourites: i16,
  comments: Vec<String>,
  unsubmitted_beatmap_count: i16,
  unknown_checksums: Vec<String>,
  modes: CollectionModes,
  difficulty_spread: HashMap<i32, f64>,
  bpm_spread: HashMap<i32, f64>
}

#[derive(Deserialize)]
pub struct CollectionBeatmaps {
  pub beatmaps: Vec<Beatmap>,
  pub beatmapsets: Vec<Beatmapset>
}

#[derive(Deserialize)]
pub struct CollectionUploader {
  pub id: i32,
  pub username: String,
  pub rank: Option<i32>
}

#[derive(Deserialize)]
struct CollectionDate {
  _seconds: i64,
  _nanoseconds: i64
}

#[derive(Deserialize)]
struct CollectionModes {
  osu: i16,
  taiko: i16,
  fruits: i16,
  mania: i16
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Gamemode {
  Osu,
  Taiko,
  Fruits,
  Mania
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BeatmapStatus {
  Ranked,
  Loved,
  Graveyard,
  Qualified,
  Unranked,
  Pending,
  Approved,
  NotSubmitted,
  WIP
}

#[derive(Deserialize)]
pub struct Beatmap {
  pub id: i32,
  pub beatmapset_id: i32,
  pub checksum: String,
  pub version: String,
  mode: Gamemode,
  difficulty_rating: f32,
  accuracy: f32,
  drain: f32,
  bpm: f32,
  cs: f32,
  ar: f32,
  hit_length: i16,
  pub status: BeatmapStatus
}

#[derive(Deserialize)]
pub struct Beatmapset {
  pub id: i32,
  creator: String,
  pub artist: String,
  pub artist_unicode: String,
  pub title: String,
  pub title_unicode: String,
  bpm: f32,
  cover: String,
  submitted_date: String,
  favourite_count: Option<i16>,
  pub status: Option<BeatmapStatus>
}

const BASE_URL: &'static str = "https://osucollector.com/api";

pub async fn get_info(id: i32) -> Result<Collection, Error> {
  let client = reqwest::Client::new();

  let response = client.get(format!("{}/collections/{}", BASE_URL, id))
    .send()
    .await?
    .json::<Collection>()
    .await?;

  Ok(response)
}

pub async fn get_beatmaps(id: i32) -> Result<CollectionBeatmaps, Error> {
  let client = reqwest::Client::new();

  let response = client.get(format!("{}/collections/{}/beatmapsv3", BASE_URL, id))
    .send()
    .await?
    .json::<CollectionBeatmaps>()
    .await?;

  Ok(response)
}