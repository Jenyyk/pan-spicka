use std::process::Command;

use serenity::builder::{CreateAttachment, CreateEmbed, CreateMessage};
use serenity::model::Color;

#[allow(dead_code)]
pub struct CreateZmeny {
    pub attachment: CreateAttachment,
    pub embed: CreateEmbed,
    pub message: CreateMessage,
}

pub async fn zmeny_message<'a, I>(mut args: I) -> Result<CreateZmeny, Box<dyn std::error::Error>>
where
    I: Iterator<Item = &'a str>,
{
    let arg = args.next().unwrap_or("7B");
    let class = arg
        .chars()
        .map(|ch| ch.to_string())
        .collect::<Vec<_>>()
        .join(".");

    Command::new("wkhtmltoimage")
        .args([
            "--run-script",
            &format!("[].forEach.call(document.querySelectorAll('table.datagrid > tbody > tr'), function(row) {{ if (row.firstElementChild && row.firstElementChild.textContent.trim() !== '{}') {{ row.style.display = 'none'; }} }});", class),
            "https://bakalari.gypce.cz/bakaweb/next/zmeny.aspx",
            "/tmp/zmeny.png"
        ]).status()?;

    let zmeny_image = &tokio::fs::File::open("/tmp/zmeny.png").await?;

    let attachment = CreateAttachment::file(zmeny_image, "zmeny.png").await?;
    let embed = CreateEmbed::new()
        .title(format!("Změny třídy {}", arg))
        .attachment("zmeny.png")
        .color(Color::from_rgb(5, 180, 255));
    let message = CreateMessage::new()
        .add_file(attachment.clone())
        .embed(embed.clone());

    Ok(CreateZmeny {
        attachment,
        embed,
        message,
    })
}

// slash command pro rozvrh
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;

pub fn register() -> CreateCommand {
    CreateCommand::new("zmeny")
        .description("pošle změny dané třídy")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "class",
                "třída ve formátu 1A, 7B atd...",
            )
            .required(true),
        )
}

pub fn help_message() -> (&'static str, &'static str) {
    (
        "`zmeny ~třída`",
        "Pošle změny dané třídy.\n`~třída` musí mít velké písmeno a být bez tečky, eg. **7B**, **2A**...",
    )
}

use crate::SlashCommand;
pub const COMMAND: SlashCommand = SlashCommand {
    register,
    help: help_message,
};
