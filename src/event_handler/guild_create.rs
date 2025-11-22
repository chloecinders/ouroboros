use serenity::all::{Context, Guild};
use sqlx::query;
use tracing::error;

use crate::{BOT_CONFIG, GUILD_SETTINGS, SQL, event_handler::Handler};

pub async fn guild_create(_handler: &Handler, ctx: Context, guild: Guild, is_new: Option<bool>) {
    if let Some(new) = is_new
        && new
    {
        if BOT_CONFIG.whitelist_enabled.is_none_or(|b| !b) {
            return;
        }

        if BOT_CONFIG
            .whitelist
            .as_ref()
            .is_none_or(|ids| !ids.contains(&guild.id.get()))
            && let Err(err) = ctx.http.leave_guild(guild.id).await
        {
            error!(
                "Could not leave non-whitelisted guild! err = {err:?}; id = {}",
                guild.id.get()
            );
        }

        if let Err(err) = query!(
            "INSERT INTO actions (guild_id) values ($1);",
            guild.id.get() as i64
        )
        .execute(&*SQL)
        .await
        {
            error!(
                "Got error during guild join settings set; guild = {} err = {}",
                guild.id.get(),
                err
            );
        }

        {
            let mut global = GUILD_SETTINGS.lock().await;
            global.invalidate();
        }
    }
}
