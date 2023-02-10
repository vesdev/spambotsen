use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::{CommandOptionType, Command};
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue,
};
use serenity::prelude::Context;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
) -> String {
    for command in
        Command::get_global_application_commands(
            &ctx.http,
        )
        .await
        .unwrap()
    {
        Command::delete_global_application_command(&ctx.http, command.id).await.unwrap();
    }
    format!("cleared")
}

pub fn register(
    command: &mut CreateApplicationCommand,
) -> &mut CreateApplicationCommand {
    command
        .name("clear-commands")
        .description("clear global commands")
}
