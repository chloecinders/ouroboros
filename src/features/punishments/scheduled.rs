use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::domain::ids::ActionId;
use crate::domain::punishment::PunishmentState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    LiftBan,
    LiftMute,
    RefreshTimeout,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::LiftBan => "lift_ban",
            Kind::LiftMute => "lift_mute",
            Kind::RefreshTimeout => "refresh_timeout",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "lift_ban" => Some(Kind::LiftBan),
            "lift_mute" => Some(Kind::LiftMute),
            "refresh_timeout" => Some(Kind::RefreshTimeout),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Due {
    pub id: i64,
    pub kind: Kind,
    pub action: Option<ActionId>,
    pub guild: Snowflake,
    pub user: Option<Snowflake>,
    pub attempts: i32,
}

pub fn next_refresh(expires_at: Option<DateTime<Utc>>) -> DateTime<Utc> {
    let ceiling = Utc::now() + Duration::days(27);
    let horizon = match expires_at {
        Some(expiry) if expiry < ceiling => expiry,
        _ => ceiling,
    };

    horizon - Duration::hours(1)
}

pub fn backoff(attempts: i32) -> Duration {
    let doubled = 30i64.saturating_mul(1i64 << attempts.clamp(0, 20));

    Duration::seconds(doubled.min(3600))
}

pub async fn schedule(
    pool: &PgPool,
    kind: Kind,
    action: &ActionId,
    guild: Snowflake,
    user: Snowflake,
    due_at: DateTime<Utc>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO scheduled_work (kind, action_id, guild_id, user_id, due_at) \
         VALUES ($1::work_kind, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
        kind.as_str() as _,
        action.as_str(),
        guild as i64,
        user as i64,
        due_at
    )
    .execute(pool)
    .await
    .ctx("schedule due work")?;

    Ok(())
}

pub async fn claim(pool: &PgPool) -> Result<Vec<Due>> {
    let rows = sqlx::query!(
        r#"UPDATE scheduled_work SET locked_until = now() + make_interval(secs => $1::double precision),
                                     attempts = attempts + 1
           WHERE id IN (
               SELECT id FROM scheduled_work
               WHERE state = 'pending' AND due_at <= now() AND locked_until < now()
               ORDER BY due_at LIMIT $2 FOR UPDATE SKIP LOCKED
           )
           RETURNING id, kind::text AS "kind!", action_id, guild_id, user_id, attempts"#,
        60.0,
        32i64
    )
    .fetch_all(pool)
    .await
    .ctx("claim due work")?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            Some(Due {
                id: row.id,
                kind: Kind::parse(&row.kind)?,
                action: row.action_id.map(ActionId::from),
                guild: row.guild_id as Snowflake,
                user: row.user_id.map(|id| id as Snowflake),
                attempts: row.attempts,
            })
        })
        .collect())
}

pub async fn finish(pool: &PgPool, id: i64) -> Result<()> {
    sqlx::query!("UPDATE scheduled_work SET state = 'done' WHERE id = $1", id)
        .execute(pool)
        .await
        .ctx("close due work")?;

    Ok(())
}

pub async fn defer(pool: &PgPool, due: &Due, why: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE scheduled_work SET due_at = now() + make_interval(secs => $1::double precision), \
         locked_until = to_timestamp(0), last_error = $2 WHERE id = $3",
        backoff(due.attempts).num_seconds() as f64,
        why,
        due.id
    )
    .execute(pool)
    .await
    .ctx("defer due work")?;

    Ok(())
}

pub async fn abandon(pool: &PgPool, due: &Due, why: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE scheduled_work SET state = 'failed', last_error = $1 WHERE id = $2",
        why,
        due.id
    )
    .execute(pool)
    .await
    .ctx("abandon due work")?;

    mark_state(pool, due, PunishmentState::Failed).await
}

pub async fn mark_state(pool: &PgPool, due: &Due, state: PunishmentState) -> Result<()> {
    let Some(action) = due.action.as_ref() else {
        return Ok(());
    };

    sqlx::query!(
        "UPDATE actions SET state = $1::punishment_state, updated_at = now() WHERE id = $2",
        state.as_str() as _,
        action.as_str()
    )
    .execute(pool)
    .await
    .ctx("mark punishment state")?;

    Ok(())
}

pub async fn cancel(pool: &PgPool, action: &ActionId, kind: Kind) -> Result<()> {
    sqlx::query!(
        "UPDATE scheduled_work SET state = 'done' \
         WHERE action_id = $1 AND kind = $2::work_kind AND state = 'pending'",
        action.as_str(),
        kind.as_str() as _
    )
    .execute(pool)
    .await
    .ctx("cancel due work")?;

    Ok(())
}
