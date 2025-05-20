// This file contains methods that interact with the "Na tácu" API
// Proudly copied from lunch bot on github https://github.com/jenyyk/lunch-bot

const API_URL: &str = "https://apiv2.natacu.cz/graphql";

use reqwest::blocking::Client;
use serde_json::{Value, json};
use serenity::{
    all::{Color, CreateEmbedAuthor, CreateEmbedFooter},
    builder::CreateEmbed,
};

pub fn fetch_food(time_delay: i64) -> Value {
    let canteen_id: u8 = dotenv::var("CANTEEN_ID")
        .unwrap_or("1".to_string())
        .parse()
        .unwrap_or(1);
    // Multiplied by 1000, because the API takes values in milliseconds, not seconds
    let timestamp: i64 = (chrono::Utc::now().timestamp() + time_delay) * 1000;
    // Parse the request body
    // We do this to modify the query variables
    let request_body_str = r#"{
        "operationName": "canteenOffersQuery",
        "variables": {
            "query": {
                "canteenId": 0,
                "from": "0",
                "to": "0",
                "order": "ASC"
            }
        },
        "query": "query canteenOffersQuery($query: GetOffersInput!) {\n  canteenOffers(query: $query) {\n    id\n    date\n    food {\n      id\n      name\n      averageRating\n      __typename\n    }\n    __typename\n  }\n}"
    }"#;
    let mut request_body: Value = serde_json::from_str(request_body_str).unwrap();

    // Modifying the request body
    request_body["variables"]["query"]["canteenId"] = json!(canteen_id);
    request_body["variables"]["query"]["from"] = json!(timestamp.to_string());
    request_body["variables"]["query"]["to"] = json!(timestamp.to_string());

    // Sending the request
    let client = Client::new();
    let food_response = client
        .post(API_URL)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_body).unwrap())
        .send()
        .unwrap();

    serde_json::from_str(&food_response.text().unwrap()).unwrap()
}

pub fn fetch_food_image(id: u32) -> String {
    // Parse the request body
    // We do this to modify the query variables
    let request_body_str = r#"{
        "operationName":"foodQuery",
        "variables":{
            "id":0
        },
        "query":"query foodQuery($id: Int!) {\n  food(id: $id) {\n    id\n    name\n    description\n    canteenId\n    averageRating\n    similarNames {\n      alternateName\n      __typename\n    }\n    photos {\n      id\n      s3url\n      __typename\n    }\n    __typename\n  }\n}"
    }"#;
    let mut request_body: Value = serde_json::from_str(request_body_str).unwrap();
    // Modifying the request body
    request_body["variables"]["id"] = json!(id);

    // Sending the request
    let client = Client::new();
    let image_response = client
        .post(API_URL)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_body).unwrap())
        .send()
        .unwrap();

    let serde_object: Value = serde_json::from_str(&image_response.text().unwrap()).unwrap();
    // JSON formatting magic to return the s3 url
    let photos = serde_object["data"]["food"]["photos"].as_array().unwrap();
    let first_photo = match photos.first() {
        Some(photo) => photo,
        _ => return "".to_string(),
    };
    first_photo["s3url"].to_string()
}

pub fn get_lunch_embed(days_forward: i64) -> Result<Vec<CreateEmbed>, String> {
    let lunch_response = fetch_food(days_forward * 86400_i64);
    // Must handle empty offers (weekends)
    let offer_array = match lunch_response["data"]["canteenOffers"]
        .as_array()
        .unwrap()
        .first()
    {
        Some(offer) => offer["food"].as_array().unwrap(),
        None => return Err(String::from("Failed getting lunches")),
    };

    let mut lunch_counter = 0;
    let mut embed_vec: Vec<CreateEmbed> = Vec::with_capacity(offer_array.len());
    for offer in offer_array {
        lunch_counter += 1;
        // Formats the image_url, as it can be missing for some foods
        let image_url = fetch_food_image(offer["id"].as_u64().unwrap_or(0) as u32);
        let mut trimmed_image_url: Option<String> = None;
        if !image_url.is_empty() {
            trimmed_image_url = Some(image_url[1..&image_url.len() - 1].to_string());
        }
        // Formats the date (gotta love the czech language)

        let date: String = match days_forward {
            0 => "Dnes".to_string(),
            1 => "Zítra".to_string(),
            2 => format!("Za {days_forward} dny").to_string(),
            3 => format!("Za {days_forward} dny").to_string(),
            4 => format!("Za {days_forward} dny").to_string(),
            _ => format!("Za {days_forward} dnů").to_string(),
        };
        let mut rating = offer["averageRating"].to_string();
        if rating == "null" {
            rating = String::from("Bez hodnocení")
        }

        let embed = CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(format!("Oběd {} · {}", lunch_counter, date))
                    .icon_url(
                        "https://www.gypce.cz/wp-content/uploads/2013/06/gypce-1.jpg",
                    ),
            )
            .description(format!("# {}", offer["name"].to_string().replace('"', "")))
            .thumbnail(trimmed_image_url.unwrap_or(String::from("")))
            .color(Color::from_rgb(255, 20, 140))
            .footer(CreateEmbedFooter::new(rating).icon_url("https://png.pngtree.com/png-vector/20230222/ourmid/pngtree-shiny-yellow-star-icon-clipart-png-image_6613580.png"));

        embed_vec.push(embed);
    }
    Ok(embed_vec)
}

use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;
pub fn register() -> CreateCommand {
    CreateCommand::new("obedy")
        .description("Zašle obědy v gypce jídelně v daný den")
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "days_forward",
            "Kolik dní dopředu oběd? 0 - dnes, 1 - zítra atd.",
        ))
}
pub fn help_message() -> (&'static str, &'static str) {
    (
        "`obedy ~dny_dopředu`",
        "Zašle obědy v gypce jídelně v daný den\n`~dny_dopředu` musí být kladné číslo, 0 - dnes, 1 - zítra atd.",
    )
}
