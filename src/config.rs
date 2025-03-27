use std::fs;

use serde::{Deserialize, de};

use crate::mirrors::{
    Mirror, beatconnect::Beatconnect, catboy::Catboy, nerinyan::Nerinyan, osudirect::OsuDirect,
};

pub enum MirrorType {
    Catboy(Catboy),
    OsuDirect(OsuDirect),
    Nerinyan(Nerinyan),
    Beatconnect(Beatconnect),
}

impl MirrorType {
    pub const ALL: [MirrorType; 4] = [
        MirrorType::Catboy(Catboy),
        MirrorType::OsuDirect(OsuDirect),
        MirrorType::Nerinyan(Nerinyan),
        MirrorType::Beatconnect(Beatconnect),
    ];

    #[allow(clippy::redundant_allocation)]
    pub fn get_mirror(&self) -> Box<&(dyn Mirror + Sync)> {
        match self {
            MirrorType::Catboy(m) => Box::new(m),
            MirrorType::OsuDirect(m) => Box::new(m),
            MirrorType::Nerinyan(m) => Box::new(m),
            MirrorType::Beatconnect(m) => Box::new(m),
        }
    }
}

impl<'de> Deserialize<'de> for MirrorType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "catboy" => Ok(MirrorType::Catboy(Catboy)),
            "osudirect" => Ok(MirrorType::OsuDirect(OsuDirect)),
            "nerinyan" => Ok(MirrorType::Nerinyan(Nerinyan)),
            "beatconnect" => Ok(MirrorType::Beatconnect(Beatconnect)),
            _ => Err(de::Error::unknown_variant(
                &s,
                &["catboy", "osudirect", "nerinyan", "beatconnect"],
            )),
        }
    }
}

//
#[derive(Deserialize)]
pub struct UserConfig {
    pub mirror_type: MirrorType,
    pub collection_name_format: String,
    pub concurrent_downloads: usize,
}

#[derive(Deserialize)]
pub struct CollectorConfig {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct OsuConfig {
    pub songs_path: String,
    pub collection_path: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub user: UserConfig,
    pub collector: CollectorConfig,
    pub osu: OsuConfig,
}

pub fn init() -> Config {
    let contents = fs::read_to_string("config.toml").expect("config.toml doesn't exist!");
    toml::from_str::<Config>(&contents).unwrap()
}
