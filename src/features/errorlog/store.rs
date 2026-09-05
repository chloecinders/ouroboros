use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::features::errorlog::Fault;

pub struct Recorded {
    pub id: i64,
    pub headline: String,
    pub detail: Option<String>,
    pub delivered: bool,
    pub occurred_at: DateTime<Utc>,
}

pub async fn keep(pool: &PgPool, guild: Snowflake, fault: &Fault) -> Result<i64> {
    let row = sqlx::query!(
        "INSERT INTO guild_errors (guild_id, headline, detail)
        VALUES ($1, $2, $3) RETURNING id",
        guild as i64,
        fault.headline,
        fault.detail
    )
    .fetch_one(pool)
    .await
    .ctx("record a guild error")?;

    Ok(row.id)
}

pub async fn delivered(pool: &PgPool, id: i64) -> Result<()> {
    sqlx::query!("UPDATE guild_errors SET delivered = true WHERE id = $1", id)
        .execute(pool)
        .await
        .ctx("mark a guild error delivered")?;

    Ok(())
}

pub async fn recent(pool: &PgPool, guild: Snowflake, limit: i64) -> Result<Vec<Recorded>> {
    let rows = sqlx::query!(
        "SELECT id, headline, detail, delivered, occurred_at
        FROM guild_errors WHERE guild_id = $1
        ORDER BY occurred_at DESC, id DESC LIMIT $2",
        guild as i64,
        limit
    )
    .fetch_all(pool)
    .await
    .ctx("read the error log")?;

    Ok(rows
        .into_iter()
        .map(|row| Recorded {
            id: row.id,
            headline: row.headline,
            detail: row.detail,
            delivered: row.delivered,
            occurred_at: row.occurred_at,
        })
        .collect())
}

pub async fn prune(pool: &PgPool, days: i64) -> Result<u64> {
    let deleted = sqlx::query!(
        "DELETE FROM guild_errors WHERE occurred_at < $1",
        Utc::now() - Duration::days(days)
    )
    .execute(pool)
    .await
    .ctx("sweep the error log")?;

    Ok(deleted.rows_affected())
}
