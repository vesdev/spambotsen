use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use forsen_lines::ForsenLines;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info};

mod forsen_lines;

struct PepePains;

impl TypeMapKey for PepePains {
    type Value = Arc<ForsenLines>;
}

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if msg.content.contains("forsen") {
            if let Err(e) = msg.channel_id.say(&ctx.http, "forsen").await {
                error!("Error sending message: {:?}", e);
            }
        }

        if msg.content.contains(":Painsge:") {
            let line = {
                let data_read = ctx.data.read().await;
                data_read
                    .get::<PepePains>()
                    .expect("Expected PepePains")
                    .clone()
            }
            .get_random();

            if let Err(e) = msg.channel_id.say(&ctx.http, line).await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_service::main]
async fn serenity(
    #[shuttle_static_folder::StaticFolder(folder = "static")] static_folder: PathBuf,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_service::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<PepePains>(Arc::new(ForsenLines::new(
            static_folder.join("forsen_lines.csv"),
        )));
    }

    Ok(client)
}
