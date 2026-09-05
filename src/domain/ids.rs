use std::fmt::{self, Display, Write};

use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

use crate::domain::Snowflake;

pub const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPRSTUVWXYZabcdefghjkmnpqrstuvwxyz123456789";

fn short() -> String {
    let mut rng = rand::rng();
    let mut out = String::with_capacity(6);

    for _ in 0..6 {
        out.push(ALPHABET[rng.random_range(0..ALPHABET.len())] as char);
    }

    out
}

fn uuid4() -> String {
    let mut bytes = [0u8; 16];

    rand::rng().fill_bytes(&mut bytes);

    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let mut out = String::with_capacity(36);

    for (at, byte) in bytes.iter().enumerate() {
        if matches!(at, 4 | 6 | 8 | 10) {
            out.push('-');
        }

        let _ = write!(out, "{byte:02x}");
    }

    out
}

macro_rules! opaque_id {
    ($name:ident, $generate:path) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn generate() -> Self {
                Self($generate())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            #[allow(dead_code, reason = "not every id is unwrapped, but each one may be")]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl From<String> for $name {
            fn from(raw: String) -> Self {
                Self(raw)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

opaque_id!(ActionId, short);
opaque_id!(RuleId, short);
opaque_id!(TranscriptId, uuid4);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageId(Snowflake);

impl MessageId {
    pub fn new(id: Snowflake) -> Self {
        Self(id)
    }

    pub fn get(&self) -> Snowflake {
        self.0
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
