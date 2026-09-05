use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::domain::action::Action;
use crate::domain::ids::ActionId;
use crate::domain::punishment::{PunishmentState, PunishmentType};
use crate::domain::reason::{Note, Reason};

struct Row {
    id: String,
    guild_id: i64,
    user_id: i64,
    moderator_id: i64,
    verb: String,
    state: String,
    reason: String,
    note: Option<String>,
    clear_days: i16,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

impl Row {
    fn hydrate(self) -> Option<Action> {
        Some(Action {
            id: ActionId::from(self.id),
            verb: PunishmentType::parse(&self.verb)?,
            guild: self.guild_id as Snowflake,
            target: self.user_id as Snowflake,
            actor: self.moderator_id as Snowflake,
            reason: Reason::new(&self.reason),
            note: self.note.as_deref().and_then(Note::new),
            state: PunishmentState::parse(&self.state)?,
            clear_days: self.clear_days as u8,
            created_at: self.created_at,
            updated_at: self.updated_at,
            expires_at: self.expires_at,
        })
    }
}

pub async fn log_target(pool: &PgPool, message: Snowflake) -> Result<Option<Snowflake>> {
    let found = sqlx::query_scalar!(
        "SELECT target_id FROM log_messages WHERE message_id = $1",
        message as i64
    )
    .fetch_optional(pool)
    .await
    .ctx("read log message target")?;

    Ok(found.map(|target| target as Snowflake))
}

pub async fn action_for_message(pool: &PgPool, message: Snowflake) -> Result<Option<ActionId>> {
    let found = sqlx::query_scalar!(
        "SELECT action_id FROM log_messages WHERE message_id = $1",
        message as i64
    )
    .fetch_optional(pool)
    .await
    .ctx("read log message action")?;

    Ok(found.flatten().map(ActionId::from))
}

pub async fn load(pool: &PgPool, guild: Snowflake, id: &ActionId) -> Result<Option<Action>> {
    let row = sqlx::query_as!(
        Row,
        r#"SELECT id, guild_id, user_id, moderator_id, verb::text AS "verb!", state::text AS "state!",
            reason, note, clear_days, created_at, updated_at, expires_at
        FROM actions WHERE guild_id = $1 AND id = $2"#,
        guild as i64,
        id.as_str()
    )
    .fetch_optional(pool)
    .await
    .ctx("load action")?;

    Ok(row.and_then(Row::hydrate))
}

pub async fn history(
    pool: &PgPool,
    guild: Snowflake,
    user: Snowflake,
    page: i64,
) -> Result<Vec<Action>> {
    let rows = sqlx::query_as!(
        Row,
        r#"SELECT id, guild_id, user_id, moderator_id, verb::text AS "verb!", state::text AS "state!",
            reason, note, clear_days, created_at, updated_at, expires_at
        FROM actions WHERE guild_id = $1 AND user_id = $2
        ORDER BY created_at DESC LIMIT $3 OFFSET $4"#,
        guild as i64,
        user as i64,
        5,
        (page - 1).max(0) * 5
    )
    .fetch_all(pool)
    .await
    .ctx("read action history")?;

    Ok(rows.into_iter().filter_map(Row::hydrate).collect())
}

pub async fn active(
    pool: &PgPool,
    guild: Snowflake,
    user: Snowflake,
    verb: PunishmentType,
) -> Result<Option<Action>> {
    let row = sqlx::query_as!(
        Row,
        r#"SELECT id, guild_id, user_id, moderator_id, verb::text AS "verb!", state::text AS "state!",
            reason, note, clear_days, created_at, updated_at, expires_at
        FROM actions
        WHERE guild_id = $1 AND user_id = $2 AND verb = $3::punishment_verb
            AND state IN ('active', 'expiring')
        ORDER BY created_at DESC LIMIT 1"#,
        guild as i64,
        user as i64,
        verb.as_str() as _
    )
    .fetch_optional(pool)
    .await
    .ctx("load active punishment")?;

    Ok(row.and_then(Row::hydrate))
}

pub async fn all_active(
    pool: &PgPool,
    guild: Snowflake,
    verb: PunishmentType,
) -> Result<Vec<Action>> {
    let rows = sqlx::query_as!(
        Row,
        r#"SELECT id, guild_id, user_id, moderator_id, verb::text AS "verb!", state::text AS "state!",
            reason, note, clear_days, created_at, updated_at, expires_at
        FROM actions
        WHERE guild_id = $1 AND verb = $2::punishment_verb AND state IN ('active', 'expiring')"#,
        guild as i64,
        verb.as_str() as _
    )
    .fetch_all(pool)
    .await
    .ctx("read active punishments")?;

    Ok(rows.into_iter().filter_map(Row::hydrate).collect())
}

pub async fn count(
    pool: &PgPool,
    guild: Snowflake,
    user: Snowflake,
    since: Option<DateTime<Utc>>,
) -> Result<Vec<(PunishmentType, DateTime<Utc>)>> {
    let rows = sqlx::query!(
        r#"SELECT verb::text AS "verb!", created_at
        FROM actions
        WHERE guild_id = $1 AND user_id = $2 AND state <> 'failed'
            AND ($3::timestamptz IS NULL OR created_at >= $3)"#,
        guild as i64,
        user as i64,
        since
    )
    .fetch_all(pool)
    .await
    .ctx("read member record")?;

    Ok(rows
        .into_iter()
        .filter_map(|row| PunishmentType::parse(&row.verb).map(|verb| (verb, row.created_at)))
        .collect())
}

pub async fn set_reason(
    pool: &PgPool,
    guild: Snowflake,
    id: &ActionId,
    reason: &Reason,
) -> Result<bool> {
    let done = sqlx::query!(
        "UPDATE actions SET reason = $1, updated_at = now() WHERE guild_id = $2 AND id = $3",
        reason.as_str(),
        guild as i64,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("amend reason")?;

    Ok(done.rows_affected() > 0)
}

pub async fn set_note(
    pool: &PgPool,
    guild: Snowflake,
    id: &ActionId,
    note: Option<&Note>,
) -> Result<bool> {
    let done = sqlx::query!(
        "UPDATE actions SET note = $1, updated_at = now() WHERE guild_id = $2 AND id = $3",
        note.map(|note| note.as_str()),
        guild as i64,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("amend note")?;

    Ok(done.rows_affected() > 0)
}

pub async fn set_expiry(
    pool: &PgPool,
    guild: Snowflake,
    id: &ActionId,
    duration: Duration,
) -> Result<bool> {
    let expires_at = match duration.is_zero() {
        true => None,
        false => Some(Utc::now() + duration),
    };

    let done = sqlx::query!(
        "UPDATE actions SET expires_at = $1, updated_at = now() WHERE guild_id = $2 AND id = $3",
        expires_at,
        guild as i64,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("amend duration")?;

    Ok(done.rows_affected() > 0)
}

pub async fn delete(pool: &PgPool, guild: Snowflake, id: &ActionId) -> Result<bool> {
    let done = sqlx::query!(
        "DELETE FROM actions WHERE guild_id = $1 AND id = $2",
        guild as i64,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("delete action")?;

    Ok(done.rows_affected() > 0)
}

pub struct Invocation {
    pub author: Snowflake,
    pub command: String,
    pub args: serde_json::Value,
    pub action: Option<ActionId>,
    pub response: Option<Snowflake>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

pub async fn remember_invocation(
    pool: &PgPool,
    guild: Snowflake,
    channel: Snowflake,
    message: Snowflake,
    author: Snowflake,
    command: &str,
    args: &serde_json::Value,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO invocations (message_id, guild_id, channel_id, author_id, command, args) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (message_id) DO UPDATE SET command = EXCLUDED.command, \
         args = EXCLUDED.args, status = 'running'",
        message as i64,
        guild as i64,
        channel as i64,
        author as i64,
        command,
        args
    )
    .execute(pool)
    .await
    .ctx("record invocation")?;

    Ok(())
}

pub async fn close_invocation(
    pool: &PgPool,
    message: Snowflake,
    args: Option<&serde_json::Value>,
    action: Option<&ActionId>,
    response: Option<Snowflake>,
    failed: bool,
) -> Result<()> {
    sqlx::query!(
        "UPDATE invocations SET action_id = $1, response_id = $2, \
         args = COALESCE($3::jsonb, args), \
         status = (CASE WHEN $4 THEN 'failed' ELSE 'complete' END)::invocation_status \
         WHERE message_id = $5",
        action.map(|id| id.as_str()),
        response.map(|id| id as i64),
        args,
        failed,
        message as i64
    )
    .execute(pool)
    .await
    .ctx("close invocation")?;

    Ok(())
}

pub async fn load_invocation(pool: &PgPool, message: Snowflake) -> Result<Option<Invocation>> {
    let row = sqlx::query!(
        r#"SELECT message_id, author_id, command, args, action_id, response_id,
                  status::text AS "status!", created_at
           FROM invocations WHERE message_id = $1"#,
        message as i64
    )
    .fetch_optional(pool)
    .await
    .ctx("load invocation")?;

    Ok(row.map(|row| Invocation {
        author: row.author_id as Snowflake,
        command: row.command,
        args: row.args,
        action: row.action_id.map(ActionId::from),
        response: row.response_id.map(|id| id as Snowflake),
        status: row.status,
        created_at: row.created_at,
    }))
}
