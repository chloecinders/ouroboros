use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::features::snippets::Scope;

#[derive(Clone, Debug)]
pub struct Snippet {
    pub name: String,
    pub body: String,
    pub scope: Scope,
}

fn rebuild(name: String, body: String, guild: Option<i64>, owner: Option<i64>) -> Option<Snippet> {
    Some(Snippet {
        name,
        body,
        scope: Scope::rebuild(guild, owner)?,
    })
}

pub async fn resolve(
    pool: &PgPool,
    guild: Snowflake,
    author: Snowflake,
    name: &str,
) -> Result<Option<Snippet>> {
    let row = sqlx::query!(
        "SELECT name, body, guild_id, owner_id FROM command_snippets
        WHERE (owner_id = $2 OR guild_id = $1) AND lower(name) = lower($3)
        ORDER BY owner_id NULLS LAST LIMIT 1",
        guild as i64,
        author as i64,
        name
    )
    .fetch_optional(pool)
    .await
    .ctx("resolve snippet")?;

    Ok(row.and_then(|row| rebuild(row.name, row.body, row.guild_id, row.owner_id)))
}

pub async fn find(pool: &PgPool, scope: Scope, name: &str) -> Result<Option<Snippet>> {
    let (guild, owner) = scope.columns();

    let row = sqlx::query!(
        "SELECT name, body, guild_id, owner_id FROM command_snippets
        WHERE guild_id IS NOT DISTINCT FROM $1::bigint
            AND owner_id IS NOT DISTINCT FROM $2::bigint
            AND lower(name) = lower($3)",
        guild,
        owner,
        name
    )
    .fetch_optional(pool)
    .await
    .ctx("read snippet")?;

    Ok(row.and_then(|row| rebuild(row.name, row.body, row.guild_id, row.owner_id)))
}

pub async fn visible(pool: &PgPool, guild: Snowflake, author: Snowflake) -> Result<Vec<Snippet>> {
    let rows = sqlx::query!(
        "SELECT name, body, guild_id, owner_id FROM command_snippets
        WHERE owner_id = $2 OR guild_id = $1
        ORDER BY owner_id NULLS LAST, lower(name)",
        guild as i64,
        author as i64
    )
    .fetch_all(pool)
    .await
    .ctx("list snippets")?;

    Ok(rows
        .into_iter()
        .filter_map(|row| rebuild(row.name, row.body, row.guild_id, row.owner_id))
        .collect())
}

pub async fn count(pool: &PgPool, scope: Scope) -> Result<i64> {
    let (guild, owner) = scope.columns();

    let row = sqlx::query!(
        r#"SELECT count(*) AS "stored!" FROM command_snippets
        WHERE guild_id IS NOT DISTINCT FROM $1::bigint
            AND owner_id IS NOT DISTINCT FROM $2::bigint"#,
        guild,
        owner
    )
    .fetch_one(pool)
    .await
    .ctx("count snippets")?;

    Ok(row.stored)
}

pub async fn save(pool: &PgPool, scope: Scope, name: &str, body: &str) -> Result<()> {
    match scope {
        Scope::User(user) => {
            sqlx::query!(
                "INSERT INTO command_snippets (owner_id, name, body) VALUES ($1, $2, $3)
                ON CONFLICT (owner_id, lower(name)) WHERE owner_id IS NOT NULL DO UPDATE
                SET name = EXCLUDED.name, body = EXCLUDED.body, updated_at = now()",
                user as i64,
                name,
                body
            )
            .execute(pool)
            .await
        }
        Scope::Server(guild) => {
            sqlx::query!(
                "INSERT INTO command_snippets (guild_id, name, body) VALUES ($1, $2, $3)
                ON CONFLICT (guild_id, lower(name)) WHERE guild_id IS NOT NULL DO UPDATE
                SET name = EXCLUDED.name, body = EXCLUDED.body, updated_at = now()",
                guild as i64,
                name,
                body
            )
            .execute(pool)
            .await
        }
    }
    .ctx("save snippet")?;

    Ok(())
}

pub async fn delete(pool: &PgPool, scope: Scope, name: &str) -> Result<bool> {
    let (guild, owner) = scope.columns();

    let done = sqlx::query!(
        "DELETE FROM command_snippets
        WHERE guild_id IS NOT DISTINCT FROM $1::bigint
            AND owner_id IS NOT DISTINCT FROM $2::bigint
            AND lower(name) = lower($3)",
        guild,
        owner,
        name
    )
    .execute(pool)
    .await
    .ctx("delete snippet")?;

    Ok(done.rows_affected() > 0)
}
