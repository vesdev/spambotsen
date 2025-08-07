use std::sync::Arc;

use eyre::OptionExt;
use tmi::Credentials;

use crate::{
    config::{ChannelId, Config},
    platform::bridge::Event,
};

use super::bridge::{Bridge, Platform};

pub async fn run(config: Arc<Config>, bridge: Arc<Bridge>, p: Platform) -> eyre::Result<()> {
    let twitch = config
        .twitch
        .as_ref()
        .ok_or_eyre("Missing twitch config!")?;

    let mut client = tmi::Client::builder()
        .credentials(Credentials {
            login: twitch.user.clone(),
            token: Some(twitch.token.clone()),
        })
        .connect()
        .await?;

    let mut channels = Vec::new();
    if let Some(bridges) = config.bridges.as_ref() {
        bridges.iter().for_each(|b| {
            let mut push_channel = |id: &ChannelId| {
                if let ChannelId::Twitch { id } = &id {
                    channels.push(id.clone());
                }
            };
            push_channel(&b.1.from);
            if b.1.symmetric {
                push_channel(&b.1.to);
            }
        });
    }

    client.join_all(channels.clone()).await?;
    println!("Connected to twitch irc");

    let mut sender = p.sender;
    let mut receiver = p.receiver;
    loop {
        tokio::select! {
            ev = receiver.recv() => {
                if let Some(ev) = ev {
                    if let ChannelId::Twitch { id } = ev.to {
                        match ev.ev {
                            Event::SendMessage { name, text } => {
                                client
                                    .privmsg(&id, &format!("{name}: {text}"))
                                    .send()
                                    .await?;
                            }
                        }
                    }
                };
            }
            msg = client.recv() => {
                match msg?.as_typed()? {
                    tmi::Message::Privmsg(msg) => {
                        let from = ChannelId::Twitch {
                             id: msg.channel().into(),
                        };
                        if let Some(channels) = bridge.get(&from) {
                            let text = bridge.translate(&from, msg.text().into());
                            sender
                                .send(
                                    from,
                                    channels,
                                    Event::SendMessage {
                                        name: msg.sender().name().into(),
                                        text,
                                    },
                                )
                                .await;
                        }
                    }
                    tmi::Message::Reconnect => {
                        client.reconnect().await?;
                        client.join_all(channels.clone()).await?;
                        println!("Reconnected to twitch irc");
                    }
                    tmi::Message::Ping(ping) => {
                        client.pong(&ping).await?;
                    }
                    _ => {}
                };
            }
        };
    }
}
