use std::sync::Arc;

use crate::forsen_lines::ForsenLines;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub hmm_emote_id: u64,
    pub ok_emote_id: u64,
}

pub struct Data {
    pub forsen_lines: Arc<ForsenLines>,
    pub config: Config,
    pub rng: std::rand::Rng,
} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
