use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::domain::action::Action;
use crate::domain::ids::ActionId;
use crate::features::references::{Attached, Captured, Origin};

pub async fn save(pool: &PgPool, action: &ActionId, captured: &Captured) -> Result<()> {
    sqlx::query!(
        "INSERT INTO action_refs
            (action_id, origin, ref_message_id, ref_channel_id, ref_author_id, ref_content, image_url)
        VALUES ($1, $2::reference_origin, $3, $4, $5, $6, $7)
        ON CONFLICT (action_id) DO UPDATE SET origin = EXCLUDED.origin,
            ref_message_id = EXCLUDED.ref_message_id, ref_channel_id = EXCLUDED.ref_channel_id,
            ref_author_id = EXCLUDED.ref_author_id, ref_content = EXCLUDED.ref_content,
        image_url = EXCLUDED.image_url",
        action.as_str(),
        captured.origin.as_str() as _,
        captured.message as i64,
        captured.channel as i64,
        captured.author as i64,
        captured.content.as_deref().map(|body| body.as_bytes()),
        captured.image_url.as_deref()
    )
    .execute(pool)
    .await
    .ctx("save reference")?;

    Ok(())
}

pub async fn archive(pool: &PgPool, action: &ActionId) -> Result<()> {
    sqlx::query!(
        "UPDATE action_refs SET origin = 'archived'::reference_origin WHERE action_id = $1",
        action.as_str()
    )
    .execute(pool)
    .await
    .ctx("archive reference")?;

    Ok(())
}

pub async fn attached(
    pool: &PgPool,
    guild: Snowflake,
    actions: &[Action],
) -> Result<Vec<Attached>> {
    if actions.is_empty() {
        return Ok(Vec::new());
    }

    let ids: Vec<String> = actions
        .iter()
        .map(|action| action.id.as_str().to_string())
        .collect();

    let rows = sqlx::query!(
        r#"SELECT refs.action_id, refs.ref_content IS NOT NULL AS "content!",
            refs.image_url IS NOT NULL AS "image!"
        FROM action_refs refs
        JOIN actions owner ON owner.id = refs.action_id
        WHERE owner.guild_id = $1 AND refs.action_id::text = ANY($2::text[])"#,
        guild as i64,
        &ids
    )
    .fetch_all(pool)
    .await
    .ctx("read attached references")?;

    Ok(actions
        .iter()
        .map(|action| {
            rows.iter()
                .find(|row| row.action_id == action.id.as_str())
                .map(|row| Attached {
                    content: row.content,
                    image: row.image,
                })
                .unwrap_or_default()
        })
        .collect())
}

pub async fn load(pool: &PgPool, guild: Snowflake, action: &ActionId) -> Result<Option<Captured>> {
    let row = sqlx::query!(
        r#"SELECT refs.origin::text AS "origin!", refs.ref_message_id, refs.ref_channel_id,
            refs.ref_author_id, refs.ref_content, refs.image_url
        FROM action_refs refs
        JOIN actions owner ON owner.id = refs.action_id
        WHERE refs.action_id = $1 AND owner.guild_id = $2"#,
        action.as_str(),
        guild as i64
    )
    .fetch_optional(pool)
    .await
    .ctx("load reference")?;

    Ok(row.and_then(|row| {
        Some(Captured {
            origin: Origin::parse(&row.origin)?,
            channel: row.ref_channel_id? as Snowflake,
            message: row.ref_message_id? as Snowflake,
            author: row.ref_author_id.unwrap_or_default() as Snowflake,
            content: row
                .ref_content
                .and_then(|body| String::from_utf8(body).ok()),
            image_url: row.image_url,
        })
    }))
}
