use std::collections::HashMap;

use serenity::all::{Context, Guild, GuildChannel, Member, PermissionOverwriteType, Permissions, User};

use crate::BOT_CONFIG;

pub async fn check_guild_permission(
    ctx: &Context,
    member: &Member,
    permission: Permissions,
) -> bool {
    let Some(guild_cached) = member.guild_id.to_guild_cached(&ctx.cache) else {
        return false;
    };

    if guild_cached.owner_id.get() == member.user.id.get() {
        return true;
    }

    for role in member.roles.clone() {
        let Some(role) = guild_cached.roles.get(&role) else {
            return false;
        };

        if role.has_permission(permission) || role.has_permission(Permissions::ADMINISTRATOR) {
            return true;
        }
    }

    false
}

pub fn permissions_for_guild(guild: Guild, member: &Member) -> HashMap<Permissions, bool> {
    let everyone = guild.roles.iter().find(|r| r.1.position == 0).unwrap();
    let mut roles = member.roles.iter().map(|r| guild.roles.get(r).unwrap()).collect::<Vec<_>>();
    roles.push(everyone.1);
    roles.sort();

    let mut base = Permissions::all().into_iter().map(|p| (p, false)).collect::<HashMap<_,_>>();

    for role in roles {
        role.permissions.into_iter().for_each(|p| {
            *base.entry(p).or_insert(false) = true;
        });
    }

    base
}

pub fn permissions_for_channel(ctx: &Context, channel: GuildChannel, member: &Member) -> HashMap<Permissions, bool> {
    let Some(guild) = channel.guild(&ctx.cache) else {
        return Permissions::all().into_iter().map(|p| (p, false)).collect::<HashMap<_,_>>()
    };
    let mut permissions = permissions_for_guild(guild.to_owned(), member);
    let everyone = guild.roles.iter().find(|r| r.1.position == 0).unwrap();
    let mut roles = member.roles.iter().map(|r| guild.roles.get(r).unwrap()).collect::<Vec<_>>();
    roles.push(everyone.1);
    roles.sort();

    let overwrites = channel.permission_overwrites;

    for role in roles {
        if let Some(overwrite) = overwrites.iter().find(
            |p| if let PermissionOverwriteType::Role(id) = p.kind && id == role.id { true } else { false })
        {
            for perm in overwrite.allow {
                *permissions.entry(perm).or_insert(false) = true;
            }

            for perm in overwrite.deny {
                *permissions.entry(perm).or_insert(false) = false;
            }
        }
    }

    if let Some(overwrite) = overwrites.iter().find(
        |p| if let PermissionOverwriteType::Member(id) = p.kind && id == ctx.cache.current_user().id { true } else { false })
    {
        for perm in overwrite.allow {
            *permissions.entry(perm).or_insert(false) = true;
        }

        for perm in overwrite.deny {
            *permissions.entry(perm).or_insert(false) = false;
        }
    }

   permissions
}

pub fn is_developer(user: &User) -> bool {
    let cfg = BOT_CONFIG.get().unwrap();
    cfg.dev_ids
        .clone()
        .is_some_and(|i| i.contains(&user.id.get()))
}
