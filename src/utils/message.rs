use serenity::all::{
    Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Message, User,
};
use tracing::warn;

use crate::constants::BRAND_BLUE;

pub async fn message_and_dm(
    ctx: &Context,
    command_msg: &Message,
    dm_user: &User,
    server_msg: impl Fn(String) -> String,
    dm_msg: String,
) {
    let dm =
        CreateMessage::new().add_embed(CreateEmbed::new().description(dm_msg).color(BRAND_BLUE));

    let mut addition = String::new();

    if dm_user.direct_message(&ctx.http, dm).await.is_err() {
        addition = String::from(" | DM failed; Target has DMs off.");
    }

    let embed = CreateEmbed::new().description(server_msg(addition)).color(BRAND_BLUE);

    let reply = CreateMessage::new()
        .add_embed(embed)
        .reference_message(command_msg)
        .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

    if let Err(err) = command_msg.channel_id.send_message(&ctx.http, reply).await {
        warn!("Could not send message; err = {err:?}");
    }
}
