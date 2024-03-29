use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::Context as _;
use common::*;
use forsen_lines::ForsenLines;
use poise::{
    serenity_prelude::{self as serenity, GatewayIntents},
    EditTracker, Prefix,
};

use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
mod commands;
mod common;
mod forsen_lines;
mod hebi;

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

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_static_folder::StaticFolder(folder = "static")] static_folder: PathBuf,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> ShuttlePoise<Data, Error> {
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::roll(), commands::hebi()],
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
                    forsen_lines: Arc::new(ForsenLines::new(
                        static_folder.join("forsen_lines.csv"),
                    )),
                    config: toml::from_str(
                        std::fs::read_to_string(static_folder.join("config.toml"))
                            .expect("config.toml not found")
                            .as_str(),
                    )
                    .unwrap(),
                })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
