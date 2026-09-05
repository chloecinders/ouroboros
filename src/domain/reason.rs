use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

use crate::platform::text::truncate;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Reason(String);

impl Reason {
    pub fn new(input: &str) -> Self {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Self::default();
        }

        Self(truncate::cap(trimmed, 500))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Reason {
    fn default() -> Self {
        Self(String::from("No reason provided"))
    }
}

impl Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Note(String);

impl Note {
    pub fn new(input: &str) -> Option<Self> {
        let trimmed = input.trim();

        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("clear") {
            return None;
        }

        Some(Self(truncate::cap(trimmed, 125)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
