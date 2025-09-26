use std::time::Duration;

use serenity::all::{Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Message, User};
use tokio::time::sleep;
use tracing::warn;

use crate::constants::BRAND_BLUE;

pub async fn message_and_dm(
    ctx: &Context,
    command_msg: &Message,
    dm_user: &User,
    server_msg: impl Fn(String) -> String,
    dm_msg: String,
    automatically_delete: bool,
) {
    let dm =
        CreateMessage::new().add_embed(CreateEmbed::new().description(dm_msg).color(BRAND_BLUE));

    let mut addition = String::new();

    if dm_user.direct_message(&ctx.http, dm).await.is_err() {
        addition = String::from(" | DM failed; Target has DMs off.");
    }

    let embed = CreateEmbed::new()
        .description(server_msg(addition))
        .color(BRAND_BLUE);

    let reply = CreateMessage::new()
        .add_embed(embed)
        .reference_message(command_msg)
        .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

    let msg = match command_msg.channel_id.send_message(&ctx.http, reply).await {
        Ok(m) => m,
        Err(err) => {
            warn!("Could not send message; err = {err:?}");
            return;
        }
    };

    if automatically_delete {
        let http = ctx.http.clone();
        let cmd_msg = command_msg.clone();

        tokio::spawn(async move {
            sleep(Duration::from_secs(5)).await;
            let _ = msg.delete(&http).await;
            let _ = cmd_msg.delete(&http).await;
        });
    }
}
