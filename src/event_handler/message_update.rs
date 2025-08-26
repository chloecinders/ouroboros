use serenity::all::{Channel, Context, CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateMessage, Message, MessageUpdateEvent};

use crate::{constants::{BRAND_BLUE, SOFT_YELLOW}, event_handler::Handler, utils::guild_log};

pub async fn message_update(
    _handler: &Handler,
    ctx: Context,
    old_if_available: Option<Message>,
    new: Option<Message>,
    event: MessageUpdateEvent,
) {
    let new_msg = match new {
        Some(m) if m.author.id != ctx.cache.current_user().id => m,
        Some(_) => return,
        None => match event.channel_id.message(&ctx.http, event.id).await {
            Ok(m) if m.author.id != ctx.cache.current_user().id => m,
            _ => return,
        },
    };

    let base = format!(
        "**MESSAGE EDITED**\n-# {0} | Target: {1} | <#{2}> ({2})",
        new_msg.id.get(),
        new_msg.author.id.get(),
        new_msg.channel_id.get()
    );

    let (desc, files, color) = match old_if_available {
        Some(old) => {
            if old.content.len() > 500 || new_msg.content.len() > 500 {
                (
                    base.clone(),
                    vec![
                        CreateAttachment::bytes(new_msg.content.as_bytes(), "new.txt"),
                        CreateAttachment::bytes(old.content.as_bytes(), "old.txt"),
                    ],
                    BRAND_BLUE.clone(),
                )
            } else {
                (
                    format!(
                        "{base}\nBefore:```\n{}\n```\nAfter:\n```\n{}\n```",
                        old.content.replace("```", "\\`\\`\\`"),
                        new_msg.content.replace("```", "\\`\\`\\`"),
                    ),
                    vec![],
                    SOFT_YELLOW.clone(),
                )
            }
        }
        None => {
            if new_msg.content.len() > 500 {
                (
                    format!("{base}\nMessage content not found in cache"),
                    vec![CreateAttachment::bytes(new_msg.content.as_bytes(), "new.txt")],
                    SOFT_YELLOW.clone(),
                )
            } else {
                (
                    format!(
                        "{base}\nBefore:```\nMessage content not found in cache\n```\nAfter:\n```\n{}\n```",
                        new_msg.content.replace("```", "\\`\\`\\`"),
                    ),
                    vec![],
                    SOFT_YELLOW.clone(),
                )
            }
        }
    };

    let mut message = CreateMessage::new().add_embed(
        CreateEmbed::new()
            .color(color)
            .description(desc)
            .author(
                CreateEmbedAuthor::new(format!("{}: {}", new_msg.author.name, new_msg.author.id.get()))
                    .icon_url(new_msg.author.avatar_url().unwrap_or(new_msg.author.default_avatar_url()))
            )
    );

    for f in files {
        message = message.add_file(f);
    }

    let guild_id = match new_msg.channel_id.to_channel(&ctx.http).await {
        Ok(Channel::Guild(g)) => g.guild_id.get(),
        _ => new_msg.guild_id.map(|g| g.get()).unwrap_or(1),
    };

    guild_log(&ctx.http, guild_id.into(), message).await;
}
