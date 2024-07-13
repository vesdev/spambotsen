use std::{collections::HashMap, path::PathBuf, sync::Arc};

use serde::Deserialize;

use crate::platform::bridge::PlatformKind;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub discord: Option<Discord>,
    pub twitch: Option<Twitch>,
    pub bridges: Option<HashMap<String, Bridge>>,
    pub translate: Option<HashMap<String, HashMap<String, String>>>,
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
    pub from: ChannelId,
    pub to: ChannelId,
    #[serde(default)]
    pub symmetric: bool,
    pub translate: Option<Translate>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Translate {
    pub from: String,
    pub to: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(tag = "type")]
pub enum ChannelId {
    Discord { id: u64 },
    Twitch { id: String },
}

impl ChannelId {
    pub fn kind(&self) -> PlatformKind {
        match self {
            ChannelId::Discord { .. } => PlatformKind::Discord,
            ChannelId::Twitch { .. } => PlatformKind::Twitch,
        }
    }
}

pub fn from_path(path: PathBuf) -> eyre::Result<Arc<Config>> {
    Ok(Arc::new(toml::from_str::<Config>(
        &std::fs::read_to_string(path)?,
    )?))
}
