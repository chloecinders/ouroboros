use serenity::all::ChannelId;
use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::domain::logtype::LogType;

pub async fn enroll(pool: &PgPool, guild: Snowflake) -> Result<()> {
    sqlx::query!(
        "INSERT INTO guild_settings (guild_id) VALUES ($1) ON CONFLICT DO NOTHING",
        guild as i64
    )
    .execute(pool)
    .await
    .ctx("enrol guild")?;

    Ok(())
}

pub async fn route_many(
    pool: &PgPool,
    guild: Snowflake,
    kinds: &[LogType],
    channel: ChannelId,
) -> Result<()> {
    if kinds.is_empty() {
        return Ok(());
    }

    enroll(pool, guild).await?;

    let names: Vec<String> = kinds.iter().map(|kind| kind.as_str().to_string()).collect();

    sqlx::query!(
        "INSERT INTO guild_log_channels (guild_id, log_type, channel_id)
        SELECT $1, kind, $3 FROM UNNEST($2::text[]) AS kind
        ON CONFLICT (guild_id, log_type) DO UPDATE SET channel_id = EXCLUDED.channel_id",
        guild as i64,
        &names,
        channel.get() as i64
    )
    .execute(pool)
    .await
    .ctx("route several log types")?;

    Ok(())
}

pub async fn unroute(pool: &PgPool, guild: Snowflake, kind: LogType) -> Result<bool> {
    let done = sqlx::query!(
        "DELETE FROM guild_log_channels WHERE guild_id = $1 AND log_type = $2",
        guild as i64,
        kind.as_str()
    )
    .execute(pool)
    .await
    .ctx("stop routing a log type")?;

    Ok(done.rows_affected() > 0)
}

pub async fn clear_channel(pool: &PgPool, guild: Snowflake, channel: ChannelId) -> Result<()> {
    sqlx::query!(
        "DELETE FROM guild_log_channels WHERE guild_id = $1 AND channel_id = $2",
        guild as i64,
        channel.get() as i64
    )
    .execute(pool)
    .await
    .ctx("clear a channel of log types")?;

    Ok(())
}

pub async fn everywhere(pool: &PgPool, kind: LogType) -> Result<Vec<(Snowflake, ChannelId)>> {
    let rows = sqlx::query!(
        "SELECT guild_id, channel_id FROM guild_log_channels WHERE log_type = $1",
        kind.as_str()
    )
    .fetch_all(pool)
    .await
    .ctx("read every route for a log type")?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.guild_id as Snowflake,
                ChannelId::new(row.channel_id as u64),
            )
        })
        .collect())
}

pub async fn routes(pool: &PgPool, guild: Snowflake) -> Result<Vec<(LogType, ChannelId)>> {
    let rows = sqlx::query!(
        "SELECT log_type, channel_id FROM guild_log_channels WHERE guild_id = $1",
        guild as i64
    )
    .fetch_all(pool)
    .await
    .ctx("read log routes")?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            Some((
                LogType::parse(&row.log_type)?,
                ChannelId::new(row.channel_id as u64),
            ))
        })
        .collect())
}
