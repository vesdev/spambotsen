use crate::common::*;
use rand::{seq::SliceRandom, thread_rng};

#[poise::command(slash_command)]
pub async fn channel_id(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().to_string();
    ctx.say(channel_id).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn roll(
    ctx: Context<'_>
    #[description = "Range to roll"]
    range: u32,
    ) -> Result<(), Error> {
    let mut rng = rand::thread_rng();
    ctx.say(rng.gen_range(0..range).to_string());
}
