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

use super::bridge::{Bridge, Platform, Sender};

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
    if let poise::Event::Ready { data_about_bot: _ } = event {
        println!("Connected to discord");
    }

    if let poise::Event::Message { new_message } = event {
        let msg = new_message;
        let config = user_data
            .config
            .discord
            .as_ref()
            .ok_or_eyre("Missing discord config!")?;

        // ugly linear search but good enough for the usecase
        if let Some(Some((_, guild))) = msg
            .guild_id
            .map(|guild_id| config.guilds.iter().find(|g| g.1.id == guild_id.0))
        {
            let msg_lowercase = msg.content.to_lowercase();
            for word in msg_lowercase.split_whitespace() {
                for reaction in config
                    .reactions
                    .iter()
                    .filter(|reaction| guild.reactions.contains(reaction.0))
                    .flat_map(|(_, reaction)| {
                        let word = word.to_string();
                        reaction
                            .matches
                            .contains(&word)
                            .then_some(serenity::ReactionType::Custom {
                                animated: reaction.animated,
                                id: serenity::EmojiId(reaction.id),
                                name: Some(word),
                            })
                    })
                {
                    msg.react(ctx, reaction).await?;
                }
            }

            if msg.author.bot {
                return Ok(());
            }

            for response in guild.responses.iter().flat_map(|response| {
                msg.content
                    .contains(response.0)
                    .then(|| match response.1.as_str() {
                        "<forsen line>" => user_data.forsen_lines.get_random(),
                        _ => response.1.clone(),
                    })
            }) {
                msg.channel_id.say(&ctx.http, response).await?;
            }

            let from = ChannelId::Discord {
                id: msg.channel_id.0,
            };

            if let Some(channels) = user_data.bridge.get(&from) {
                let mut sender = user_data.sender.lock().await;
                let text = user_data.bridge.translate(&from, msg.content.clone());
                sender
                    .send(
                        from,
                        channels,
                        Event::SendMessage {
                            name: msg.author.name.clone(),
                            text,
                        },
                    )
                    .await;
            }
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
            let sender = Mutex::new(p.sender.clone());
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
