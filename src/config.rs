use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub discord: Option<Discord>,
    pub twitch: Option<Twitch>,
    pub bridges: HashMap<String, Bridge>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Emote {
    pub id: u64,
    pub matches: Vec<String>,
    #[serde(default)]
    pub animated: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Discord {
    pub token: String,
    pub reactions: HashMap<String, Emote>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Twitch {
    pub user: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Bridge {
    pub channels: Vec<ChannelId>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(tag = "type")]
pub enum ChannelId {
    Discord { id: u64 },
    Twitch { id: String },
}

pub fn from_path(path: PathBuf) -> eyre::Result<Config> {
    Ok(toml::from_str::<Config>(&std::fs::read_to_string(path)?)?)
}
