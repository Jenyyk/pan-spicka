mod rozvrh;

use serenity::async_trait;
use serenity::builder::{EditAttachments, EditMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

struct CommandMeta {
    msg: Message,
    context: Context,
}

#[async_trait]
impl EventHandler for Handler {
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
        _ => return Err(format!("Command `{}` not recognized", command)),
    };

    Ok(())
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
