use config::{ChannelId, Config};
use eyre::Context;
use platform::bridge::Bridge;
mod commands;
mod common;
mod config;
mod forsen_lines;
mod platform;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1).unwrap_or(&"./spambotsen.toml".into()).clone();

    let config =
        config::from_path(config_path.into()).context("Unable to parse configuration from path")?;

    let bridges = Bridges::new(&config);
    let discord = {
        let config = config.clone();
        async move {
            platform::discord::run(config, bridges.discord)
                .await
                .unwrap();
        }
    };

    let twitch = async move {
        platform::twitch::run(config, bridges.twitch).await.unwrap();
    };

    tokio::select! {
        _ = discord => {},
        _ = twitch => {},
    }

    Ok(())
}

struct Bridges {
    discord: Bridge,
    twitch: Bridge,
}

impl Bridges {
    fn new(config: &Config) -> Self {
        let mut bridges = Self {
            discord: Bridge::new(),
            twitch: Bridge::new(),
        };

        config.bridges.iter().for_each(|(_, bridge)| {
            for channel in &bridge.channels {
                match channel {
                    ChannelId::Discord { .. } => {
                        bridges
                            .twitch
                            .listen(channel.clone(), bridges.discord.sender());
                    }
                    ChannelId::Twitch { .. } => {
                        bridges
                            .discord
                            .listen(channel.clone(), bridges.twitch.sender());
                    }
                };
            }
        });

        bridges
    }
}
