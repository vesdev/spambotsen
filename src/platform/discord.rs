use std::{path::PathBuf, sync::Arc, time::Duration};

use common::*;
use eyre::{Context, OptionExt};
use forsen_lines::ForsenLines;
use poise::{
    serenity_prelude::{self as serenity, GatewayIntents},
    EditTracker,
};

use crate::{
    commands, common,
    config::{self, ChannelId, Config},
    forsen_lines,
    platform::bridge::Event,
};

use super::bridge::Bridge;

pub type Ctx<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub forsen_lines: Arc<ForsenLines>,
    pub config: config::Discord,
} // User data, which is stored and accessible in all command invocations

async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    if let poise::Event::Message { new_message } = event {
        let msg = new_message;

        let msg_lowercase = msg.content.to_lowercase();
        let words = msg_lowercase.split_whitespace();
        for word in words {
            for (_, reaction) in user_data.config.reactions.iter() {
                if reaction.matches.contains(&word.into()) {
                    msg.react(
                        ctx,
                        poise::serenity_prelude::ReactionType::Custom {
                            animated: reaction.animated,
                            id: serenity::EmojiId(reaction.id),
                            name: Some(word.into()),
                        },
                    )
                    .await?;
                }
            }
        }

        if msg.author.bot {
            return Ok(());
        }
        if msg_lowercase.contains("forsen") {
            msg.channel_id.say(&ctx.http, "forsen").await?;
        }

        if msg.content.contains(":Painsge:") {
            let line = user_data.forsen_lines.get_random();
            msg.channel_id.say(&ctx.http, line).await?;
        }
    }

    Ok(())
}

pub async fn run(config: Config, mut bridge: Bridge) -> eyre::Result<()> {
    let discord = config.discord.ok_or_eyre("Missing discord config!")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::roll()],
            event_handler: |ctx, event, framework, user_data| {
                Box::pin(event_handler(ctx, event, framework, user_data))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                edit_tracker: Some(EditTracker::for_timespan(Duration::from_secs(60))),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(discord.token.clone())
        .intents(serenity::GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    forsen_lines: Arc::new(ForsenLines::new(PathBuf::from(
                        "static/forsen_lines.csv",
                    ))),
                    config: discord.clone(),
                })
            })
        })
        .build()
        .await
        .context("Failed to build poise framework")?;

    let handle_bridge = {
        let http = framework.client().cache_and_http.http.clone();
        async move {
            loop {
                let ev = bridge.recv();
                if let Some((ChannelId::Discord { id }, ev)) = ev.await {
                    match ev {
                        Event::SendMessage { name, text } => {
                            poise::serenity_prelude::ChannelId(id)
                                .say(&http, format!("{name}: {text}"))
                                .await
                                .unwrap();
                        }
                    }
                };
            }
        }
    };

    tokio::select! {
        _ = framework.start() => {},
        _ = handle_bridge => {},
    }
    Ok(())
}
