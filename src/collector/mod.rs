use reqwest::Error;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub uploader: CollectionUploader,
    pub beatmap_count: i16,
    pub beatmapsets: Vec<CollectionInfoBeatmapsets>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionInfoBeatmapsets {
    pub id: i32,
    pub beatmaps: Vec<CollectionInfoBeatmap>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionInfoBeatmap {
    pub checksum: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionBeatmaps {
    pub beatmaps: Vec<Beatmap>,
    pub beatmapsets: Vec<Beatmapset>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionUploader {
    pub username: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CollectionDate {}

#[derive(Clone, Debug, Deserialize)]
struct CollectionModes {}

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
    pub checksum: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Beatmapset {
    pub id: i32,
    pub artist: String,
    pub title: String,
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
