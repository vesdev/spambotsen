use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue,
};

pub fn run(
    options: &[CommandDataOption],
) -> String {
    let option = options
        .get(0)
        .expect("Expected chennel option")
        .resolved
        .as_ref()
        .expect("Expected chennel object");

    if let CommandDataOptionValue::Channel(
        channel,
    ) = option
    {
        format!(
            "{}'s id is {}",
            channel.name.clone().unwrap(),
            channel.id
        )
    } else {
        "Please provide a valid channel"
            .to_string()
    }
}

pub fn register(
    command: &mut CreateApplicationCommand,
) -> &mut CreateApplicationCommand {
    command
        .name("id")
        .description("Get a channel id")
        .create_option(|option| {
            option
                .name("channel-id")
                .description(
                    "The channel id to lookup",
                )
                .kind(CommandOptionType::Channel)
                .required(true)
        })
}
