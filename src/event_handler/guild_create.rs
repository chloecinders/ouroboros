use serenity::all::{Context, Guild};
use sqlx::query;
use tracing::error;

use crate::{event_handler::Handler, BOT_CONFIG, GUILD_SETTINGS, SQL};

pub async fn guild_create(_handler: &Handler, ctx: Context, guild: Guild, is_new: Option<bool>) {
    if let Some(new) = is_new && new {
        let cfg = BOT_CONFIG.get().unwrap();

        if cfg.whitelist_enabled.map_or(true, |b| !b) {
            return;
        }

        if cfg.whitelist.as_ref().map_or(true, |ids| !ids.contains(&guild.id.get())) {
            if let Err(err) = ctx.http.leave_guild(guild.id).await {
                error!("Could not leave non-whitelisted guild! err = {err:?}; id = {}", guild.id.get());
            }
        }

        if let Err(err) = query!(
            "INSERT INTO actions (guild_id) values ($1);",
            guild.id.get() as i64
        ).execute(SQL.get().unwrap()).await {
            error!("Got error during guild join settings set; guild = {} err = {}", guild.id.get(), err);
        }

        {
            let mut global = GUILD_SETTINGS.get().unwrap().lock().await;
            global.invalidate();
        }
    }
}
