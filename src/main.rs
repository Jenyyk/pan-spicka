mod chatbot;
mod database;
mod lunch_fetch;
mod rozvrh;
mod zmeny;

use serenity::{
    async_trait,
    builder::{
        CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage, EditAttachments, EditMessage,
    },
    gateway::ActivityData,
    model::{application::Interaction, channel::Message, prelude::Ready},
    prelude::*,
};

use database::Database;

struct CommandMeta {
    msg: Message,
    context: Context,
}

pub struct SlashCommand {
    register: fn() -> CreateCommand,
    help: fn() -> (&'static str, &'static str),
}

const SLASH_COMMANDS: [SlashCommand; 4] = [
    rozvrh::COMMAND,
    lunch_fetch::COMMAND,
    chatbot::COMMAND,
    zmeny::COMMAND,
];

// Event Handler implementations
struct Handler;
#[async_trait]
impl EventHandler for Handler {
    // Bot start
    async fn ready(&self, ctx: Context, _ready: Ready) {
        // register global commands
        let mut global_commands = Vec::new();
        for command in SLASH_COMMANDS {
            global_commands.push((command.register)());
        }
        match serenity::model::application::Command::set_global_commands(&ctx.http, global_commands)
            .await
        {
            Ok(_) => println!("Succesfully registered global commands"),
            Err(why) => println!("Error registering global commands: {why:?}"),
        };

        // set custom activity
        ctx.set_activity(Some(ActivityData::custom("Kontroluju boty")));
    }

    // Slash command handler
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
                "zmeny" => {
                    let class = get_option_str(&command.data.options, "class").unwrap_or("7B");

                    let resp = zmeny::zmeny_message(vec![class].into_iter()).await.unwrap();
                    CreateInteractionResponseFollowup::new()
                        .add_file(resp.attachment)
                        .add_embed(resp.embed)
                }
                "ai" => {
                    let arg = get_option_str(&command.data.options, "message").unwrap_or("");

                    let resp = chatbot::chat(arg).await.unwrap();
                    CreateInteractionResponseFollowup::new().content(resp)
                }
                "help" => CreateInteractionResponseFollowup::new().add_embed(help_content()),
                "obedy" => {
                    let days_forward = get_option_str(&command.data.options, "days_forward")
                        .unwrap_or("0")
                        .parse::<i64>()
                        .unwrap_or(0);
                    match lunch_fetch::get_lunch_embed(days_forward) {
                        Ok(vec) => CreateInteractionResponseFollowup::new().embeds(vec),
                        Err(why) => CreateInteractionResponseFollowup::new()
                            .content(format!("Command failed: {}", why)),
                    }
                }
                _ => CreateInteractionResponseFollowup::new().content("to neexistuje"),
            };

            if let Err(why) = command.create_followup(&ctx.http, message).await {
                println!("Failed responding to slash command: {why:?}");
            }
        }
    }

    // Message handler
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            let _ = msg.channel_id.say(&ctx.http, "Pong!").await;
            return;
        }

        let mut message_iterator = msg.content.split(" ");

        let self_id = ctx.cache.current_user().id;
        // checks if bot was mentioned
        // This is so we can use the ai command to respond to the bot being mentioned
        if msg.mentions.iter().any(|user| user.id == self_id) {
            let _ = invoke_command(
                CommandMeta {
                    msg: msg.clone(),
                    context: ctx.clone(),
                },
                "ai",
                message_iterator,
            )
            .await;
            return;
        }

        // Return if message isnt a prefix command
        if let Some(first_word) = message_iterator.next() {
            if first_word != "kys" && first_word != "!ps" && first_word != "186" {
                return;
            }
        } else {
            return;
        }

        let command = match message_iterator.next() {
            Some(command) => command,
            _ => "unknown command",
        };

        // Run the command
        if let Err(why) = invoke_command(
            CommandMeta {
                msg: msg.clone(),
                context: ctx.clone(),
            },
            command,
            message_iterator,
        )
        .await
        {
            println!("Failed to invoke command: {why:?}");
            let _ = msg
                .channel_id
                .say(&ctx.http, format!("Failed to invoke command: {}", why))
                .await;
        }
    }
}

