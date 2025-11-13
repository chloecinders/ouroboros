use std::fs;

use serenity::{all::{Context, Permissions, RoleId}, futures::StreamExt};
use sqlx::query;
use tracing::{error, info};

use crate::{BOT_CONFIG, GUILD_SETTINGS, SQL, config::Environment, event_handler::Handler, utils::cache::permission_cache::CommandPermissionRequest};

pub async fn shards_ready(handler: &Handler, ctx: Context, _total_shards: u32) {
    let cfg = BOT_CONFIG.get().unwrap();

    finish_update(&ctx).await;
    check_whitelist(cfg, &ctx).await;
    update_guild_settings(&ctx).await;
    fill_message_cache(handler, &ctx).await;
    fill_permission_cache(handler, &ctx).await;
}

pub async fn finish_update(ctx: &Context) {
    let ids = {
        if let Some(arg) = std::env::args()
            .collect::<Vec<String>>()
            .iter()
            .find(|a| a.starts_with("--id"))
        {
            let Some(ids) = arg.split("=").last() else {
                return;
            };

            ids.to_string()
        } else if let Ok(ids) = fs::read_to_string("./update.txt") {
            let _ = fs::remove_file("./update.txt");
            ids
        } else {
            return;
        }
    };

    let mut parts = ids.split(':');

    let (channel_id, msg_id) = match (parts.next(), parts.next()) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            return;
        }
    };

    let Ok(channel) = ctx
        .http
        .get_channel(channel_id.parse::<u64>().unwrap().into())
        .await
    else {
        return;
    };

    let Ok(message) = channel
        .id()
        .message(ctx, msg_id.parse::<u64>().unwrap())
        .await
    else {
        return;
    };

    info!("Replying to update command; channel = {channel:?} message = {message:?}");

    let _ = message.reply(ctx, "Update finished!").await;
}

pub async fn update_guild_settings(ctx: &Context) {
    info!("Adding missing guilds to guild_settings");
    let guild_ids: Vec<String> = ctx
        .cache
        .guilds()
        .iter()
        .map(|g| format!("({})", g.get()))
        .collect();

    let query = format!(
        r#"INSERT INTO guild_settings (guild_id) VALUES {} ON CONFLICT (guild_id) DO NOTHING;"#,
        guild_ids.join(", ")
    );

    if let Err(err) = sqlx::query(&query).execute(SQL.get().unwrap()).await {
        error!("Couldnt add missing guilds to guild_settings; err = {err:?}")
    }

    {
        let mut settings = GUILD_SETTINGS.get().unwrap().lock().await;
        settings.invalidate();
    }
}

pub async fn check_whitelist(cfg: &Environment, ctx: &Context) {
    if cfg.whitelist_enabled.is_none_or(|b| !b) {
        return;
    }

    for guild in ctx.cache.guilds() {
        if cfg
            .whitelist
            .as_ref()
            .is_none_or(|ids| !ids.contains(&guild.get()))
            && let Err(err) = ctx.http.leave_guild(guild).await
        {
            error!(
                "Could not leave non-whitelisted guild! err = {err:?}; id = {}",
                guild.get()
            );
        }
    }
}

pub async fn fill_message_cache(handler: &Handler, ctx: &Context) {
    let existing_data = match query!("SELECT * FROM message_cache_store")
        .fetch_all(SQL.get().unwrap())
        .await
    {
        Ok(r) => r,
        Err(err) => {
            error!("Couldnt fetch latest message cache counts; err = {err:?}");
            return;
        }
    };

    let mut lock = handler.message_cache.lock().await;

    for guild in ctx.cache.guilds() {
        let Some(cached) = guild.to_guild_cached(&ctx.cache) else {
            continue;
        };

        for id in cached.channels.keys() {
            lock.assign_count(id.get(), 100);
        }
    }

    for record in existing_data {
        lock.assign_count(record.channel_id as u64, record.message_count as usize);
    }
}

pub async fn fill_permission_cache(handler: &Handler, ctx: &Context) {
    for guild in ctx.cache.guilds() {
        let Ok(partial) = guild.to_partial_guild(&ctx).await else {
            continue;
        };

        let mut valid_roles: Vec<RoleId> = Vec::new();

        for (id, role) in &partial.roles {
            if role.has_permission(Permissions::MANAGE_MESSAGES)
                || role.has_permission(Permissions::BAN_MEMBERS)
                || role.has_permission(Permissions::KICK_MEMBERS)
                || role.has_permission(Permissions::ADMINISTRATOR) {
                valid_roles.push(id.clone());
            }
        }

        let mut members = guild.members_iter(&ctx).boxed();

        while let Some(member_result) = members.next().await {
            let Ok(member) = member_result else {
                continue;
            };

            if member.roles.iter().any(|r| valid_roles.contains(&r)) {
                let mut cache = handler.permission_cache.lock().await;
                cache.can_run(CommandPermissionRequest {
                    command: handler.commands.iter().find(|c| c.get_name() == "ban").cloned().unwrap(),
                    member,
                    guild: partial.clone(),
                    handler: handler.clone(),
                }).await;
            }
        }
    }
}
