use chrono::{Datelike, Duration, Utc};
use sqlx::PgPool;

use crate::command::error::{Ctx, Result};

pub async fn ensure(pool: &PgPool) -> Result<()> {
    let now = Utc::now();
    let month_start = now.year() * 12 + now.month() as i32 - 1;

    for table in ["messages", "message_edits"] {
        let existing = attached(pool, table).await?;

        for offset in 0..=3 {
            let months = month_start + offset;
            let (year, month) = (months.div_euclid(12), months.rem_euclid(12) as u32 + 1);
            let next = months + 1;
            let (next_year, next_month) = (next.div_euclid(12), next.rem_euclid(12) as u32 + 1);

            let name = format!("{table}_{year:04}{month:02}");

            if existing.iter().any(|partition| partition == &name) {
                continue;
            }

            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS public.{name} PARTITION OF public.{table}
                FOR VALUES FROM ('{year:04}-{month:02}-01') TO ('{next_year:04}-{next_month:02}-01')"
            ))
            .execute(pool)
            .await
            .ctx("create message partition")?;
        }
    }

    Ok(())
}

pub fn partition_month(table: &str, name: &str) -> Option<(i32, u32)> {
    let suffix = name.strip_prefix(table)?.strip_prefix('_')?;

    if suffix.len() != 6 || !suffix.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let year = suffix[..4].parse().ok()?;
    let month = suffix[4..].parse().ok()?;

    (1..=12).contains(&month).then_some((year, month))
}

pub async fn prune(pool: &PgPool) -> Result<u64> {
    let now = Utc::now();
    let mut removed = 0;
    let cutoff = now.year() * 12 + now.month() as i32 - 6;

    for table in ["messages", "message_edits"] {
        for name in attached(pool, table).await? {
            let Some(month) = partition_month(table, &name) else {
                continue;
            };

            if month.0 * 12 + month.1 as i32 > cutoff {
                continue;
            }

            rescue(pool, table, &name).await?;

            sqlx::query(&format!("DROP TABLE IF EXISTS public.{name}"))
                .execute(pool)
                .await
                .ctx("drop message partition")?;

            removed += 1;
        }
    }

    Ok(removed)
}

async fn attached(pool: &PgPool, table: &str) -> Result<Vec<String>> {
    let names = sqlx::query_scalar!(
        "SELECT child.relname AS \"name!\" FROM pg_inherits
        JOIN pg_class parent ON parent.oid = pg_inherits.inhparent
        JOIN pg_class child ON child.oid = pg_inherits.inhrelid
        WHERE parent.relname = $1",
        table
    )
    .fetch_all(pool)
    .await
    .ctx("list message partitions")?;

    Ok(names)
}

async fn rescue(pool: &PgPool, table: &str, name: &str) -> Result<()> {
    if table != "messages" {
        return Ok(());
    }

    let mut tx = pool.begin().await.ctx("begin partition rescue")?;

    sqlx::query(&format!(
        "ALTER TABLE public.{table} DETACH PARTITION public.{name}"
    ))
    .execute(&mut *tx)
    .await
    .ctx("detach message partition")?;

    sqlx::query(&format!(
        "INSERT INTO public.{table}
        SELECT stale.* FROM public.{name} AS stale
        WHERE EXISTS (
            SELECT 1 FROM public.transcript_messages pinned
            WHERE pinned.message_id = stale.message_id
        )
        ON CONFLICT DO NOTHING"
    ))
    .execute(&mut *tx)
    .await
    .ctx("rescue pinned messages")?;

    tx.commit().await.ctx("finish partition rescue")?;

    Ok(())
}

pub async fn sweep(pool: &PgPool) -> Result<u64> {
    let traces = sqlx::query!(
        "DELETE FROM command_traces WHERE created_at < $1",
        Utc::now() - Duration::days(14)
    )
    .execute(pool)
    .await
    .ctx("sweep command traces")?;

    let evaluations = sqlx::query!(
        "DELETE FROM ocr_image_evaluations WHERE last_seen_at < $1",
        Utc::now() - Duration::days(90)
    )
    .execute(pool)
    .await
    .ctx("sweep image evaluations")?;

    Ok(traces.rows_affected() + evaluations.rows_affected())
}
