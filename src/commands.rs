use crate::{common::*, platform::discord};

use rand::{thread_rng, Rng};

#[poise::command(slash_command)]
pub async fn roll(
    ctx: discord::Ctx<'_>,
    #[description = "Expression"] input: String,
) -> Result<(), Error> {
    let result = roll_dice::roll(&input, thread_rng().gen(), u64::MAX)?;

    ctx.say(result.to_string()).await?;

    Ok(())
}

// #[poise::command(prefix_command, track_edits, subcommands("disassemble"))]
// pub async fn hebi(
//     ctx: Context<'_>,
//     #[description = "Hebi code to eval"] source: CodeBlock,
// ) -> Result<(), Error> {
//     let mut embed = CreateEmbed::default();
//     embed.description(crate::hebi::eval_hebi(source.code, false).await);
//     ctx.send(|r| {
//         r.embed(|e| {
//             *e = embed.clone();
//             e
//         })
//     })
//     .await;

//     Ok(())
// }

// #[poise::command(prefix_command, track_edits)]
// pub async fn disassemble(
//     ctx: Context<'_>,
//     #[description = "Hebi code to eval"] source: CodeBlock,
// ) -> Result<(), Error> {
//     let mut embed = CreateEmbed::default();
//     embed.description(crate::hebi::eval_hebi(source.code, true).await);
//     ctx.send(|r| {
//         r.embed(|e| {
//             *e = embed.clone();
//             e
//         })
//     })
//     .await;

//     Ok(())
// }
