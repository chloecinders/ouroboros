use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::command::error::Result;
use crate::domain::Snowflake;
use crate::features::permissions::rule::RuleSet;
use crate::features::permissions::store;
use crate::platform::cache::Cache;

pub type Compiled = Arc<RuleSet>;

pub struct Permits {
    sets: Cache<Snowflake, Compiled>,
}

impl Default for Permits {
    fn default() -> Self {
        Self::new()
    }
}

impl Permits {
    pub fn new() -> Self {
        Self {
            sets: Cache::new(4096, Some(Duration::from_secs(900))),
        }
    }

    pub fn forget(&self, guild: Snowflake) {
        self.sets.remove(&guild);
    }

    pub async fn compiled(&self, pool: &PgPool, guild: Snowflake) -> Result<Compiled> {
        if let Some(known) = self.sets.get(&guild) {
            return Ok(known);
        }

        let shared = Arc::new(store::all(pool, guild).await?);

        self.sets.insert(guild, Arc::clone(&shared));

        Ok(shared)
    }
}
