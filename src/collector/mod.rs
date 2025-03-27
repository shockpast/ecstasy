use std::collections::HashMap;

use reqwest::Error;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub _description: Option<String>,
    pub uploader: CollectionUploader,
    _date_uploaded: CollectionDate,
    _date_last_modified: CollectionDate,
    pub beatmap_count: i16,
    _favourites: i16,
    _comments: Vec<String>,
    _unsubmitted_beatmap_count: i16,
    _unknown_checksums: Vec<String>,
    pub beatmapsets: Vec<CollectionInfoBeatmapsets>,
    _modes: CollectionModes,
    _difficulty_spread: HashMap<i32, f64>,
    _bpm_spread: HashMap<i32, f64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionInfoBeatmapsets {
    pub id: i32,
    pub beatmaps: Vec<CollectionInfoBeatmap>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionInfoBeatmap {
    pub _id: i32,
    pub checksum: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionBeatmaps {
    pub _beatmaps: Vec<Beatmap>,
    pub beatmapsets: Vec<Beatmapset>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionUploader {
    pub _id: i32,
    pub username: String,
    pub _rank: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
struct CollectionDate {
    _seconds: i64,
    _nanoseconds: i64,
}

#[derive(Clone, Debug, Deserialize)]
struct CollectionModes {
    _osu: i16,
    _taiko: i16,
    _fruits: i16,
    _mania: i16,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Gamemode {
    Osu,
    Taiko,
    Fruits,
    Mania,
}

#[derive(Clone, Debug, Deserialize)]
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
    Wip,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Beatmap {
    pub _id: i32,
    pub _beatmapset_id: i32,
    pub _checksum: String,
    pub _version: String,
    _mode: Gamemode,
    _difficulty_rating: f32,
    _accuracy: f32,
    _drain: f32,
    _bpm: f32,
    _cs: f32,
    _ar: f32,
    _hit_length: i16,
    pub _status: BeatmapStatus,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Beatmapset {
    pub id: i32,
    _creator: String,
    pub artist: String,
    pub _artist_unicode: String,
    pub title: String,
    pub _title_unicode: String,
    _bpm: f32,
    _cover: String,
    _submitted_date: String,
    _favourite_count: Option<i16>,
    pub _status: Option<BeatmapStatus>,
}

const BASE_URL: &str = "https://osucollector.com/api";

pub async fn get_info(id: i32) -> Result<Collection, Error> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/collections/{}", BASE_URL, id))
        .send()
        .await?
        .json::<Collection>()
        .await?;

    Ok(response)
}

pub async fn get_beatmaps(id: i32) -> Result<CollectionBeatmaps, Error> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/collections/{}/beatmapsv3", BASE_URL, id))
        .send()
        .await?
        .json::<CollectionBeatmaps>()
        .await?;

    Ok(response)
}
