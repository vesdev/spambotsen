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
