use eyre::Context;
use platform::bridge::BridgeBuilder;

use crate::platform::bridge::PlatformKind;
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

    let mut builder = BridgeBuilder::new();

    config.bridges.iter().for_each(|(_, brdg)| {
        builder.bridge(brdg.from.clone(), brdg.to.clone());
    });

    let (bridge, platforms) = builder.build();

    platforms.into_iter().for_each(|(kind, p)| {
        let config = config.clone();
        let bridge = bridge.clone();
        match kind {
            PlatformKind::Twitch => {
                tokio::spawn(async move {
                    platform::twitch::run(config, bridge.clone(), p)
                        .await
                        .unwrap();
                });
            }
            PlatformKind::Discord => {
                tokio::spawn(async move {
                    platform::discord::run(config, bridge, p).await.unwrap();
                });
            }
        }
    });

    // -1 worker LULE dont look
    #[allow(clippy::empty_loop)]
    loop {}
}
