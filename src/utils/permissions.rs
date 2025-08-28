use serenity::all::{Context, Member, Permissions, User};

use crate::BOT_CONFIG;

pub async fn check_guild_permission(
    ctx: &Context,
    member: &Member,
    permission: Permissions,
) -> bool {
    if let Some(g) = member.guild_id.to_guild_cached(&ctx.cache)
        && g.owner_id.get() == member.user.id.get()
    {
        return true;
    }

    for role in member.roles.clone() {
        let Ok(role) = member.guild_id.role(&ctx.http, role).await else {
            return false;
        };

        if role.has_permission(permission) || role.has_permission(Permissions::ADMINISTRATOR) {
            return true;
        }
    }

    false
}

pub fn is_developer(user: &User) -> bool {
    let cfg = BOT_CONFIG.get().unwrap();
    cfg.dev_ids
        .clone()
        .is_some_and(|i| i.contains(&user.id.get()))
}
