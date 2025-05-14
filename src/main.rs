mod rozvrh;

use serenity::async_trait;
use serenity::builder::{
    CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};
use serenity::builder::{EditAttachments, EditMessage};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

struct CommandMeta {
    msg: Message,
    context: Context,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let _ = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Defer(
                        CreateInteractionResponseMessage::new().content("Přemejšlim... 🤔"),
                    ),
                )
                .await;
            let message = match command.data.name.as_str() {
                "rozvrh" => {
                    let class = get_option_str(&command.data.options, "class").unwrap_or("7B");
                    let time = get_option_str(&command.data.options, "time").unwrap_or("0");

                    let resp = rozvrh::rozvrh_message(vec![class, time].into_iter())
                        .await
                        .unwrap();
                    CreateInteractionResponseFollowup::new()
                        .add_file(resp.attachment)
                        .add_embed(resp.embed)
                }
                _ => CreateInteractionResponseFollowup::new().content("to neexistuje"),
            };

            if let Err(why) = command.create_followup(&ctx.http, message).await {
                println!("Failed responding to slash command: {why:?}");
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            let _ = msg.channel_id.say(&ctx.http, "Pong!").await;
            return;
        }

        let mut message_iterator = msg.content.split(" ");
        if let Some(first_word) = message_iterator.next() {
            if first_word != "kys" && first_word != "186" {
                return;
            }
            let command = match message_iterator.next() {
                Some(command) => command,
                _ => "unknown command",
            };

            match invoke_command(
                CommandMeta {
                    msg: msg.clone(),
                    context: ctx.clone(),
                },
                command,
                message_iterator,
            )
            .await
            {
                Ok(_) => {}
                Err(why) => {
                    println!("Failed to invoke command: {why:?}");
                    let _ = msg
                        .channel_id
                        .say(&ctx.http, format!("Failed to invoke command: {}", why))
                        .await;
                }
            };
        }
    }
}
async fn invoke_command<'a, I>(meta: CommandMeta, command: &str, arguments: I) -> Result<(), String>
where
    I: Iterator<Item = &'a str>,
{
    match command {
        "rozvrh" => {
            let think_msg = meta
                .msg
                .channel_id
                .say(&meta.context.http, "Přemejšlim... 🤔")
                .await;
            let response = rozvrh::rozvrh_message(arguments);

            let edit_builder = match response.await {
                Ok(resp) => EditMessage::new()
                    .content("Bazinga ☝🤓")
                    .embed(resp.embed)
                    .attachments(EditAttachments::new().add(resp.attachment)),
                Err(why) => EditMessage::new().content(format!("Něco se pokazilo: {}", why)),
            };

            if let Ok(mut think_msg_ok) = think_msg {
                if let Err(why) = think_msg_ok.edit(&meta.context.http, edit_builder).await {
                    println!("failed to edit message: {why:?}");
                }
            };
        }

        "register" => {
            if let Some(guild_id) = meta.msg.guild_id {
                let _ = meta
                    .msg
                    .channel_id
                    .say(&meta.context.http, "Registruju / commandy 🤓")
                    .await;
                if let Err(why) = guild_id
                    .set_commands(&meta.context.http, vec![rozvrh::register()])
                    .await
                {
                    println!("failed to register: {why:?}");
                }
            }
        }

        _ => return Err(format!("Command `{}` not recognized", command)),
    };

    Ok(())
}

// helper function to get slash command parameters
use serenity::model::application::CommandDataOption;
fn get_option_str<'a>(options: &'a [CommandDataOption], name: &str) -> Option<&'a str> {
    options.iter().find(|opt| opt.name == name)?.value.as_str()
}

use dotenv::dotenv;
#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("BOT_TOKEN").expect("insert your bot token dumbass");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("failed to create client");

    let _ = client.start().await;
}
