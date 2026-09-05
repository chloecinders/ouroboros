use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::features::permissions::rule::{Effect, Rule, RuleSet, Scope, Target};

fn rebuild(
    id: i64,
    scope: &str,
    subject: i64,
    target: String,
    effect: &str,
    priority: i32,
) -> Option<Rule> {
    Some(Rule {
        id,
        scope: Scope::parse(scope)?,
        subject: subject as Snowflake,
        target: Target::parse(&target),
        effect: Effect::parse(effect)?,
        priority,
    })
}

pub async fn all(pool: &PgPool, guild: Snowflake) -> Result<RuleSet> {
    let rows = sqlx::query!(
        r#"SELECT id, scope::text AS "scope!", subject_id, target,
            effect::text AS "effect!", priority
        FROM guild_permissions WHERE guild_id = $1 ORDER BY id"#,
        guild as i64
    )
    .fetch_all(pool)
    .await
    .ctx("load guild permissions")?;

    Ok(RuleSet::compile(
        rows.into_iter()
            .filter_map(|row| {
                rebuild(
                    row.id,
                    &row.scope,
                    row.subject_id,
                    row.target,
                    &row.effect,
                    row.priority,
                )
            })
            .collect(),
    ))
}

pub async fn add(
    pool: &PgPool,
    guild: Snowflake,
    scope: Scope,
    subject: Snowflake,
    target: &Target,
    effect: Effect,
    priority: i32,
) -> Result<i64> {
    let row = sqlx::query!(
        "INSERT INTO guild_permissions
            (guild_id, scope, subject_id, target, effect, priority)
        VALUES ($1, $2::permission_scope, $3, $4, $5::permission_effect, $6) RETURNING id",
        guild as i64,
        scope.as_str() as _,
        subject as i64,
        target.render(),
        effect.as_str() as _,
        priority
    )
    .fetch_one(pool)
    .await
    .ctx("add guild permission")?;

    Ok(row.id)
}

pub async fn set_priority(
    pool: &PgPool,
    guild: Snowflake,
    id: i64,
    priority: i32,
) -> Result<Option<Rule>> {
    let row = sqlx::query!(
        r#"UPDATE guild_permissions SET priority = $3
        WHERE guild_id = $1 AND id = $2
        RETURNING id, scope::text AS "scope!", subject_id, target,
            effect::text AS "effect!", priority"#,
        guild as i64,
        id,
        priority
    )
    .fetch_optional(pool)
    .await
    .ctx("set guild permission priority")?;

    Ok(row.and_then(|row| {
        rebuild(
            row.id,
            &row.scope,
            row.subject_id,
            row.target,
            &row.effect,
            row.priority,
        )
    }))
}

pub async fn remove(pool: &PgPool, guild: Snowflake, id: i64) -> Result<bool> {
    let done = sqlx::query!(
        "DELETE FROM guild_permissions WHERE guild_id = $1 AND id = $2",
        guild as i64,
        id
    )
    .execute(pool)
    .await
    .ctx("remove guild permission")?;

    Ok(done.rows_affected() > 0)
}

pub async fn clear(pool: &PgPool, guild: Snowflake) -> Result<u64> {
    let done = sqlx::query!(
        "DELETE FROM guild_permissions WHERE guild_id = $1",
        guild as i64
    )
    .execute(pool)
    .await
    .ctx("clear guild permissions")?;

    Ok(done.rows_affected())
}
