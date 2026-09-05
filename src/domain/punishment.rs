use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::Snowflake;
use crate::domain::ids::ActionId;
use crate::domain::reason::{Note, Reason};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PunishmentType {
    Warn,
    Kick,
    Ban,
    Softban,
    Mute,
    Unban,
    Unmute,
}

impl PunishmentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PunishmentType::Warn => "warn",
            PunishmentType::Kick => "kick",
            PunishmentType::Ban => "ban",
            PunishmentType::Softban => "softban",
            PunishmentType::Mute => "mute",
            PunishmentType::Unban => "unban",
            PunishmentType::Unmute => "unmute",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "warn" => Some(PunishmentType::Warn),
            "kick" => Some(PunishmentType::Kick),
            "ban" => Some(PunishmentType::Ban),
            "softban" => Some(PunishmentType::Softban),
            "mute" => Some(PunishmentType::Mute),
            "unban" => Some(PunishmentType::Unban),
            "unmute" => Some(PunishmentType::Unmute),
            _ => None,
        }
    }

    pub fn headline(&self) -> &'static str {
        match self {
            PunishmentType::Warn => "MEMBER WARNED",
            PunishmentType::Kick => "MEMBER KICKED",
            PunishmentType::Ban => "MEMBER BANNED",
            PunishmentType::Softban => "MEMBER SOFTBANNED",
            PunishmentType::Mute => "MEMBER MUTED",
            PunishmentType::Unban => "MEMBER UNBANNED",
            PunishmentType::Unmute => "MEMBER UNMUTED",
        }
    }

    pub fn shout(&self) -> &'static str {
        match self {
            PunishmentType::Warn => "WARNED",
            PunishmentType::Kick => "KICKED",
            PunishmentType::Ban => "BANNED",
            PunishmentType::Softban => "SOFTBANNED",
            PunishmentType::Mute => "MUTED",
            PunishmentType::Unban => "UNBANNED",
            PunishmentType::Unmute => "UNMUTED",
        }
    }

    pub fn participle(&self) -> &'static str {
        match self {
            PunishmentType::Warn => "warned",
            PunishmentType::Kick => "kicked",
            PunishmentType::Ban => "banned",
            PunishmentType::Softban => "softbanned",
            PunishmentType::Mute => "muted",
            PunishmentType::Unban => "unbanned",
            PunishmentType::Unmute => "unmuted",
        }
    }

    pub fn removes_target(&self) -> bool {
        matches!(
            self,
            PunishmentType::Kick | PunishmentType::Ban | PunishmentType::Softban
        )
    }

    pub fn has_duration(&self) -> bool {
        matches!(self, PunishmentType::Ban | PunishmentType::Mute)
    }

    pub fn dm_timing(&self) -> DmTiming {
        match self.removes_target() {
            true => DmTiming::Before,
            false => DmTiming::After,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DmTiming {
    Before,
    After,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PunishmentState {
    Pending,
    Active,
    Expiring,
    Ended,
    Revoked,
    Lapsed,
    Failed,
}

impl PunishmentState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PunishmentState::Pending => "pending",
            PunishmentState::Active => "active",
            PunishmentState::Expiring => "expiring",
            PunishmentState::Ended => "ended",
            PunishmentState::Revoked => "revoked",
            PunishmentState::Lapsed => "lapsed",
            PunishmentState::Failed => "failed",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pending" => Some(PunishmentState::Pending),
            "active" => Some(PunishmentState::Active),
            "expiring" => Some(PunishmentState::Expiring),
            "ended" => Some(PunishmentState::Ended),
            "revoked" => Some(PunishmentState::Revoked),
            "lapsed" => Some(PunishmentState::Lapsed),
            "failed" => Some(PunishmentState::Failed),
            _ => None,
        }
    }

    pub fn active(&self) -> bool {
        matches!(self, PunishmentState::Active | PunishmentState::Expiring)
    }
}

#[derive(Clone, Debug)]
pub struct Punishment {
    pub id: ActionId,
    pub verb: PunishmentType,
    pub guild: Snowflake,
    pub actor: Snowflake,
    pub target: Snowflake,
    pub reason: Reason,
    pub note: Option<Note>,
    pub duration: Duration,
    pub clear_days: u8,
    pub silent: bool,
}

impl Punishment {
    pub fn new(
        verb: PunishmentType,
        guild: Snowflake,
        actor: Snowflake,
        target: Snowflake,
    ) -> Self {
        Self {
            id: ActionId::generate(),
            verb,
            guild,
            actor,
            target,
            reason: Reason::default(),
            note: None,
            duration: Duration::zero(),
            clear_days: 0,
            silent: false,
        }
    }

    pub fn reason(mut self, reason: Reason) -> Self {
        self.reason = reason;
        self
    }

    pub fn note(mut self, note: Option<Note>) -> Self {
        self.note = note;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn clear_days(mut self, days: u8) -> Self {
        self.clear_days = days.min(7);
        self
    }

    pub fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    pub fn is_permanent(&self) -> bool {
        self.duration.is_zero()
    }

    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        if self.is_permanent() {
            return None;
        }

        Some(Utc::now() + self.duration)
    }

    pub fn timeout_until(&self) -> DateTime<Utc> {
        let ceiling = Utc::now() + Duration::days(27);

        match self.expires_at() {
            Some(expiry) if expiry < ceiling => expiry,
            _ => ceiling,
        }
    }

    pub fn audit_marker(&self) -> String {
        format!(
            "Aegis Managed {}: log id `{}`. Please use Aegis to reverse this to avoid accidental re-application! Reason: {}",
            self.verb.participle(),
            self.id,
            self.reason
        )
    }
}
