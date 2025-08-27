use serenity::all::{Change, Context, CreateEmbed, CreateEmbedAuthor, CreateMessage, GuildMemberUpdateEvent, Member, MemberAction, audit_log::Action};

use crate::{constants::BRAND_BLUE, event_handler::Handler, utils::guild_log};

pub async fn guild_member_update(_handler: &Handler, ctx: Context, old_if_available: Option<Member>, _new: Option<Member>, event: GuildMemberUpdateEvent) {
    let audit_log = {
        match event.guild_id.audit_logs(&ctx.http, Some(Action::Member(MemberAction::Update)), None, None, Some(10)).await {
            Ok(l) => Some(l),
            Err(_) => None,
        }
    };

    let mut moderator_id: Option<u64> = None;
    let mut reason: Option<String> = None;
    let mut old_nick: Option<Option<String>> = old_if_available.map(|o| o.nick);

    if let Some(logs) = audit_log {
        'o: for entry in logs.entries {
            for change in entry.changes.unwrap_or(Vec::new()) {
                if let Change::Nick { old, new } = change {
                    if event.user.id.get() == entry.user_id.get() && new == event.nick {
                        moderator_id = Some(entry.user_id.get());
                        reason = entry.reason.clone();
                        old_nick = Some(old);
                        break 'o;
                    }
                }
            }
        }
    }

    let name = if let Some(old) = old_nick {
        format!("\nName:\n`{}` -> `{}`", old.unwrap_or(String::from("(none)")), event.nick.unwrap_or(String::from("(none)")))
    } else {
        format!("\nName:\n`{}`", event.nick.unwrap_or(String::from("(none)")))
    };

    let moderator = if let Some(id) = moderator_id {
        format!(" | Actor: <@{id}>")
    } else {
        String::new()
    };

    let reason = if let Some(reason) = reason {
        format!("\nReason:\n```{reason} ```")
    } else {
        String::new()
    };

    let description = format!("**MEMBER UPDATE**\n-# <@{}>{}{}{}", event.user.id, moderator, name, reason);
    let embed = CreateEmbed::new()
        .color(BRAND_BLUE.clone())
        .description(description)
        .author(
            CreateEmbedAuthor::new(format!("{}: {}", event.user.name, event.user.id.get()))
                .icon_url(event.user.avatar_url().unwrap_or(event.user.default_avatar_url()))
        );

    guild_log(&ctx.http, event.guild_id, CreateMessage::new().add_embed(embed)).await;
}
