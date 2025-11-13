use serenity::all::{Context, Role};

use crate::event_handler::Handler;

pub async fn guild_role_update(
    handler: &Handler,
    _ctx: Context,
    _old_data_if_available: Option<Role>,
    new: Role,
) {
    {
        let mut permission_lock = handler.permission_cache.lock().await;
        permission_lock.invalidate_guild(new.guild_id.get()).await;
    }
}
