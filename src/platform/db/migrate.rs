use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};
use tracing::info;

static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("_sqlx_migrations has a migration {0}, which is not in this build")]
    AheadLedger(i64),
    #[error("could not read _sqlx_migrations: {0}")]
    Ledger(#[from] sqlx::Error),
    #[error("migration failed: {0}")]
    Failed(#[from] MigrateError),
}

pub async fn run(pool: &PgPool) -> Result<(), Failure> {
    discard(pool).await?;

    MIGRATOR.run(pool).await.map_err(Failure::Failed)
}

async fn discard(pool: &PgPool) -> Result<(), Failure> {
    let Some(earliest) = MIGRATOR.iter().map(|migration| migration.version).min() else {
        return Ok(());
    };

    let ledger: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations')::text")
            .fetch_one(pool)
            .await?;

    if ledger.is_none() {
        return Ok(());
    }

    let recorded: Vec<i64> = sqlx::query_scalar("SELECT version FROM public._sqlx_migrations")
        .fetch_all(pool)
        .await?;

    if let Some(version) = recorded
        .iter()
        .find(|version| **version >= earliest && !MIGRATOR.version_exists(**version))
    {
        return Err(Failure::AheadLedger(*version));
    }

    let cleared = sqlx::query("DELETE FROM public._sqlx_migrations WHERE version < $1")
        .bind(earliest)
        .execute(pool)
        .await?
        .rows_affected();

    if cleared > 0 {
        info!("discarded {cleared} pre-rewrite migration records");
    }

    Ok(())
}
