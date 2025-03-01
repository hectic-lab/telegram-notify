#[macro_use]
extern crate rocket;
use rocket::State;
use color_eyre::Result;
use reqwest;
use rocket::serde::{json::Json, Deserialize, Serialize};

struct TelegramBotToken(String);

async fn telegram_send_message(
    user_id: i64,
    message_text: &str,
    parse_mode: &str,
    token: &str,
) -> Result<()> {
    #[derive(serde::Serialize)]
    struct TelegramSendMessage {
        chat_id: i64,
        text: String,
        parse_mode: String,
    }

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);

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

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct SendMessageResponse {
    successful: Vec<i64>,
    errors: Vec<i64>,
}

#[post("/send_message", format = "json", data = "<message_data>")]
async fn send_message(
    message_data: Json<MessageData<'_>>,
    token: &State<TelegramBotToken>,
) -> Json<SendMessageResponse> {
    let mut successful = Vec::new();
    let mut errors = Vec::new();

    for user_id in message_data.user_list.clone() {
        match telegram_send_message(
            user_id,
            message_data.message,
            message_data.parse_mode,
            &token.0,
        )
        .await
        {
            Ok(_) => successful.push(user_id),
            Err(_) => errors.push(user_id),
        }
    }
    Json(SendMessageResponse { successful, errors })
}

#[launch]
fn rocket() -> _ {
    let telegram_bot_token =
        std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN is not set");
    let port: i32 = std::env::var("PORT")
    .unwrap_or_else(|_| "8000".to_string())
    .parse()
    .unwrap_or(8000);
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", port)))
        .manage(TelegramBotToken(telegram_bot_token))
        .mount("/", routes![send_message])
}
