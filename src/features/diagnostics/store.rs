use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::platform::db::writer::Batched;

pub struct TraceRow {
    pub message: Snowflake,
    pub command: &'static str,
    pub nanos: i64,
    pub success: bool,
    pub failure: Option<String>,
    pub points: serde_json::Value,
}

pub async fn record(pool: &PgPool, batch: &[TraceRow]) -> Result<()> {
    let messages: Vec<i64> = batch.iter().map(|row| row.message as i64).collect();
    let commands: Vec<String> = batch.iter().map(|row| row.command.to_string()).collect();
    let durations: Vec<i64> = batch.iter().map(|row| row.nanos).collect();
    let outcomes: Vec<bool> = batch.iter().map(|row| row.success).collect();
    let failures: Vec<Option<String>> = batch.iter().map(|row| row.failure.clone()).collect();
    let points: Vec<serde_json::Value> = batch.iter().map(|row| row.points.clone()).collect();

    sqlx::query!(
        "INSERT INTO command_traces (message_id, command_name, total_duration_nanos, success, failure, points)
        SELECT * FROM UNNEST($1::bigint[], $2::text[], $3::bigint[], $4::bool[], $5::text[], $6::jsonb[])",
        &messages,
        &commands,
        &durations,
        &outcomes,
        &failures as &[Option<String>],
        &points
    )
    .execute(pool)
    .await
    .ctx("record command traces")?;

    Ok(())
}

pub fn sink(pool: PgPool) -> Batched<TraceRow> {
    Batched::spawn("command trace", move |batch| {
        let pool = pool.clone();

        async move {
            if let Err(failure) = record(&pool, &batch).await {
                tracing::warn!("could not write command traces; err = {failure}");
            }
        }
    })
}

pub struct Timing {
    pub command: String,
    pub nanos: i64,
    pub failure: Option<String>,
    pub points: serde_json::Value,
}

pub async fn of_message(pool: &PgPool, message: Snowflake) -> Result<Option<Timing>> {
    let row = sqlx::query!(
        "SELECT command_name, total_duration_nanos, failure, points
        FROM command_traces WHERE message_id = $1
        ORDER BY created_at DESC LIMIT 1",
        message as i64
    )
    .fetch_optional(pool)
    .await
    .ctx("read command trace")?;

    Ok(row.map(|row| Timing {
        command: row.command_name,
        nanos: row.total_duration_nanos,
        failure: row.failure,
        points: row.points,
    }))
}
