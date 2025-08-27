use serenity::all::{Channel, Context, CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateMessage, Message, MessageUpdateEvent};

use crate::{constants::SOFT_YELLOW, event_handler::Handler, utils::guild_log};

pub async fn message_update(
    _handler: &Handler,
    ctx: Context,
    old_if_available: Option<Message>,
    new: Option<Message>,
    event: MessageUpdateEvent,
) {
    if event.edited_timestamp.is_none() {
        return;
    }

    let mut new_msg = match new {
        Some(m) if m.author.id != ctx.cache.current_user().id => m,
        Some(_) => return,
        None => match event.channel_id.message(&ctx.http, event.id).await {
            Ok(m) if m.author.id != ctx.cache.current_user().id => m,
            _ => return,
        },
    };

    if new_msg.content.is_empty() {
        new_msg.content = String::from("(no content)");
    }

    let base = format!(
        "**MESSAGE EDITED**\n-# Message {0} [jump](https://discord.com/channels/{3}/{0}/{2}) | Target: <@{1}> | Channel: <#{2}> ({2})",
        new_msg.id.get(),
        new_msg.author.id.get(),
        new_msg.channel_id.get(),
        new_msg.guild_id.map(|g| g.get()).unwrap_or(0)
    );

    let (desc, files) = match old_if_available {
        Some(mut old) => {
            if old.content.is_empty() {
                old.content = String::from("(no content)");
            }

            if old.content.len() > 500 || new_msg.content.len() > 500 {
                (
                    base.clone(),
                    vec![
                        CreateAttachment::bytes(new_msg.content.as_bytes(), "new.txt"),
                        CreateAttachment::bytes(old.content.as_bytes(), "old.txt"),
                    ]
                )
            } else {
                (
                    format!(
                        "{base}\nBefore:```\n{}\n```\nAfter:\n```\n{}\n```",
                        old.content.replace("```", "\\`\\`\\`"),
                        new_msg.content.replace("```", "\\`\\`\\`"),
                    ),
                    vec![]
                )
            }
        }
        None => {
            if new_msg.content.len() > 500 {
                (
                    format!("{base}\nMessage content not found in cache"),
                    vec![CreateAttachment::bytes(new_msg.content.as_bytes(), "new.txt")]
                )
            } else {
                (
                    format!(
                        "{base}\nBefore:```\nMessage content not found in cache\n```\nAfter:\n```\n{}\n```",
                        new_msg.content.replace("```", "\\`\\`\\`"),
                    ),
                    vec![]
                )
            }
        }
    };

    let mut message = CreateMessage::new().add_embed(
        CreateEmbed::new()
            .color(SOFT_YELLOW.clone())
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
