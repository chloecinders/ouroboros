use chrono::Duration as ChronoDuration;
use serde::{Deserialize, Serialize};

use crate::domain::Snowflake;
use crate::domain::punishment::PunishmentType;

use super::secs;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Threshold {
    pub count: u32,
    #[serde(with = "secs")]
    pub window: ChronoDuration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Notify {
    #[default]
    Log,
    None,
    Channel(Snowflake),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub punishment_type: Option<PunishmentType>,
    #[serde(with = "secs")]
    pub duration: ChronoDuration,
    pub clear_days: u8,
    pub delete: bool,
    pub notify: Notify,
    pub reason: Option<String>,
}

impl Outcome {
    pub fn severity(&self) -> u8 {
        match self.punishment_type {
            Some(PunishmentType::Ban) => 7,
            Some(PunishmentType::Softban) => 6,
            Some(PunishmentType::Kick) => 5,
            Some(PunishmentType::Mute) => 4,
            Some(PunishmentType::Warn) => 3,
            Some(PunishmentType::Unban) | Some(PunishmentType::Unmute) => 2,
            None if self.delete => 1,
            None => 0,
        }
    }

    pub fn acts(&self) -> bool {
        self.punishment_type.is_some() || self.delete || matches!(self.notify, Notify::Channel(_))
    }
}
