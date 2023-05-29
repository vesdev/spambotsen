use crate::common::*;
use hebi::Hebi;
use poise::CodeBlock;
use rand::{thread_rng, Rng};
use std::time::Duration;

#[poise::command(slash_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "Range to roll"] range: u32,
) -> Result<(), Error> {
    let roll = thread_rng().gen_range(0..range).to_string();

    ctx.say(roll).await?;

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn hebi(
    ctx: Context<'_>,
    #[description = "Hebi code to eval"] source: CodeBlock,
) -> Result<(), Error> {
    ctx.say(crate::hebi::eval_hebi(source.code).await).await?;

    Ok(())
}
