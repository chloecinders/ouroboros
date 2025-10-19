use serenity::all::{
    Context, GuildChannel, Member, PartialGuild, PermissionOverwriteType, Permissions, User
};
use tracing::warn;

use crate::BOT_CONFIG;

/// Checks if a member has a permission in the guild. Ingnores channel overrides.
pub async fn check_guild_permission(ctx: &Context, member: &Member, permission: Permissions) -> bool {
    let guild = match member.guild_id.to_partial_guild(&ctx).await {
        Ok(guild) => guild,
        Err(err) => {
            warn!("Couldn't get PartialGuild from GuildId; err = {err:?}");
            return false;
        }
    };

    if guild.owner_id.get() == member.user.id.get() {
        return true;
    }

    for role in member.roles.clone() {
        let Some(role) = guild.roles.get(&role) else {
            return false;
        };

        if role.has_permission(permission) || role.has_permission(Permissions::ADMINISTRATOR) {
            return true;
        }
    }

    false
}

/// Checks if a member has a permission in a guilds channel. Respects channel overrides.
pub async fn check_channel_permission(
    ctx: &Context,
    channel: &GuildChannel,
    member: &Member,
    permission: Permissions,
) -> bool {
    match member.guild_id.to_partial_guild(&ctx).await {
        Ok(guild) => {
            if guild.owner_id.get() == member.user.id.get() {
                return true;
            }
        },
        Err(err) => {
            warn!("Couldn't get PartialGuild from GuildId; err = {err:?}");
        }
    };

    #[allow(deprecated)] // Serenity has no equivalent not-deprecated function...
    if let Ok(perms) = member.permissions(&ctx.cache) && perms.contains(Permissions::ADMINISTRATOR) {
        return true;
    }

    let channel_perms = permissions_for_channel(ctx, channel, member).await;
    channel_perms.contains(Permissions::ADMINISTRATOR) || channel_perms.contains(permission) // another admin check since the above can fail
}

/// Gets all the permissions of a member in a guild.
pub fn permissions_for_guild(guild: PartialGuild, member: &Member) -> Permissions {
    let everyone = guild.roles.iter().find(|r| r.1.position == 0).unwrap();
    let mut roles = member
        .roles
        .iter()
        .map(|r| guild.roles.get(r).unwrap())
        .collect::<Vec<_>>();
    roles.push(everyone.1);
    roles.sort();

    let mut base = Permissions::empty();

    for role in roles {
        role.permissions.into_iter().for_each(|p| {
            base.insert(p);
        });
    }

    base
}

/// Gets all the permissions of a member in a guild channel, including channel overrides.
pub async fn permissions_for_channel(
    ctx: &Context,
    channel: &GuildChannel,
    member: &Member,
) -> Permissions {
    let guild = match channel.guild_id.to_partial_guild(&ctx).await {
        Ok(g) => g,
        Err(err) => {
            warn!("Couldn't get PartialGuild during permissions check; err = {err:?}");
            return Permissions::empty();
        }
    };
    let mut permissions = permissions_for_guild(guild.to_owned(), member);
    let everyone = guild.roles.iter().find(|r| r.1.position == 0).unwrap();
    let mut roles = member
        .roles
        .iter()
        .map(|r| guild.roles.get(r).unwrap())
        .collect::<Vec<_>>();
    roles.push(everyone.1);
    roles.sort();

    let overwrites = channel.permission_overwrites.clone();

    for role in roles {
        if let Some(overwrite) = overwrites.iter().find(|p| {
            if let PermissionOverwriteType::Role(id) = p.kind
                && id == role.id
            {
                true
            } else {
                false
            }
        }) {
            for perm in overwrite.allow {
                permissions.insert(perm);
            }

            for perm in overwrite.deny {
                permissions.remove(perm);
            }
        }
    }

    if let Some(overwrite) = overwrites.iter().find(|p| {
        if let PermissionOverwriteType::Member(id) = p.kind
            && id == ctx.cache.current_user().id
        {
            true
        } else {
            false
        }
    }) {
        for perm in overwrite.allow {
            permissions.insert(perm);
        }

        for perm in overwrite.deny {
            permissions.remove(perm);
        }
    }

    permissions
}

/// Checks if a user is a developer using the BOT_CONFIG.
pub fn is_developer(user: &User) -> bool {
    let cfg = BOT_CONFIG.get().unwrap();
    cfg.dev_ids
        .clone()
        .is_some_and(|i| i.contains(&user.id.get()))
}
