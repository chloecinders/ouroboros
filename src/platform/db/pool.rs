use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::app::config::Environment;

pub fn min_connections(config: &Environment) -> u32 {
    let ceiling = config.max_connections.unwrap_or(5);

    config.min_connections.unwrap_or(1).min(ceiling)
}

pub async fn connect(config: &Environment) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections.unwrap_or(5))
        .min_connections(min_connections(config))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .connect(&config.database_url)
        .await
}
