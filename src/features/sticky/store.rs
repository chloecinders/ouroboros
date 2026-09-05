use serenity::all::Color;
use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;

#[derive(Clone, Debug)]
pub struct Sticky {
    pub content: String,
    pub title: Option<String>,
    pub color: Option<Color>,
    pub last: Option<Snowflake>,
}

pub async fn get(pool: &PgPool, channel: Snowflake) -> Result<Option<Sticky>> {
    let row = sqlx::query!(
        "SELECT content, title, color, last_message_id FROM sticky_messages WHERE channel_id = $1",
        channel as i64
    )
    .fetch_optional(pool)
    .await
    .ctx("read sticky message")?;

    Ok(row.map(|row| Sticky {
        content: row.content,
        title: row.title,
        color: row.color.map(|color| Color::new(color as u32)),
        last: row.last_message_id.map(|id| id as Snowflake),
    }))
}

pub async fn set(
    pool: &PgPool,
    guild: Snowflake,
    channel: Snowflake,
    content: &str,
    title: Option<&str>,
    color: Option<Color>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO sticky_messages (channel_id, guild_id, content, title, color)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (channel_id) DO UPDATE SET content = EXCLUDED.content,
            title = EXCLUDED.title, color = EXCLUDED.color, updated_at = now()",
        channel as i64,
        guild as i64,
        content,
        title,
        color.map(|color| color.0 as i64)
    )
    .execute(pool)
    .await
    .ctx("save sticky message")?;

    Ok(())
}

pub async fn clear(pool: &PgPool, channel: Snowflake) -> Result<bool> {
    let done = sqlx::query!(
        "DELETE FROM sticky_messages WHERE channel_id = $1",
        channel as i64
    )
    .execute(pool)
    .await
    .ctx("clear sticky message")?;

    Ok(done.rows_affected() > 0)
}

pub async fn mark_posted(pool: &PgPool, channel: Snowflake, message: Snowflake) -> Result<()> {
    sqlx::query!(
        "UPDATE sticky_messages SET last_message_id = $1 WHERE channel_id = $2",
        message as i64,
        channel as i64
    )
    .execute(pool)
    .await
    .ctx("record sticky message")?;

    Ok(())
}
