use std::sync::Arc;
use std::time::Duration;

use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::command::error::Result;
use crate::domain::Snowflake;
use crate::features::automod::rule::{Mode, Rule, Source};
use crate::features::automod::{managed, store};
use crate::platform::cache::Cache;
use crate::platform::text;

pub type Enabled = Arc<Vec<Rule>>;

pub struct Rules {
    enabled: Cache<Snowflake, Enabled>,
}

impl Default for Rules {
    fn default() -> Self {
        Self::new()
    }
}

impl Rules {
    pub fn new() -> Self {
        Self {
            enabled: Cache::new(4096, Some(Duration::from_secs(900))),
        }
    }

    pub fn forget(&self, guild: Snowflake) {
        self.enabled.remove(&guild);
    }

    pub fn forget_everywhere(&self) {
        self.enabled.clear();
    }

    pub async fn enabled(&self, pool: &PgPool, guild: Snowflake) -> Result<Enabled> {
        if let Some(known) = self.enabled.get(&guild) {
            return Ok(known);
        }

        let mut loaded: Vec<Rule> = store::all(pool, guild)
            .await?
            .into_iter()
            .filter(|rule| rule.mode == Mode::Active)
            .collect();

        loaded.extend(managed::store::enabled(pool, guild).await?);

        let shared = Arc::new(loaded);

        self.enabled.insert(guild, Arc::clone(&shared));

        Ok(shared)
    }
}

pub fn wanted(rules: &[Rule]) -> Vec<Source> {
    let mut out: Vec<Source> = Vec::new();

    for source in rules.iter().flat_map(|rule| rule.body.sources()) {
        if !out.contains(source) {
            out.push(*source);
        }
    }

    out.sort_unstable();
    out
}

pub fn reading(rules: &[Rule], source: Source) -> Vec<&Rule> {
    rules
        .iter()
        .filter(|rule| rule.body.has_source(source))
        .collect()
}

pub fn image_hash(rules: &[Rule]) -> Option<String> {
    let mut parts: Vec<String> = reading(rules, Source::Image)
        .iter()
        .map(|rule| rule.hash())
        .collect();

    if parts.is_empty() {
        return None;
    }

    parts.sort_unstable();
    parts.dedup();

    Some(text::hex(&Sha256::digest(
        format!("r3:{}", parts.join(":")).as_bytes(),
    )))
}
