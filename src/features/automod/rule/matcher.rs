use regex::Regex;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::platform::text::fuzzy;

#[derive(Clone, Debug)]
pub struct Pattern(Regex);

impl Pattern {
    pub fn new(pattern: &str) -> Option<Self> {
        Regex::new(pattern).ok().map(Pattern)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.0.is_match(text)
    }
}

impl Serialize for Pattern {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Pattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let pattern = String::deserialize(deserializer)?;

        Pattern::new(&pattern).ok_or_else(|| D::Error::custom("unparseable regex"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Matcher {
    Literal { text: String },
    Regex { pattern: Pattern },
}

impl Matcher {
    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        if raw.len() >= 2
            && let Some(body) = raw
                .strip_prefix('/')
                .and_then(|rest| rest.strip_suffix('/'))
        {
            return Pattern::new(body)
                .map(|pattern| Matcher::Regex { pattern })
                .ok_or("regex does not compile");
        }

        match raw.is_empty() {
            true => Err("empty pattern"),
            false => Ok(Matcher::Literal {
                text: raw.to_string(),
            }),
        }
    }

    pub fn test(&self, read: &fuzzy::Haystack) -> bool {
        match self {
            Matcher::Literal { text: needle } => fuzzy::contains_loose(needle, read, 0.95),
            Matcher::Regex { pattern } => pattern.is_match(read.text()),
        }
    }

    pub fn render(&self) -> String {
        match self {
            Matcher::Literal { text } => format!("{text:?}"),
            Matcher::Regex { pattern } => format!("/{}/", pattern.as_str()),
        }
    }
}