// Invoke message commands
async fn invoke_command<'a, I>(
    meta: CommandMeta,
    command: &str,
    mut arguments: I,
) -> Result<(), String>
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

        "zmeny" => {
            let think_msg = meta
                .msg
                .channel_id
                .say(&meta.context.http, "Přemejšlim... 🤔")
                .await;
            let response = zmeny::zmeny_message(arguments);

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
                    .set_commands(
                        &meta.context.http,
                        vec![
                            rozvrh::register(),
                            zmeny::register(),
                            chatbot::register(),
                            lunch_fetch::register(),
                            CreateCommand::new("help").description("zašle pomocné menu"),
                        ],
                    )
                    .await
                {
                    println!("failed to register: {why:?}");
                }
            }
        }
        "unregister" => {
            if let Some(guild_id) = meta.msg.guild_id {
                let _ = meta
                    .msg
                    .channel_id
                    .say(&meta.context.http, "Odebírám / commandy 🤓")
                    .await;
                if let Err(why) = guild_id.set_commands(&meta.context.http, vec![]).await {
                    println!("failed to unregister: {why:?}");
                }
            }
        }

        "help" => {
            if let Err(why) = meta
                .msg
                .channel_id
                .send_message(
                    &meta.context.http,
                    CreateMessage::new().embed(match arguments.next() {
                        Some("rozvrh") => CreateEmbed::new().help_field(rozvrh::help_message()),
                        Some("ai") => CreateEmbed::new().help_field(chatbot::help_message()),
                        Some("obedy") => CreateEmbed::new().help_field(lunch_fetch::help_message()),
                        Some("zmeny") => CreateEmbed::new().help_field(zmeny::help_message()),
                        _ => help_content(),
                    }),
                )
                .await
            {
                println!("Failed sending help: {why:?}");
            }
        }

        "ai" => {
            let think_msg = meta
                .msg
                .channel_id
                .say(&meta.context.http, "Přemejšlim... 🤔")
                .await;

            let ai_response = match chatbot::chat(&arguments.collect::<Vec<&str>>().join(" ")).await
            {
                Ok(resp) => resp,
                Err(why) => format!("Failed talking with AI: {}", why),
            };

            let edit_builder = EditMessage::new().content(ai_response);

            if let Ok(mut think_msg_ok) = think_msg {
                if let Err(why) = think_msg_ok.edit(&meta.context.http, edit_builder).await {
                    println!("failed to edit message: {why:?}");
                }
            };
        }

        "status" => {
            if meta.msg.author.id.to_string() != "416295343198568458" {
                println!("Dev command executed by not admin");
                return Err(String::from(
                    "Insufficient permissions: Only admins can use this command.",
                ));
            }
            match arguments.next() {
                Some("competing") => meta.context.set_activity(Some(ActivityData::competing(
                    arguments.collect::<Vec<&str>>().join(" "),
                ))),
                Some("custom") => meta.context.set_activity(Some(ActivityData::custom(
                    arguments.collect::<Vec<&str>>().join(" "),
                ))),
                Some("listening") => meta.context.set_activity(Some(ActivityData::listening(
                    arguments.collect::<Vec<&str>>().join(" "),
                ))),
                Some("playing") => meta.context.set_activity(Some(ActivityData::playing(
                    arguments.collect::<Vec<&str>>().join(" "),
                ))),
                Some("watching") => meta.context.set_activity(Some(ActivityData::watching(
                    arguments.collect::<Vec<&str>>().join(" "),
                ))),
                _ => meta.context.set_activity(None),
            }
        }

        "obedy" => {
            let days_forward = arguments.next().unwrap_or("0").parse::<i64>().unwrap_or(0);

            let embed_vec = match lunch_fetch::get_lunch_embed(days_forward) {
                Ok(vec) => vec,
                Err(e) => return Err(e.to_string()),
            };

            let _ = meta
                .msg
                .channel_id
                .send_message(&meta.context, CreateMessage::new().embeds(embed_vec))
                .await;
        }

        "announcements" => {
            // first check permissions
            let guild_id = match meta.msg.guild_id {
                Some(gid) => gid,
                None => return Err(String::from("Failed getting guild id")),
            };
            let channel = match meta.msg.channel_id.to_channel(&meta.context.http).await {
                Ok(ch) => match ch.guild() {
                    Some(gch) => gch,
                    None => return Err(String::from("Failed getting guild channel")),
                },
                Err(why) => return Err(format!("Failed getting channel: {}", why)),
            };
            let member = match guild_id
                .member(&meta.context.http, meta.msg.author.id)
                .await
            {
                Ok(member) => member,
                Err(why) => return Err(format!("Failed getting member object: {}", why)),
            };
            match guild_id.to_guild_cached(&meta.context.cache) {
                Some(guild) => {
                    if !guild
                        .user_permissions_in(&channel, &member)
                        .manage_channels()
                    {
                        return Err(String::from("Insufficient permissions"));
                    }
                }
                None => {
                    return Err(String::from("Failed checking permissions"));
                }
            }

            let mut to_set = Some(meta.msg.channel_id.to_string());
            if arguments.next() == Some("disable") {
                to_set = None
            }

            if let Err(why) = Database::set_announcement_channel(guild_id.to_string(), to_set) {
                return Err(why.to_string());
            }
            let _ = meta
                .msg
                .channel_id
                .say(&meta.context.http, "Kanál nastaven!")
                .await;
        }

        "announce" => {
            // first check permissions
            if meta.msg.author.id.to_string() != "416295343198568458" {
                println!("Dev command executed by not admin");
                return Err(String::from(
                    "Insufficient permissions: Only admins can use this command.",
                ));
            }

            let message_content = arguments.collect::<Vec<&str>>().join(" ");
            let data = match Database::get_data() {
                Ok(data) => data,
                Err(why) => return Err(format!("Failed to get database: {}", why)),
            };

            for (_id_str, srv_data) in data {
                if let Some(channel_id_str) = &srv_data.announcement_channel {
                    let channel_id = match channel_id_str.parse::<u64>() {
                        Ok(id) => serenity::model::id::ChannelId::new(id),
                        Err(why) => {
                            println!("Failed to parse channel id: {}", why);
                            continue;
                        }
                    };

                    if let Err(why) = channel_id.say(&meta.context.http, &message_content).await {
                        println!("Failed to send message to channel {}: {}", channel_id, why);
                    }
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

fn help_content() -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .field(
            "Prefixy",
            "Pan Špička přijímá tyto prefixy před zprávami jako příkazy:\n`!ps`, `kys`, `186`",
            false,
        )
        .field(
            "Lomítkové Příkazy",
            "Pan Špička také přijímá discordem podporované příkazy začínající s `/`",
            false,
        )
        .field("Pan Špička podporuje tyto příkazy:", "", false);
    for command in SLASH_COMMANDS {
        embed = embed.help_field((command.help)());
    }
    embed
}
// YES this is ABSOLUTELY NEEDED
pub trait HelpField {
    fn help_field(self, args: (&str, &str)) -> Self;
}
impl HelpField for CreateEmbed {
    fn help_field(self, args: (&str, &str)) -> Self {
        self.field(args.0, args.1, false)
    }
}

use dotenv::dotenv;
#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("BOT_TOKEN").expect("insert your bot token dumbass");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("failed to create client");

    println!("Launching pan-spicka v0.1.3");
    let _ = client.start().await;
}
