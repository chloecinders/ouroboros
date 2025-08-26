use std::collections::HashMap;

use serde::Serialize;
use sqlx::query;

use crate::{utils::AnyError, SQL};

#[derive(Debug, Serialize, Clone, Default)]
pub struct GuildSettings {
    inner: HashMap<u64, Settings>,
    invalid: bool,
}

impl GuildSettings {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            invalid: true
        }
    }

    pub fn invalidate(&mut self) {
        self.invalid = true;
    }

    pub async fn get(&mut self, guild: u64) -> Result<Settings, AnyError> {
        if self.invalid {
            let new_data = self.fetch_data().await?;
            self.inner = new_data;
            self.invalid = false;
        }

        match self.inner.get(&guild) {
            Some(s) => Ok(s.clone()),
            None => Err(AnyError::new("guild_not_found"))
        }
    }

    async fn fetch_data(&self) -> Result<HashMap<u64, Settings>, AnyError> {
        if let Ok(data) = query!("SELECT * FROM guild_settings").fetch_all(SQL.get().unwrap()).await {
            let mut map: HashMap<u64, Settings> = HashMap::new();

            data.into_iter().for_each(|record| {
                map.insert(
                    record.guild_id as u64,
                    Settings {
                        log: SettingsLog {
                            channel: record.log_channel.map(|n| n as u64)
                        }
                    }
                );
            });

            Ok(map)
        } else {
            Err(AnyError::new("database_faild"))
        }
    }
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct Settings {
    pub log: SettingsLog
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct SettingsLog {
    pub channel: Option<u64>
}
