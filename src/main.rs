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

    let mut builder = BridgeBuilder::default();

    for b in &config.bridges {
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
