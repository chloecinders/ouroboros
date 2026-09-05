use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::domain::ids::RuleId;
use crate::features::automod::clause::{self, Part};
use crate::features::automod::managed::{Managed, Offer, Subscription, combine};
use crate::features::automod::rule::{Body, Mode, Rule};

fn body_of(source: &str, compiled: serde_json::Value, part: Part) -> Option<Body> {
    serde_json::from_value::<Body>(compiled)
        .ok()
        .or_else(|| clause::parse_as(source, 0, part).ok())
}

fn rebuild(
    id: String,
    name: String,
    description: String,
    source: String,
    compiled: serde_json::Value,
    mode: &str,
) -> Option<Managed> {
    let body = body_of(&source, compiled, Part::Detection)?;

    Some(Managed {
        id: RuleId::from(id),
        name,
        description,
        mode: Mode::parse(mode)?,
        source,
        body,
    })
}

fn subscribed(
    rule: RuleId,
    guild: i64,
    mode: &str,
    response: String,
    compiled: serde_json::Value,
) -> Option<Subscription> {
    Some(Subscription {
        rule,
        guild: guild as Snowflake,
        mode: Mode::parse(mode)?,
        response: body_of(&response, compiled, Part::Response)?,
        written: response,
    })
}

pub async fn all(pool: &PgPool) -> Result<Vec<Managed>> {
    let rows = sqlx::query!(
        r#"SELECT id, name, description, source, compiled, mode::text AS "mode!"
            FROM managed_rules ORDER BY lower(name)"#
    )
    .fetch_all(pool)
    .await
    .ctx("load managed rules")?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            rebuild(
                row.id,
                row.name,
                row.description,
                row.source,
                row.compiled,
                &row.mode,
            )
        })
        .collect())
}

pub async fn find(pool: &PgPool, name: &str) -> Result<Option<Managed>> {
    let row = sqlx::query!(
        r#"SELECT id, name, description, source, compiled, mode::text AS "mode!"
           FROM managed_rules WHERE lower(name) = lower($1)"#,
        name
    )
    .fetch_optional(pool)
    .await
    .ctx("load managed rule")?;

    Ok(row.and_then(|row| {
        rebuild(
            row.id,
            row.name,
            row.description,
            row.source,
            row.compiled,
            &row.mode,
        )
    }))
}

pub async fn by_id(pool: &PgPool, id: &str) -> Result<Option<Managed>> {
    let row = sqlx::query!(
        r#"SELECT id, name, description, source, compiled, mode::text AS "mode!"
           FROM managed_rules WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
    .ctx("load managed rule by id")?;

    Ok(row.and_then(|row| {
        rebuild(
            row.id,
            row.name,
            row.description,
            row.source,
            row.compiled,
            &row.mode,
        )
    }))
}

pub async fn save(pool: &PgPool, managed: &Managed) -> Result<RuleId> {
    let compiled = serde_json::to_value(&managed.body).unwrap_or_default();

    let row = sqlx::query!(
        r#"INSERT INTO managed_rules (id, name, description, source, compiled, mode, rule_hash)
        VALUES ($1, $2, $3, $4, $5, $6::rule_mode, $7)
        ON CONFLICT (lower(name)) DO UPDATE
        SET source = EXCLUDED.source,
            compiled = EXCLUDED.compiled,
            rule_hash = EXCLUDED.rule_hash,
            updated_at = now()
        RETURNING id"#,
        managed.id.as_str(),
        managed.name,
        managed.description,
        managed.source,
        compiled,
        managed.mode.as_str() as _,
        managed.body.hash()
    )
    .fetch_one(pool)
    .await
    .ctx("save managed rule")?;

    Ok(RuleId::from(row.id))
}

pub async fn rewrite(pool: &PgPool, managed: &Managed) -> Result<bool> {
    let compiled = serde_json::to_value(&managed.body).unwrap_or_default();

    let done = sqlx::query!(
        r#"UPDATE managed_rules
        SET name = $2, description = $3, source = $4, compiled = $5, mode = $6::rule_mode,
            rule_hash = $7, updated_at = now()
        WHERE id = $1"#,
        managed.id.as_str(),
        managed.name,
        managed.description,
        managed.source,
        compiled,
        managed.mode.as_str() as _,
        managed.body.hash()
    )
    .execute(pool)
    .await
    .ctx("rewrite managed rule")?;

    Ok(done.rows_affected() > 0)
}

pub async fn delete_by_id(pool: &PgPool, id: &RuleId) -> Result<bool> {
    let done = sqlx::query!("DELETE FROM managed_rules WHERE id = $1", id.as_str())
        .execute(pool)
        .await
        .ctx("delete managed rule by id")?;

    Ok(done.rows_affected() > 0)
}

pub async fn set_description(pool: &PgPool, id: &RuleId, description: &str) -> Result<bool> {
    let done = sqlx::query!(
        "UPDATE managed_rules SET description = $1, updated_at = now() WHERE id = $2",
        description,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("set managed rule description")?;

    Ok(done.rows_affected() > 0)
}

pub async fn set_mode(pool: &PgPool, id: &RuleId, mode: Mode) -> Result<bool> {
    let done = sqlx::query!(
        "UPDATE managed_rules SET mode = $1::rule_mode, updated_at = now() WHERE id = $2",
        mode.as_str() as _,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("publish managed rule")?;

    Ok(done.rows_affected() > 0)
}

pub async fn delete(pool: &PgPool, name: &str) -> Result<bool> {
    let done = sqlx::query!(
        "DELETE FROM managed_rules WHERE lower(name) = lower($1)",
        name
    )
    .execute(pool)
    .await
    .ctx("delete managed rule")?;

    Ok(done.rows_affected() > 0)
}

pub async fn subscription(
    pool: &PgPool,
    guild: Snowflake,
    id: &RuleId,
) -> Result<Option<Subscription>> {
    let row = sqlx::query!(
        r#"SELECT mode::text AS "mode!", response, compiled
        FROM managed_rule_guilds WHERE guild_id = $1 AND rule_id = $2"#,
        guild as i64,
        id.as_str()
    )
    .fetch_optional(pool)
    .await
    .ctx("read managed rule subscription")?;

    Ok(row.and_then(|row| {
        subscribed(
            id.clone(),
            guild as i64,
            &row.mode,
            row.response,
            row.compiled,
        )
    }))
}

pub async fn subscriptions(pool: &PgPool, guild: Snowflake) -> Result<Vec<Subscription>> {
    let rows = sqlx::query!(
        r#"SELECT rule_id, mode::text AS "mode!", response, compiled
        FROM managed_rule_guilds WHERE guild_id = $1"#,
        guild as i64
    )
    .fetch_all(pool)
    .await
    .ctx("read managed rule subscriptions")?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            subscribed(
                RuleId::from(row.rule_id),
                guild as i64,
                &row.mode,
                row.response,
                row.compiled,
            )
        })
        .collect())
}

