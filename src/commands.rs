use crate::common::*;
use rand::{thread_rng, Rng};

#[poise::command(slash_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "Range to roll"] range: u32,
) -> Result<(), Error> {
    let roll = thread_rng().gen_range(0..range).to_string();

    ctx.say(roll).await?;

    Ok(())
}
