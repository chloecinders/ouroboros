use serenity::all::{Context, GuildId, Role, RoleId};

use crate::event_handler::Handler;

pub async fn guild_role_delete(
    handler: &Handler,
    _ctx: Context,
    guild_id: GuildId,
    _removed_role_id: RoleId,
    _removed_role_data_if_available: Option<Role>,
) {
    {
        let mut permission_lock = handler.permission_cache.lock().await;
        permission_lock.invalidate_guild(guild_id.get()).await;
    }
}
