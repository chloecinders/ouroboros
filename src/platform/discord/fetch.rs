use std::collections::HashMap;

use serenity::all::{
    CacheHttp, ChannelId, GuildChannel, GuildId, Member, Permissions, RoleId, User, UserId,
};

use crate::command::error::{Ctx, Error, Result};
use crate::platform::discord::permissions::{Role, Snapshot};

pub async fn snapshot(discord: impl CacheHttp, guild: GuildId) -> Result<Snapshot> {
    if let Some(cached) = discord.cache().and_then(|cache| cache.guild(guild)) {
        return Ok(Snapshot {
            guild,
            owner: cached.owner_id,
            roles: roles(
                cached
                    .roles
                    .iter()
                    .map(|(id, role)| (*id, role.permissions, role.position)),
            ),
        });
    }

    let fetched = guild
        .to_partial_guild(discord.http())
        .await
        .ctx("fetch guild")?;

    Ok(Snapshot {
        guild,
        owner: fetched.owner_id,
        roles: roles(
            fetched
                .roles
                .iter()
                .map(|(id, role)| (*id, role.permissions, role.position)),
        ),
    })
}

fn roles(roles: impl Iterator<Item = (RoleId, Permissions, u16)>) -> HashMap<RoleId, Role> {
    roles
        .map(|(id, permissions, position)| {
            (
                id,
                Role {
                    permissions,
                    position: position as i64,
                },
            )
        })
        .collect()
}

pub async fn member(discord: impl CacheHttp, guild: GuildId, user: UserId) -> Result<Member> {
    if let Some(cached) = discord
        .cache()
        .and_then(|cache| cache.guild(guild))
        .and_then(|guild| guild.members.get(&user).cloned())
    {
        return Ok(cached);
    }

    guild.member(discord.http(), user).await.ctx("fetch member")
}

pub async fn channel(
    discord: impl CacheHttp,
    guild: GuildId,
    channel: ChannelId,
) -> Result<GuildChannel> {
    if let Some(cached) = discord
        .cache()
        .and_then(|cache| cache.guild(guild))
        .and_then(|guild| guild.channels.get(&channel).cloned())
    {
        return Ok(cached);
    }

    let fetched = channel
        .to_channel(&discord)
        .await
        .ctx("fetch channel")?
        .guild()
        .ctx("channel is not a guild channel")?;

    match fetched.guild_id == guild {
        true => Ok(fetched),
        false => Err(Error::bare().title("channel not in this server")),
    }
}

pub async fn user(discord: impl CacheHttp, user: UserId) -> Result<User> {
    if let Some(cached) = discord.cache().and_then(|cache| cache.user(user)) {
        return Ok(cached.clone());
    }

    discord.http().get_user(user).await.ctx("fetch user")
}

pub async fn guild_name(discord: impl CacheHttp, guild: GuildId) -> String {
    if let Some(cached) = discord.cache().and_then(|cache| cache.guild(guild)) {
        return cached.name.clone();
    }

    guild
        .to_partial_guild(discord.http())
        .await
        .ctx("fetch guild name")
        .map(|fetched| fetched.name)
        .unwrap_or_else(|_| String::from("UNKNOWN_GUILD"))
}
