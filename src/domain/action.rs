use chrono::{DateTime, Duration, Utc};

use crate::domain::Snowflake;
use crate::domain::ids::ActionId;
use crate::domain::punishment::{Punishment, PunishmentState, PunishmentType};
use crate::domain::reason::{Note, Reason};

#[derive(Clone, Debug)]
pub struct Action {
    pub id: ActionId,
    pub verb: PunishmentType,
    pub guild: Snowflake,
    pub target: Snowflake,
    pub actor: Snowflake,
    pub reason: Reason,
    pub note: Option<Note>,
    pub state: PunishmentState,
    pub clear_days: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Action {
    pub fn duration(&self) -> Duration {
        match self.expires_at {
            Some(expiry) => expiry.signed_duration_since(self.created_at),
            None => Duration::zero(),
        }
    }

    pub fn to_punishment(&self) -> Punishment {
        Punishment {
            id: self.id.clone(),
            verb: self.verb,
            guild: self.guild,
            actor: self.actor,
            target: self.target,
            reason: self.reason.clone(),
            note: self.note.clone(),
            duration: self.duration(),
            clear_days: self.clear_days,
            silent: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Amendment {
    Reason,
    Note,
    Duration,
    Never,
}
