use serenity::all::{Context, Guild, PartialGuild};

use crate::event_handler::Handler;

pub async fn guild_update(
    handler: &Handler,
    _ctx: Context,
    _old_data_if_available: Option<Guild>,
    new_data: PartialGuild,
) {
    {
        let mut permission_lock = handler.permission_cache.lock().await;
        permission_lock.invalidate_guild(new_data.id.get()).await;
    }
}
