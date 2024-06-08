use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub discord_token: String,
    pub reactions: HashMap<String, Emote>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Emote {
    pub id: u64,
    pub matches: Vec<String>,
    #[serde(default)]
    pub animated: bool,
}

pub fn from_path(path: PathBuf) -> eyre::Result<Config> {
    Ok(toml::from_str::<Config>(&std::fs::read_to_string(path)?)?)
}
