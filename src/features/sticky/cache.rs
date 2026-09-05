use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::command::error::Result;
use crate::domain::Snowflake;
use crate::features::sticky::store;
use crate::platform::cache::Cache;

pub type Cached = Option<Arc<store::Sticky>>;

pub struct Stickies {
    known: Cache<Snowflake, Cached>,
}

impl Default for Stickies {
    fn default() -> Self {
        Self::new()
    }
}

impl Stickies {
    pub fn new() -> Self {
        Self {
            known: Cache::new(8192, Some(Duration::from_secs(900))),
        }
    }

    pub fn forget(&self, channel: Snowflake) {
        self.known.remove(&channel);
    }

    pub async fn of(&self, pool: &PgPool, channel: Snowflake) -> Result<Cached> {
        if let Some(known) = self.known.get(&channel) {
            return Ok(known);
        }

        let loaded = store::get(pool, channel).await?.map(Arc::new);

        self.known.insert(channel, loaded.clone());

        Ok(loaded)
    }
}
