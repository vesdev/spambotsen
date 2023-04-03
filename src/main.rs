use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use forsen_lines::ForsenLines;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::prelude::*;
use serenity::{async_trait, model::prelude::GuildId};
use shuttle_secrets::SecretStore;
use tracing::{error, info};

mod commands;
mod forsen_lines;

use commands::channel_id::*;
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

        if msg.content.to_lowercase().contains("forsen") {
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

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "clear-commands" => {
                    commands::clear_commands::run(&command.data.options, &ctx).await
                }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::channel_id::register(command)
        })
        .await;
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_static_folder::StaticFolder(folder = "static")] static_folder: PathBuf,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
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

    Ok(client.into())
}
