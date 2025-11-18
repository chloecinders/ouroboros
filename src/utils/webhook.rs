use chrono::Local;
use serenity::{all::{CreateEmbed, CreateMessage}, json};
use tracing::warn;

use crate::{BOT_CONFIG, constants::BRAND_RED};

pub fn send_error(title: String, body: String) {
    let config = BOT_CONFIG.get().unwrap();
    let Some(webhook_url) = config.webhook.clone() else {
        return;
    };

    let time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    tokio::spawn(async move {
        let msg = CreateMessage::new()
            .embed(
                CreateEmbed::new()
                    .color(BRAND_RED)
                    .description(format!("**{title}**\n`{time}` {body}"))
            );

        let body = json::to_string(&msg);
        let err = reqwest::Client::new()
            .post(webhook_url)
            .body(body.unwrap_or_default())
            .header("content-type", "application/json")
            .send()
            .await;

        match err {
            Ok(body) => warn!("Sent error; response = {}", body.text().await.unwrap_or_default()),
            Err(e) => warn!("Error while sending error... {e:?}"),
        }
    });
}
