use eyre::OptionExt;
use tmi::Credentials;

use crate::{
    config::{ChannelId, Config},
    platform::bridge::Event,
};

use super::bridge::Bridge;

pub async fn run(config: Config, mut bridge: Bridge) -> eyre::Result<()> {
    let twitch = config.twitch.ok_or_eyre("Missing twitch config!")?;

    let mut client = tmi::Client::builder()
        .credentials(Credentials {
            login: twitch.user,
            token: Some(twitch.token),
        })
        .connect()
        .await?;

    let mut channels = Vec::new();
    for bridge in config.bridges {
        for id in bridge.1.channels {
            if let ChannelId::Twitch { id } = id {
                channels.push(id.clone());
            }
        }
    }
    dbg!(channels.clone());

    client.join_all(channels.clone()).await?;

    loop {
        let msg = client.recv().await?;
        match msg.as_typed()? {
            tmi::Message::Privmsg(msg) => {
                bridge
                    .send(Event::SendMessage {
                        name: msg.sender().name().into(),
                        text: msg.text().into(),
                    })
                    .await;
            }
            tmi::Message::Reconnect => {
                client.reconnect().await?;
                client.join_all(channels.clone()).await?;
            }
            tmi::Message::Ping(ping) => {
                client.pong(&ping).await?;
            }
            _ => {}
        }
    }
}
