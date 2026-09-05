use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serenity::all::ChannelId;
use sqlx::PgPool;

use crate::command::error::Result;
use crate::domain::Snowflake;
use crate::domain::logtype::LogType;
use crate::features::settings::store;
use crate::platform::cache::Cache;

pub type Routes = Arc<HashMap<&'static str, ChannelId>>;

pub struct Settings {
    routes: Cache<Snowflake, Routes>,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub fn new() -> Self {
        Self {
            routes: Cache::new(4096, Some(Duration::from_secs(900))),
        }
    }

    pub fn forget(&self, guild: Snowflake) {
        self.routes.remove(&guild);
    }

    pub async fn routes(&self, pool: &PgPool, guild: Snowflake) -> Result<Routes> {
        if let Some(known) = self.routes.get(&guild) {
            return Ok(known);
        }

        let loaded: HashMap<&'static str, ChannelId> = store::routes(pool, guild)
            .await?
            .into_iter()
            .map(|(kind, channel)| (kind.as_str(), channel))
            .collect();
        let shared = Arc::new(loaded);

        self.routes.insert(guild, Arc::clone(&shared));

        Ok(shared)
    }

    pub async fn channel_for(
        &self,
        pool: &PgPool,
        guild: Snowflake,
        kind: LogType,
    ) -> Result<Option<ChannelId>> {
        Ok(self.routes(pool, guild).await?.get(kind.as_str()).copied())
    }
}
