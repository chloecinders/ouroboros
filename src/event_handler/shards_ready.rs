use serenity::all::Context;
use tracing::error;

use crate::{event_handler::Handler, BOT_CONFIG};

pub async fn shards_ready(_handler: &Handler, ctx: Context, _total_shards: u32) {
    let cfg = BOT_CONFIG.get().unwrap();

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
