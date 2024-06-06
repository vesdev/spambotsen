use std::sync::Arc;

use crate::{config::Config, forsen_lines::ForsenLines};

pub struct Data {
    pub forsen_lines: Arc<ForsenLines>,
    pub config: Config,
} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
