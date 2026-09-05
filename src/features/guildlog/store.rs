use serenity::all::{ChannelId, MessageId};
use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::domain::ids::ActionId;

pub struct Entry {
    pub message: MessageId,
    pub channel: ChannelId,
    pub guild: Snowflake,
    pub target: Snowflake,
    pub moderator: Option<Snowflake>,
    pub action: Option<ActionId>,
}

pub async fn remember(pool: &PgPool, entry: &Entry) -> Result<()> {
    sqlx::query!(
        "INSERT INTO log_messages (message_id, guild_id, channel_id, target_id, moderator_id, action_id)
        VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (message_id) DO NOTHING",
        entry.message.get() as i64,
        entry.guild as i64,
        entry.channel.get() as i64,
        entry.target as i64,
        entry.moderator.map(|id| id as i64),
        entry.action.as_ref().map(|id| id.as_str())
    )
    .execute(pool)
    .await
    .ctx("remember log message")?;

    Ok(())
}

pub async fn attribute(pool: &PgPool, message: Snowflake, actor: Option<Snowflake>) -> Result<()> {
    sqlx::query!(
        "UPDATE log_messages SET moderator_id = $1 WHERE message_id = $2",
        actor.map(|id| id as i64),
        message as i64
    )
    .execute(pool)
    .await
    .ctx("attribute log message")?;

    Ok(())
}

pub async fn locate(
    pool: &PgPool,
    guild: Snowflake,
    action: &ActionId,
) -> Result<Option<(ChannelId, MessageId)>> {
    let found = sqlx::query!(
        "SELECT channel_id, message_id FROM log_messages WHERE guild_id = $1 AND action_id = $2",
        guild as i64,
        action.as_str()
    )
    .fetch_optional(pool)
    .await
    .ctx("locate log message")?;

    Ok(found.and_then(|row| {
        row.channel_id.map(|channel| {
            (
                ChannelId::new(channel as u64),
                MessageId::new(row.message_id as u64),
            )
        })
    }))
}
