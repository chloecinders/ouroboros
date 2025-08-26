use chrono::Utc;
use serenity::all::{audit_log::Action, Channel, ChannelId, Context, CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateMessage, GuildId, MessageAction, MessageId};

use crate::{constants::BRAND_RED, event_handler::Handler, utils::{guild_log, snowflake_to_timestamp}};

pub async fn message_delete(_handler: &Handler, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, _guild_id: Option<GuildId>) { 
    let some_msg = {
        if let Some(msg) = ctx.cache.message(channel_id, deleted_message_id) {
            if msg.author.id.get() == ctx.cache.current_user().id.get() {
                return;
            }

            Some(msg.clone())
        } else {
            None
        }
    };

    let guild_id = match channel_id.to_channel(&ctx.http).await {
        Ok(Channel::Guild(guild_channel)) => guild_channel.guild_id,
        _ => {return}
    };

    let audit_log = {
        match guild_id.audit_logs(&ctx.http, Some(Action::Message(MessageAction::Delete)), None, None, Some(10)).await {
            Ok(l) => Some(l),
            Err(_) => None,
        }
    };

    let mut moderator_id: Option<u64> = None;

    if let Some(logs) = audit_log {
        if let Some(entry) = logs.entries.first() {
            if let Some(target) = entry.target_id
                && let Some(Some(channel)) = entry.options.clone().map(|o| o.channel_id)
                && let Some(msg) = some_msg.clone()
            {
                if target.get() == msg.author.id.get() && channel.get() == msg.channel_id.get() {
                    moderator_id = Some(entry.user_id.get());
                }
            }
        } else {
            for entry in logs.entries {
                let entry_time = snowflake_to_timestamp(entry.id.get());

                if
                    (Utc::now() - entry_time).num_seconds().abs() <= 5
                    && let Some(target) = entry.target_id
                    && let Some(Some(channel)) = entry.options.clone().map(|o| o.channel_id)
                    && let Some(msg) = some_msg.clone()
                {
                    if target.get() == msg.author.id.get() && channel.get() == msg.channel_id.get() {
                        moderator_id = Some(entry.user_id.get());
                    }
                }
            }
        }

    }

    let mut description = format!(
        "**MESSAGE DELETED**\n-# {0} ",
        deleted_message_id.get()
    );
    let mut files = vec![];
    let mut embed = CreateEmbed::new()
            .color(BRAND_RED.clone());

    if let Some(msg) = some_msg.clone() {
        description.push_str(&format!("| Target: <@{}> ", msg.author.id.get()));
        embed = embed.author(
            CreateEmbedAuthor::new(format!("{}: {}", msg.author.name, msg.author.id.get()))
                .icon_url(msg.author.avatar_url().unwrap_or(msg.author.default_avatar_url()))
        );

        for attachment in msg.attachments.iter() {
            let name = attachment.filename.clone();
            if let Ok(bytes) = attachment.download().await {
                files.push(CreateAttachment::bytes(bytes, name));
            };
        }
    }

    description.push_str(&format!("| <#{0}> ({0}) ", channel_id.get()));

    if let Some(moderator) = moderator_id {
        description.push_str(&format!("| User: <@{0}> ({0}) ", moderator));
    }

    if let Some(msg) = some_msg {
        description.push_str(&format!(
            "\n{}",
            if msg.content.is_empty() { String::new() } else { format!("```\n{}\n```", msg.content) }
        ));
    } else {
        description.push_str(
            "\nContent not found in cache",
        );
    }

    guild_log(
        &ctx.http,
        guild_id,
        CreateMessage::new().add_embed(
            embed.description(description)
        ).add_files(files)
    ).await;
}
