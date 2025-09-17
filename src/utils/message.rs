use serenity::all::{
    Context, CreateAllowedMentions, CreateEmbed, CreateEmbedFooter, CreateMessage, Message, User,
};
use tracing::warn;

use crate::constants::BRAND_BLUE;

pub async fn message_and_dm(
    ctx: &Context,
    command_msg: &Message,
    dm_user: &User,
    server_msg: String,
    dm_msg: String,
    log_id: Option<String>,
) {
    let dm =
        CreateMessage::new().add_embed(CreateEmbed::new().description(dm_msg).color(BRAND_BLUE));

    let mut footer = if let Some(id) = log_id {
        format!("Log ID: {id}")
    } else {
        String::new()
    };

    if dm_user.direct_message(&ctx.http, dm).await.is_err() {
        footer.push_str(" | DM failed. Target DMs off.");
    }

    let mut embed = CreateEmbed::new().description(server_msg).color(BRAND_BLUE);

    if !footer.is_empty() {
        embed = embed.footer(CreateEmbedFooter::new(footer));
    }

    let reply = CreateMessage::new()
        .add_embed(embed)
        .reference_message(command_msg)
        .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

    if let Err(err) = command_msg.channel_id.send_message(&ctx.http, reply).await {
        warn!("Could not send message; err = {err:?}");
    }
}
