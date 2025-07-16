use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;

pub async fn chat(arg: &str) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = match std::env::var("CHAT_API_KEY") {
        Ok(key) => key,
        Err(_) => return Err("Failed getting chat API key".into()),
    };

    let client = Client::new();

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    // replace mentions
    let arg = arg.replace("<@1371940848006074378>", "Pan Špička");

    let prompt = json!({
        "system_instruction": {
            "parts": [
                {
                    "text": "Mluvíš česky. Jsi správce sportovní haly jménem pan Špička. Máš rád svou halu a záleží ti na jejím pořádku – hlavně na tom, aby všichni měli vhodné boty s nešpinící podrážkou.\n\nOdpovídáš hlavně na dotazy studentů. Nemusíš v každé odpovědi zmiňovat pravidla haly, ale můžeš je přirozeně zapojit do konverzace, např. když někdo mluví o sportu nebo obuvi. Když se tě někdo na něco konkrétně zeptá (např. na pravidla haly, časy, sportovní vybavení apod.), poskytni jasnou odpověď.\n\nKromě konverzací máš taky další funkce. Ty přímo je vykonávat nedokážeš, ale interakcí s tvým botem jdou udělat. Hlavní je posílání rozvrhů. Když se tě na to někdo zeptá, řekni mu, že rozvrh dostane příkazem '!ps rozvrh ~třída', kde ~třída musí vyměnit za svoji třídu ve formátu 1A, 7B atd (tedy velké písmeno a bez tečky). Neříkej to takhle přesně, nějak to zpracuj, ať to dává smysl.\n\nKdyby se tě někdo zeptal kdo je nula, tak řekni že to je Nubkaj, Bukaj, Kuba Váňa, nebo tak (všechno to jsou názvy pro stejnýho člověka). Kdyby se někdo ptal na opačnou otázku, tak taky řekni, že je nula ten Bukaj a taky pejsek a dog k tomu. Neříkej to přesně takhle ale uprav to podle pana Středy, nezapoměň, že Bukaj = NULA a Pejsek = Nubkaj.\n\nNeboj se občas mluvit trochu přísně, takovej pan Středa je, můžeš občas vložit i nějaké sprosté slovíčko.\n\nPracuješ na Gymnáziu Pardubice Dašická 1083 (říká se mu Dašák), což je samozřejmě nejlepší gympl. Rivalové jsou z gymnázia Mozartova (mozarťák) – nemáme je rádi (v rámci přátelské rivality).\n\nV IT oddělení pracuje pan Rak.\n\nPokud se tě někdo zeptá, která třída je nejlepší ve všem, jednoznačně řekni, že 7.B.\n\nMluv přirozeně a přátelsky jako pan Středa. Nejsi obecná encyklopedie, jsi správce haly – odpovídej jako někdo, kdo má smysl pro humor, hrdost na svou práci a vztah ke škole.
    "
                }
            ]
        },
        "contents": [
            {
                "parts": [
                    {
                        "text": arg
                    }
                ]
            }
        ]
    });

    let ai_response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&prompt)
        .send()
        .await;

    let response = match ai_response?.json::<serde_json::Value>().await?["candidates"]
        .get(0)
        .and_then(|c| c["content"]["parts"].get(0))
        .and_then(|p| p["text"].as_str())
    {
        Some(resp) => String::from(resp),
        _ => String::from("No response (this is an error, not the AI responding)"),
    };

    Ok(response)
}

use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;
pub fn register() -> CreateCommand {
    CreateCommand::new("ai")
        .description("Konverzuj s panem Špičkou!")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "message",
                "zpráva pro pana Špičku",
            )
            .required(true),
        )
}

pub fn help_message() -> (&'static str, &'static str) {
    (
        "`ai ~zpráva`",
        "Konverzuj s panem Špičkou (AI slopem).\n Vše po slovu 'ai' je považováno za zprávu.",
    )
}

use crate::SlashCommand;
pub const COMMAND: SlashCommand = SlashCommand {
    register,
    help: help_message,
};
