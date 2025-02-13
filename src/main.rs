#[macro_use]
extern crate rocket;
use color_eyre::Result;
use lazy_static::lazy_static;
use reqwest;
use rocket::serde::{json::Json, Deserialize};

lazy_static! {
    static ref TELEGRAM_BOT_TOKEN: String =
        std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN is not set");
}

async fn telegram_send_message(user_id: i64, message_text: &str, parse_mode: &str) -> Result<()> {
    #[derive(serde::Serialize)]
    struct TelegramSendMessage {
        chat_id: i64,
        text: String,
        parse_mode: String,
    }

    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        *TELEGRAM_BOT_TOKEN
    );

    let response = reqwest::Client::new()
        .post(&url)
        .json(&TelegramSendMessage {
            chat_id: user_id,
            text: message_text.to_string(),
            parse_mode: parse_mode.to_string(),
        })
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => Ok(()),
        _ => Err(color_eyre::eyre::eyre!("Failed to send message")),
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct MessageData<'r> {
    message: &'r str,
    #[serde(default = "default_parse_mode")]
    parse_mode: &'r str,
    user_list: Vec<i64>,
}

fn default_parse_mode() -> &'static str {
    "MarkdownV2"
}

#[derive(rocket::serde::Serialize)]
struct SendMessageResponse {
    successful: Vec<i64>,
    errors: Vec<i64>,
}

#[post("/send_message", format = "json", data = "<message_data>")]
async fn send_message(message_data: Json<MessageData<'_>>) -> Json<SendMessageResponse>{
    let message = message_data.message;
    let parse_mode = message_data.parse_mode;
    let user_list = message_data.user_list.clone();

    let mut successful: Vec<i64> = Vec::new();
    let mut errors: Vec<i64> = Vec::new();

    for user_id in user_list {
        match telegram_send_message(user_id, message, parse_mode).await {
            Ok(_) => {
                successful.push(user_id);
            },
            Err(_) => {
                errors.push(user_id);
            }
        }
    }
    Json::from(SendMessageResponse {
        successful,
        errors,
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![send_message])
}
