use serenity::all::{Channel, Context, CreateAttachment, CreateEmbed, CreateMessage, Message, MessageUpdateEvent};

use crate::{constants::{BRAND_BLUE, SOFT_YELLOW}, event_handler::Handler, utils::guild_log};

pub async fn message_update(_handler: &Handler, ctx: Context, old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent) {
    let new_msg = {
        match new {
            Some(m) => m,
            None => {
                let Ok(m) = event.channel_id.message(&ctx.http, event.id).await else {
                    return;
                };
                m
            }
        }
    };

    match () {
        () if old_if_available.is_some() => {
            let old = old_if_available.unwrap();
            let mut message = CreateMessage::new();

            if old.content.len() > 500 || new_msg.content.len() > 500 {
                message = message.add_embed(
                    CreateEmbed::new()
                        .color(BRAND_BLUE.clone())
                        .description(format!(
                            "<@{0}> ({1}: {0}) edited a message ({2}) in <#{3}> ({3})",
                            new_msg.author.id,
                            new_msg.author.name,
                            new_msg.id.get(),
                            new_msg.channel_id.get(),
                        ))
                );

                message = message.add_file(
                    CreateAttachment::bytes(new_msg.content.as_bytes(), "new.txt")
                );

                message = message.add_file(
                    CreateAttachment::bytes(old.content.as_bytes(), "old.txt")
                );
            } else {
                message = message.add_embed(
                    CreateEmbed::new()
                        .color(SOFT_YELLOW.clone())
                        .description(format!(
                            "Message ({2}) edited in <#{3}> ({3}) by <@{0}> ({1}: {0})\n\nBefore:```\n{4}\n```\nAfter:\n```\n{5}\n```",
                            new_msg.author.id,
                            new_msg.author.name,
                            new_msg.id.get(),
                            new_msg.channel_id.get(),
                            old.content.replace("```", "\\`\\`\\`"),
                            new_msg.content.replace("```", "\\`\\`\\`")
                        ))
                );
            }

            guild_log(&ctx.http, new_msg.guild_id.unwrap_or(1.into()), message).await;
        },
        _ => {
            let mut message = CreateMessage::new();

            if new_msg.content.len() > 500 {
                message = message.add_embed(
                    CreateEmbed::new()
                        .color(SOFT_YELLOW.clone())
                        .description(format!(
                            "Message ({2}) edited in <#{3}> ({3}) by <@{0}> ({1}: {0})\n\nMessage content not found in cache",
                            new_msg.author.id,
                            new_msg.author.name,
                            new_msg.id.get(),
                            new_msg.channel_id.get()
                        ))
                );

                message = message.add_file(
                    CreateAttachment::bytes(new_msg.content.as_bytes(), "new.txt")
                );
            } else {
                message = message.add_embed(
                    CreateEmbed::new()
                        .color(SOFT_YELLOW.clone())
                        .description(format!(
                            "Message ({2}) edited in <#{3}> ({3}) by <@{0}> ({1}: {0})\n\nBefore:```\nnMessage content not found in cache\n```\nAfter:\n```\n{4}\n```",
                            new_msg.author.id,
                            new_msg.author.name,
                            new_msg.id.get(),
                            new_msg.channel_id.get(),
                            new_msg.content.replace("```", "\\`\\`\\`")
                        ))
                );
            }

            let guild_id = match new_msg.channel_id.to_channel(&ctx.http).await {
                Ok(Channel::Guild(guild_channel)) => guild_channel.guild_id.get(),
                _ => new_msg.guild_id.map(|g| g.get()).unwrap_or(1)
            };

            guild_log(&ctx.http, guild_id.into(), message).await;
        }
    };
}
