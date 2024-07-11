use std::sync::Arc;

use eyre::OptionExt;
use tmi::Credentials;

use crate::{
    config::{ChannelId, Config},
    platform::bridge::{Event, RawEvent},
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
    for bridge in &config.bridges {
        if let ChannelId::Twitch { id } = &bridge.1.from {
            channels.push(id.clone());
        }
    }

    client.join_all(channels.clone()).await?;

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
                        if let Some((to, _)) = bridge.get(&from) {
                            sender
                                .send(RawEvent {
                                    from,
                                    to: to.clone(),
                                    ev: Event::SendMessage {
                                        name: msg.sender().name().into(),
                                        text: msg.text().into(),
                                    },
                                })
                                .await;
                        }
                    }
                    tmi::Message::Reconnect => {
                        client.reconnect().await?;
                        client.join_all(channels.clone()).await?;
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
