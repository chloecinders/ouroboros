use std::io::Cursor;

use chrono::Utc;
use image::{DynamicImage, GenericImage, imageops::FilterType};
use reqwest::Client;
use serenity::all::{
    Change, Context, CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateMessage,
    GuildMemberUpdateEvent, Member, MemberAction, audit_log::Action,
};
use tracing::warn;

use crate::{
    GUILD_SETTINGS,
    constants::BRAND_BLUE,
    event_handler::Handler,
    utils::{guild_log, snowflake_to_timestamp},
};

pub async fn guild_member_update(
    _handler: &Handler,
    ctx: Context,
    old_if_available: Option<Member>,
    new: Option<Member>,
    event: GuildMemberUpdateEvent,
) {
    {
        let mut settings = GUILD_SETTINGS.get().unwrap().lock().await;
        let Ok(guild_settings) = settings.get(event.guild_id.get()).await else {
            warn!(
                "Found guild with no cached settings; Id = {}",
                event.guild_id.get()
            );
            return;
        };

        if event.user.bot && guild_settings.log.log_bots.is_none_or(|b| !b) {
            return;
        }
    }

    let audit_log = event
        .guild_id
        .audit_logs(
            &ctx.http,
            Some(Action::Member(MemberAction::Update)),
            None,
            None,
            Some(10),
        )
        .await
        .ok();

    let mut moderator_id: Option<u64> = None;
    let mut reason: Option<String> = None;
    let mut old_nick: Option<Option<String>> = old_if_available.clone().map(|o| o.nick);

    if let Some(logs) = audit_log {
        'o: for entry in logs.entries {
            for change in entry.changes.unwrap_or(Vec::new()) {
                let entry_time = snowflake_to_timestamp(entry.id.get());

                if let Change::Nick { old, new } = change
                    && event.user.id.get() == entry.user_id.get()
                    && new == event.nick
                    && (Utc::now() - entry_time).num_seconds().abs() <= 300
                {
                    if old_if_available.clone().is_some_and(|old_user| {
                        old.clone()
                            .is_some_and(|old_nick| old_user.display_name() == old_nick)
                    }) {
                        continue;
                    }

                    moderator_id = Some(entry.user_id.get());
                    reason = entry.reason.clone();

                    if Some(old.clone()) != old_nick {
                        old_nick = Some(old);
                    }
                    break 'o;
                }
            }
        }
    }

    let name = if let Some(old) = old_nick {
        if old == event.nick {
            String::new()
        } else {
            format!(
                "\nName:\n`{}` -> `{}`",
                old.unwrap_or(String::from("(none)")),
                event.nick.unwrap_or(String::from("(none)"))
            )
        }
    } else {
        String::new()
    };

    let avatar = if let Some(old) = old_if_available
        && let Some(new) = new
    {
        if old.avatar.or(old.user.avatar) == new.avatar.or(new.user.avatar) {
            (String::new(), None)
        } else {
            let client = Client::new();

            if let (Some(old_image), Some(new_image)) = (
                get_member_avatar_image(&client, old).await,
                get_member_avatar_image(&client, new).await,
            ) {
                let target_height = old_image.height();

                let old_image = old_image.resize(
                    old_image.width() * target_height / old_image.height(),
                    target_height,
                    FilterType::Lanczos3,
                );

                let new_image = new_image.resize(
                    new_image.width() * target_height / new_image.height(),
                    target_height,
                    FilterType::Lanczos3,
                );

                let total_width = target_height * 2;
                let mut output = DynamicImage::new_rgba8(total_width, target_height);

                output.copy_from(&old_image, 0, 0).unwrap();
                output.copy_from(&new_image, new_image.width(), 0).unwrap();

                let mut buff = Vec::new();
                if output
                    .write_to(&mut Cursor::new(&mut buff), image::ImageFormat::WebP)
                    .is_err()
                {
                    (String::new(), None)
                } else {
                    (String::from("\nAvatar:\n"), Some(buff))
                }
            } else {
                (String::new(), None)
            }
        }
    } else {
        (String::new(), None)
    };

    if name.is_empty() && avatar.0.is_empty() {
        return;
    }

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

    let description = format!(
        "**MEMBER UPDATE**\n-# <@{}>{}{}{}{}",
        event.user.id, moderator, name, reason, avatar.0
    );
    let mut embed = CreateEmbed::new()
        .color(BRAND_BLUE)
        .description(description)
        .author(
            CreateEmbedAuthor::new(format!("{}: {}", event.user.name, event.user.id.get()))
                .icon_url(
                    event
                        .user
                        .avatar_url()
                        .unwrap_or(event.user.default_avatar_url()),
                ),
        );

    if !avatar.0.is_empty() {
        embed = embed.image("attachment://avatar.webp");
    }

    let mut msg = CreateMessage::new().add_embed(embed);

    if !avatar.0.is_empty() {
        msg = msg.add_file(CreateAttachment::bytes(avatar.1.unwrap(), "avatar.webp"));
    }

    guild_log(&ctx.http, event.guild_id, msg).await;
}

async fn get_member_avatar_image(client: &Client, member: Member) -> Option<image::DynamicImage> {
    let avatar_req = client
        .get(
            member.avatar_url().unwrap_or(
                member
                    .user
                    .avatar_url()
                    .unwrap_or(member.user.default_avatar_url()),
            ),
        )
        .send()
        .await
        .ok()?;
    let bytes = avatar_req.bytes().await.ok()?;
    image::load_from_memory(&bytes).ok()
}
