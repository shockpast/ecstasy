use std::fs;

use serde::{Deserialize, de};

use crate::{
    mirrors::{
        Mirror, beatconnect::Beatconnect, catboy::Catboy, nerinyan::Nerinyan, osudirect::OsuDirect,
        sayobot::Sayobot,
    },
    utilities::osu,
};

pub enum MirrorType {
    Catboy(Catboy),
    OsuDirect(OsuDirect),
    Nerinyan(Nerinyan),
    Beatconnect(Beatconnect),
    Sayobot(Sayobot),
}

impl MirrorType {
    pub const ALL: [MirrorType; 5] = [
        MirrorType::Catboy(Catboy),
        MirrorType::OsuDirect(OsuDirect),
        MirrorType::Nerinyan(Nerinyan),
        MirrorType::Beatconnect(Beatconnect),
        MirrorType::Sayobot(Sayobot),
    ];

    #[allow(clippy::redundant_allocation)]
    pub fn get_mirror(&self) -> Box<&(dyn Mirror + Sync)> {
        match self {
            MirrorType::Catboy(m) => Box::new(m),
            MirrorType::OsuDirect(m) => Box::new(m),
            MirrorType::Nerinyan(m) => Box::new(m),
            MirrorType::Beatconnect(m) => Box::new(m),
            MirrorType::Sayobot(m) => Box::new(m),
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
            "sayobot" => Ok(MirrorType::Sayobot(Sayobot)),
            _ => Err(de::Error::unknown_variant(
                &s,
                &["catboy", "osudirect", "nerinyan", "beatconnect", "sayobot"],
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

#[derive(Default, Deserialize)]
pub struct OsuConfig {
    pub songs_path: String,
    pub collection_path: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub user: UserConfig,
    pub collector: CollectorConfig,
    #[serde(skip_deserializing)]
    pub osu: OsuConfig,
}

pub fn init() -> Config {
    let contents = fs::read_to_string("config.toml").expect("config.toml doesn't exist!");
    let mut config = toml::from_str::<Config>(&contents).unwrap();

    if config.user.concurrent_downloads > 6 {
        panic!("It's highly recommended, that you won't use more than 6 \"threads\" to download maps, otherwise you will get banned from mirrors.");
    }

    let osu_path = osu::find_game().unwrap();
    config.osu.songs_path = format!("{}\\Songs", osu_path);
    config.osu.collection_path = format!("{}\\collection.db", osu_path);

    config
}
