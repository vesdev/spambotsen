use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub discord_token: String,
    pub hmm_emote_id: u64,
    pub ok_emote_id: u64,
}

pub fn from_path(path: PathBuf) -> eyre::Result<Config> {
    Ok(toml::from_str::<Config>(&std::fs::read_to_string(path)?)?)
}
