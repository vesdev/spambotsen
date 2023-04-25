use crate::common::*;

#[poise::command(slash_command)]
pub async fn channel_id(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().to_string();
    ctx.say(channel_id).await?;

    Ok(())
}
