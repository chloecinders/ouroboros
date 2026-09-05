use std::time::Duration;

use crate::domain::Snowflake;
use crate::platform::cache::Cache;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub fingerprint: String,
    pub text: String,
}

pub struct Readings {
    seen: Cache<Snowflake, Vec<Reading>>,
}

impl Default for Readings {
    fn default() -> Self {
        Self::new()
    }
}

impl Readings {
    pub fn new() -> Self {
        Self {
            seen: Cache::new(256, Some(Duration::from_secs(3600))),
        }
    }

    pub fn remember(&self, message: Snowflake, reading: Reading) {
        let mut readings = self.seen.get(&message).unwrap_or_default();

        readings.push(reading);
        self.seen.insert(message, readings);
    }

    pub fn forget_all(&self) {
        self.seen.clear();
    }

    pub fn sweep(&self) {
        self.seen.sweep();
    }
}
