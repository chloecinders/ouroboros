use serenity::all::Context;
use sqlx::query;
use tracing::{error, info};

use crate::{event_handler::Handler, BOT_CONFIG, GUILD_SETTINGS, SQL};

pub async fn shards_ready(_handler: &Handler, ctx: Context, _total_shards: u32) {
    let cfg = BOT_CONFIG.get().unwrap();

    finish_update(&ctx).await;

    info!("Adding missing guilds to guild_settings");
    let guild_ids: Vec<String> = ctx.cache.guilds().iter().map(|g| format!("({})", g.get())).collect();

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

    if cfg.whitelist_enabled.map_or(true, |b| !b) {
        return;
    }

    for guild in ctx.cache.guilds() {
        if cfg.whitelist.as_ref().map_or(true, |ids| !ids.contains(&guild.get())) {
            if let Err(err) = ctx.http.leave_guild(guild).await {
                error!("Could not leave non-whitelisted guild! err = {err:?}; id = {}", guild.get());
            }
        }
    }
}

pub async fn finish_update(ctx: &Context) {
    if let Some(arg) = std::env::args().collect::<Vec<String>>().iter().find(|a| a.starts_with("--id")) {
        let Some(ids) = arg.split("=").last() else {
            return;
        };

        let mut parts = ids.split(':');

        let (channel_id, msg_id) = match (parts.next(), parts.next()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                return;
            }
        };

        let Ok(channel) = ctx.http.get_channel(channel_id.parse::<u64>().unwrap().into()).await else {
            return;
        };

        let Ok(message) = channel.id().message(ctx, msg_id.parse::<u64>().unwrap()).await else {
            return;
        };

        info!("Replying to update command; channel = {channel:?} message = {message:?}");

        let _ = message.reply(ctx, "Update finished!").await;
    }
}