pub async fn subscribe(pool: &PgPool, guild: Snowflake, id: &RuleId) -> Result<bool> {
    let done = sqlx::query!(
        "INSERT INTO managed_rule_guilds (rule_id, guild_id) VALUES ($1, $2)
        ON CONFLICT DO NOTHING",
        id.as_str(),
        guild as i64
    )
    .execute(pool)
    .await
    .ctx("subscribe to managed rule")?;

    Ok(done.rows_affected() > 0)
}

pub async fn unsubscribe(pool: &PgPool, guild: Snowflake, id: &RuleId) -> Result<bool> {
    let done = sqlx::query!(
        "DELETE FROM managed_rule_guilds WHERE guild_id = $1 AND rule_id = $2",
        guild as i64,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("unsubscribe from managed rule")?;

    Ok(done.rows_affected() > 0)
}

pub async fn set_guild_mode(
    pool: &PgPool,
    guild: Snowflake,
    id: &RuleId,
    mode: Mode,
) -> Result<bool> {
    let done = sqlx::query!(
        "UPDATE managed_rule_guilds SET mode = $1::rule_mode
        WHERE guild_id = $2 AND rule_id = $3",
        mode.as_str() as _,
        guild as i64,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("set managed rule mode")?;

    Ok(done.rows_affected() > 0)
}

pub async fn set_response(
    pool: &PgPool,
    guild: Snowflake,
    id: &RuleId,
    written: &str,
    response: &Body,
) -> Result<bool> {
    let compiled = serde_json::to_value(response).unwrap_or_default();

    let done = sqlx::query!(
        "UPDATE managed_rule_guilds SET response = $1, compiled = $2
        WHERE guild_id = $3 AND rule_id = $4",
        written,
        compiled,
        guild as i64,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("set managed rule response")?;

    Ok(done.rows_affected() > 0)
}

pub async fn offers(pool: &PgPool, guild: Snowflake) -> Result<Vec<Offer>> {
    let offered = all(pool).await?;
    let taken = subscriptions(pool, guild).await?;

    Ok(offered
        .into_iter()
        .filter_map(|managed| {
            let subscription = taken.iter().find(|taken| taken.rule == managed.id).cloned();

            match managed.mode == Mode::Active || subscription.is_some() {
                true => Some(Offer {
                    managed,
                    subscription,
                }),
                false => None,
            }
        })
        .collect())
}

pub async fn enabled(pool: &PgPool, guild: Snowflake) -> Result<Vec<Rule>> {
    let rows = sqlx::query!(
        r#"SELECT r.id, r.name, r.description, r.source, r.compiled, r.mode::text AS "mode!",
        g.mode::text AS "guild_mode!", g.response, g.compiled AS "answer"
        FROM managed_rule_guilds AS g
        JOIN managed_rules AS r ON r.id = g.rule_id
        WHERE g.guild_id = $1 AND g.mode <> 'disabled' AND r.mode <> 'disabled'
        ORDER BY lower(r.name)"#,
        guild as i64
    )
    .fetch_all(pool)
    .await
    .ctx("load enabled managed rules")?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let managed = rebuild(
                row.id,
                row.name,
                row.description,
                row.source,
                row.compiled,
                &row.mode,
            )?;
            let subscription = subscribed(
                managed.id.clone(),
                guild as i64,
                &row.guild_mode,
                row.response,
                row.answer,
            )?;

            Some(combine(&managed, &subscription))
        })
        .collect())
}
