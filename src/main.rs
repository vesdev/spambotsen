use std::{path::PathBuf, sync::Arc, time::Duration};

use common::*;
use eyre::Context;
use forsen_lines::ForsenLines;
use poise::{
    serenity_prelude::{self as serenity, GatewayIntents},
    EditTracker,
};

mod commands;
mod common;
mod config;
mod forsen_lines;
// mod hebi;

async fn event_event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    if let poise::Event::Message { new_message } = event {
        let msg = new_message;

        if msg.author.bot {
            return Ok(());
        }
        let msg_lowercase = msg.content.to_lowercase();
        if msg_lowercase.contains("forsen") {
            msg.channel_id.say(&ctx.http, "forsen").await?;
        }

        if msg.content.contains(":Painsge:") {
            let line = user_data.forsen_lines.get_random();
            msg.channel_id.say(&ctx.http, line).await?;
        }

        if msg_lowercase == "ok"
            || msg_lowercase == "okay"
            || msg_lowercase.contains(" ok ")
            || msg_lowercase.contains(":okay:")
            || msg_lowercase.contains(" okay ")
        {
            msg.react(
                ctx,
                poise::serenity_prelude::ReactionType::Custom {
                    animated: false,
                    id: serenity::EmojiId(user_data.config.ok_emote_id),
                    name: Some("monkahmm".to_string()),
                },
            )
            .await?;
        }

        if msg.content.contains("hmm") {
            msg.react(
                ctx,
                poise::serenity_prelude::ReactionType::Custom {
                    animated: false,
                    id: serenity::EmojiId(user_data.config.hmm_emote_id),
                    name: Some("monkahmm".to_string()),
                },
            )
            .await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1).unwrap_or(&"./spambotsen.toml".into()).clone();

    let config =
        config::from_path(config_path.into()).context("Unable to parse configuration from path")?;

    let discord_token = config.discord_token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::roll()],
            event_handler: |ctx, event, framework, user_data| {
                Box::pin(event_event_handler(ctx, event, framework, user_data))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                edit_tracker: Some(EditTracker::for_timespan(Duration::from_secs(60))),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(discord_token)
        .intents(serenity::GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    forsen_lines: Arc::new(ForsenLines::new(PathBuf::from(
                        "static/forsen_lines.csv",
                    ))),
                    config,
                })
            })
        })
        .build()
        .await
        .context("Failed to build poise framework")?;

    framework.start().await?;
    Ok(())
}
