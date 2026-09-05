use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::domain::ids::RuleId;
use crate::features::automod::clause;
use crate::features::automod::rule::{Author, Body, Mode, Rule};

fn rebuild(
    id: String,
    guild: i64,
    name: String,
    source: String,
    compiled: serde_json::Value,
    mode: &str,
) -> Option<Rule> {
    let cached = match compiled.as_object().is_some_and(|fields| fields.is_empty()) {
        true => None,
        false => serde_json::from_value::<Body>(compiled).ok(),
    };

    let body = cached.or_else(|| clause::parse(&source, 0).ok())?;

    Some(Rule {
        id: RuleId::from(id),
        guild: guild as Snowflake,
        name,
        mode: Mode::parse(mode)?,
        author: Author::Guild,
        source,
        body,
    })
}

pub async fn all(pool: &PgPool, guild: Snowflake) -> Result<Vec<Rule>> {
    let rows = sqlx::query!(
        r#"SELECT id, guild_id, name, source, compiled, mode::text AS "mode!"
        FROM automod_rules WHERE guild_id = $1 ORDER BY lower(name)"#,
        guild as i64
    )
    .fetch_all(pool)
    .await
    .ctx("load automod rules")?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            rebuild(
                row.id,
                row.guild_id,
                row.name,
                row.source,
                row.compiled,
                &row.mode,
            )
        })
        .collect())
}

pub async fn find(pool: &PgPool, guild: Snowflake, name: &str) -> Result<Option<Rule>> {
    let row = sqlx::query!(
        r#"SELECT id, guild_id, name, source, compiled, mode::text AS "mode!"
        FROM automod_rules WHERE guild_id = $1 AND lower(name) = lower($2)"#,
        guild as i64,
        name
    )
    .fetch_optional(pool)
    .await
    .ctx("load automod rule")?;

    Ok(row.and_then(|row| {
        rebuild(
            row.id,
            row.guild_id,
            row.name,
            row.source,
            row.compiled,
            &row.mode,
        )
    }))
}

pub async fn by_id(pool: &PgPool, id: &str) -> Result<Option<Rule>> {
    let row = sqlx::query!(
        r#"SELECT id, guild_id, name, source, compiled, mode::text AS "mode!"
        FROM automod_rules WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
    .ctx("load automod rule by id")?;

    Ok(row.and_then(|row| {
        rebuild(
            row.id,
            row.guild_id,
            row.name,
            row.source,
            row.compiled,
            &row.mode,
        )
    }))
}

pub async fn save(pool: &PgPool, rule: &Rule) -> Result<RuleId> {
    let compiled = serde_json::to_value(&rule.body).unwrap_or_default();

    let row = sqlx::query!(
        r#"INSERT INTO automod_rules (id, guild_id, name, source, compiled, mode, rule_hash)
        VALUES ($1, $2, $3, $4, $5, $6::rule_mode, $7)
        ON CONFLICT (guild_id, lower(name)) DO UPDATE
            SET source = EXCLUDED.source,
                compiled = EXCLUDED.compiled,
                rule_hash = EXCLUDED.rule_hash,
                updated_at = now()
        RETURNING id"#,
        rule.id.as_str(),
        rule.guild as i64,
        rule.name,
        rule.source,
        compiled,
        rule.mode.as_str() as _,
        rule.hash()
    )
    .fetch_one(pool)
    .await
    .ctx("save automod rule")?;

    Ok(RuleId::from(row.id))
}

pub async fn update(pool: &PgPool, rule: &Rule) -> Result<bool> {
    let compiled = serde_json::to_value(&rule.body).unwrap_or_default();

    let done = sqlx::query!(
        r#"UPDATE automod_rules
        SET name = $2, source = $3, compiled = $4, mode = $5::rule_mode,
            rule_hash = $6, updated_at = now()
        WHERE id = $1"#,
        rule.id.as_str(),
        rule.name,
        rule.source,
        compiled,
        rule.mode.as_str() as _,
        rule.hash()
    )
    .execute(pool)
    .await
    .ctx("update automod rule")?;

    Ok(done.rows_affected() > 0)
}

pub async fn set_mode(pool: &PgPool, id: &RuleId, mode: Mode) -> Result<bool> {
    let done = sqlx::query!(
        "UPDATE automod_rules SET mode = $1::rule_mode, updated_at = now() WHERE id = $2",
        mode.as_str() as _,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("arm automod rule")?;

    Ok(done.rows_affected() > 0)
}

pub async fn delete(pool: &PgPool, guild: Snowflake, name: &str) -> Result<bool> {
    let done = sqlx::query!(
        "DELETE FROM automod_rules WHERE guild_id = $1 AND lower(name) = lower($2)",
        guild as i64,
        name
    )
    .execute(pool)
    .await
    .ctx("delete automod rule")?;

    Ok(done.rows_affected() > 0)
}

pub async fn delete_by_id(pool: &PgPool, id: &RuleId) -> Result<bool> {
    let done = sqlx::query!("DELETE FROM automod_rules WHERE id = $1", id.as_str())
        .execute(pool)
        .await
        .ctx("delete automod rule by id")?;

    Ok(done.rows_affected() > 0)
}

pub async fn evaluated(pool: &PgPool, image: &str, rules: &str) -> Result<Option<bool>> {
    let row = sqlx::query!(
        "UPDATE ocr_image_evaluations SET last_seen_at = now()
        WHERE image_hash = $1 AND rule_hash = $2 RETURNING is_match",
        image,
        rules
    )
    .fetch_optional(pool)
    .await
    .ctx("read image evaluation")?;

    Ok(row.map(|row| row.is_match))
}

pub async fn remember(pool: &PgPool, image: &str, rules: &str, matched: bool) -> Result<()> {
    sqlx::query!(
        "INSERT INTO ocr_image_evaluations (image_hash, rule_hash, is_match) VALUES ($1, $2, $3)
        ON CONFLICT (image_hash, rule_hash) DO UPDATE SET last_seen_at = now()",
        image,
        rules,
        matched
    )
    .execute(pool)
    .await
    .ctx("record image evaluation")?;

    Ok(())
}
