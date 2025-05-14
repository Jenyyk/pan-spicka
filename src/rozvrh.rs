use std::process::Command;

use serenity::builder::{CreateAttachment, CreateEmbed, CreateMessage};
use serenity::model::Color;

#[allow(dead_code)]
pub struct CreateRozvrh {
    pub attachment: CreateAttachment,
    pub embed: CreateEmbed,
    pub message: CreateMessage,
}

pub async fn rozvrh_message(
    mut args: std::str::Split<'_, &'static str>,
) -> Result<CreateRozvrh, Box<dyn std::error::Error>> {
    let class = args.next().unwrap_or("7B");
    let time = args.next().unwrap_or("0");
    let (arg, mode) = match class {
        // Třídy (Classes)
        "1A" => ("2G", "Class"),
        "1B" => ("2I", "Class"),
        "1F" => ("2J", "Class"),
        "1H" => ("2K", "Class"),
        "1J" => ("2L", "Class"),
        "2A" => ("2C", "Class"),
        "2B" => ("2D", "Class"),
        "2F" => ("2E", "Class"),
        "2H" => ("2F", "Class"),
        "3A" => ("28", "Class"),
        "3B" => ("29", "Class"),
        "3F" => ("2A", "Class"),
        "3H" => ("2B", "Class"),
        "4A" => ("24", "Class"),
        "4B" => ("25", "Class"),
        "4F" => ("26", "Class"),
        "4H" => ("27", "Class"),
        "5A" => ("20", "Class"),
        "5B" => ("21", "Class"),
        "6A" => ("1W", "Class"),
        "6B" => ("1X", "Class"),
        "7A" => ("1S", "Class"),
        "7B" => ("1T", "Class"),
        "8A" => ("1O", "Class"),
        "8B" => ("1P", "Class"),

        // Učebny (Rooms)
        "106" => ("ZW", "Room"),
        "107" => ("A5", "Room"),
        "108" => ("RR", "Room"),
        "109" => ("84", "Room"),
        "110" => ("35", "Room"),
        "111" => ("GZ", "Room"),
        "203" => ("C0", "Room"),
        "205" => ("U2", "Room"),
        "206" => ("51", "Room"),
        "207" => ("50", "Room"),
        "208" => ("6B", "Room"),
        "303" => ("N5", "Room"),
        "304" => ("32", "Room"),
        "305" => ("1N", "Room"),
        "306" => ("LG", "Room"),
        "307" => ("G4", "Room"),
        "308" => ("VF", "Room"),
        "404" => ("SR", "Room"),
        "405" => ("30", "Room"),
        "406" => ("0X", "Room"),
        "407" => ("FO", "Room"),
        "408" => ("DT", "Room"),
        "409" => ("G0", "Room"),
        "412" => ("9B", "Room"),
        "Bl" => ("YC", "Room"),
        "Bp" => ("FR", "Room"),
        "Br" => ("PL", "Room"),
        "Cl" => ("48", "Room"),
        "Cp" => ("CZ", "Room"),
        "Cr" => ("3S", "Room"),
        "Fl" => ("FV", "Room"),
        "Fp" => ("IP", "Room"),
        "Fr" => ("EF", "Room"),
        "J1" => ("B9", "Room"),
        "J2" => ("T0", "Room"),
        "J3" => ("JI", "Room"),
        "J4" => ("S8", "Room"),
        "MM1" => ("ZY", "Room"),
        "MM2" => ("OG", "Room"),
        "Pv" => ("9A", "Room"),
        "ŠJ" => ("01", "Room"),
        "T1" => ("5Z", "Room"),
        "T2" => ("00", "Room"),
        "T3" => ("NP", "Room"),
        "T4" => ("ZZ", "Room"),
        "V1" => ("38", "Room"),
        "V2" => ("77", "Room"),
        "V3" => ("3Z", "Room"),
        "V4" => ("5H", "Room"),

        // Default
        _ => return Err("Nechápu, kterej rozvrh chceš 🤔".into()),
    };
    let time = match time {
        "+1" => "Next",
        _ => "Actual",
    };

    let path = format!(
        "https://bakalari.gypce.cz/bakaweb/Timetable/Public/{}/{}/{}",
        time, mode, arg
    );

    Command::new("wkhtmltoimage")
        .args([
            "--run-script",
            "document.getElementById('c-p-bn').click()",
            &path,
            "/tmp/rozvrh.png",
        ])
        .status()?;

    let rozvrh_image = &tokio::fs::File::open("/tmp/rozvrh.png").await?;

    let attachment = CreateAttachment::file(rozvrh_image, "rozvrh.png").await?;
    let embed = CreateEmbed::new()
        .title(format!("rozvrh pro {}", class))
        .attachment("rozvrh.png")
        .color(Color::from_rgb(5, 180, 255));
    let message = CreateMessage::new()
        .add_file(attachment.clone())
        .embed(embed.clone());

    Ok(CreateRozvrh {
        attachment,
        embed,
        message,
    })
}
