use std::sync::Arc;

use config::Config;
use eyre::Context;
use platform::bridge::BridgeBuilder;
use tokio::task::JoinSet;

use crate::platform::bridge::PlatformKind;
mod commands;
mod common;
mod config;
mod forsen_lines;
mod platform;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1).unwrap_or(&"./spambotsen.toml".into()).clone();

    let config =
        config::from_path(config_path.into()).context("Unable to parse configuration from path")?;

    println!("Starting spambotsen...");
    // Restart bot if exception isnt handled gracefully
    loop {
        let result = run(config.clone()).await;

        // TODO: proper logging
        println!("{result:?}");
    }

    Ok(())
}

async fn run(config: Arc<Config>) -> eyre::Result<()> {
    let mut builder = BridgeBuilder::default();

    for b in config.bridges.iter() {
        for b in b.values() {
            let mut translate_from = None;
            let mut translate_to = None;
            if let (Some(t), Some(s)) = (config.translate.as_ref(), b.translate.as_ref()) {
                translate_from = t.get(&s.from).cloned();
                if let Some(to) = s.to.as_ref() {
                    translate_to = t
                        .get(to)
                        .cloned()
                        .map(|m| m.into_iter().map(|(a, b)| (b, a)).collect());
                }
            }

            builder.bridge(
                b.from.clone(),
                b.to.clone(),
                b.symmetric,
                translate_from,
                translate_to,
            );
        }
    }

    let (bridge, platforms) = builder.build();

    let mut set = JoinSet::new();
    for (kind, p) in platforms {
        let config = config.clone();
        let bridge = bridge.clone();
        match kind {
            PlatformKind::Twitch => {
                set.spawn(async move {
                    platform::twitch::run(config, bridge.clone(), p)
                        .await
                        .expect("Disconnected from twitch");
                });
            }
            PlatformKind::Discord => {
                set.spawn(async move {
                    platform::discord::run(config, bridge, p)
                        .await
                        .expect("Disconnected from discord");
                });
            }
        }
    }
    set.join_next().await;
    Ok(())
}
