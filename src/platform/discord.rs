use std::{path::PathBuf, sync::Arc, time::Duration};

use common::*;
use eyre::{Context, OptionExt};
use forsen_lines::ForsenLines;
use poise::{
    serenity_prelude::{self as serenity, GatewayIntents},
    EditTracker,
};
use tokio::sync::Mutex;

use crate::{
    commands, common,
    config::{ChannelId, Config},
    forsen_lines,
    platform::bridge::Event,
};

use super::bridge::{Bridge, Platform, RawEvent, Sender};

pub type Ctx<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub forsen_lines: Arc<ForsenLines>,
    pub config: Arc<Config>,
    pub bridge: Arc<Bridge>,
    pub sender: Mutex<Sender>,
} // User data, which is stored and accessible in all command invocations

async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    if let poise::Event::Message { new_message } = event {
        let msg = new_message;
        let config = user_data
            .config
            .discord
            .as_ref()
            .ok_or_eyre("Missing discord config!")?;

        let msg_lowercase = msg.content.to_lowercase();
        let words = msg_lowercase.split_whitespace();
        for word in words {
            for (_, reaction) in config.reactions.iter() {
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

        let from = ChannelId::Discord {
            id: msg.channel_id.0,
        };

        if let Some((to, _)) = user_data.bridge.get(&from) {
            let mut sender = user_data.sender.lock().await;
            sender
                .send(RawEvent {
                    from,
                    to: to.clone(),
                    ev: Event::SendMessage {
                        name: msg.author.name.clone(),
                        text: msg.content.clone(),
                    },
                })
                .await;
        }
    }

    Ok(())
}

pub async fn run(config: Arc<Config>, bridge: Arc<Bridge>, p: Platform) -> eyre::Result<()> {
    let discord = config
        .discord
        .as_ref()
        .ok_or_eyre("Missing discord config!")?;

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
        .setup({
            let bridge = bridge.clone();
            let sender = Mutex::new(p.sender);
            |ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(Data {
                        forsen_lines: Arc::new(ForsenLines::new(PathBuf::from(
                            "static/forsen_lines.csv",
                        ))),
                        config: config.clone(),
                        bridge,
                        sender,
                    })
                })
            }
        })
        .build()
        .await
        .context("Failed to build poise framework")?;

    let handle_bridge = {
        let http = framework.client().cache_and_http.http.clone();
        let mut r = p.receiver;
        async move {
            loop {
                let ev = r.recv().await;
                if let Some(ev) = ev {
                    if let ChannelId::Discord { id } = ev.to {
                        match ev.ev {
                            Event::SendMessage { name, text } => {
                                poise::serenity_prelude::ChannelId(id)
                                    .say(&http, format!("{name}: {text}"))
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                };
            }
        }
    };

    tokio::select! {
        _ = framework.start() => {},
        _ = handle_bridge => {},
    };

    Ok(())
}
