use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use regex::Regex;
use sha2::{Digest, Sha256};

use crate::{SQL, database::ActionType, utils::consume_pgsql_error};

const OCR_RESULT_CACHE_MAX: usize = 1000;

/// Stores the OCR output for a single message attachment, plus whether it matched a rule.
#[derive(Clone, Debug)]
pub struct OcrDebugEntry {
    /// The raw OCR text extracted from the image.
    pub text: String,
    /// If the text matched a rule: (rule name, rule id, matched pattern).
    pub matched: Option<(String, String, String)>,
}

/// A small bounded FIFO cache that maps message_id → Vec of per-attachment debug entries.
pub struct OcrResultCache {
    entries: std::collections::HashMap<u64, Vec<OcrDebugEntry>>,
    order: std::collections::VecDeque<u64>,
}

impl OcrResultCache {
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            order: std::collections::VecDeque::new(),
        }
    }

    pub fn insert(&mut self, message_id: u64, entries: Vec<OcrDebugEntry>) {
        if !self.entries.contains_key(&message_id) {
            if self.entries.len() >= OCR_RESULT_CACHE_MAX {
                if let Some(old) = self.order.pop_front() {
                    self.entries.remove(&old);
                }
            }
            self.order.push_back(message_id);
        }
        self.entries.insert(message_id, entries);
    }

    pub fn get(&self, message_id: u64) -> Option<&Vec<OcrDebugEntry>> {
        self.entries.get(&message_id)
    }
}



pub struct RuleCache {
    ocr: Vec<Rule>,
    recent_triggers: HashMap<(String, u64), Instant>,
}

impl RuleCache {
    pub fn new() -> Self {
        Self {
            ocr: Vec::new(),
            recent_triggers: HashMap::new(),
        }
    }

    pub fn check_debounce(&mut self, rule_id: String, user_id: u64) -> bool {
        let key = (rule_id, user_id);
        if let Some(last_triggered) = self.recent_triggers.get(&key) {
            if last_triggered.elapsed() < Duration::from_secs(15) {
                return false;
            }
        }

        if self.recent_triggers.len() > 1000 {
            self.recent_triggers
                .retain(|_, v| v.elapsed() < Duration::from_secs(15));
        }

        self.recent_triggers.insert(key, Instant::now());
        true
    }

    pub fn insert_ocr(&mut self, rule: Rule) {
        self.ocr.push(rule);
    }

    pub fn remove(&mut self, id: &str) {
        self.ocr.retain(|r| r.id != id);
    }

    pub fn get_by_id(&self, id: &str) -> Option<&Rule> {
        self.ocr.iter().find(|r| r.id == id)
    }

    pub fn get_active_rules(&self, guild_id: u64) -> Vec<Rule> {
        self.ocr.iter().filter(|r| r.guild_id == guild_id).cloned().collect()
    }

    pub fn has_ocr_rules(&self, guild_id: u64) -> bool {
        self.ocr.iter().any(|r| r.guild_id == guild_id)
    }

    pub fn matches(&self, guild_id: u64, input: String) -> Option<Rule> {
        for rule in &self.ocr {
            if rule.guild_id == guild_id && rule.matches(&input) {
                return Some(rule.clone());
            }
        }
        None
    }

    pub async fn populate_from_db(&mut self) {
        let res = match sqlx::query(
            "
            SELECT
                id,
                name,
                guild_id,
                type,
                rule,
                is_regex,
                reason,
                punishment_type,
                duration,
                silent,
                day_clear_amount,
                log_channel_id
            FROM automod_rules;
        ",
        )
        .fetch_all(&*SQL)
        .await
        {
            Ok(d) => d,
            Err(err) => {
                consume_pgsql_error("POPULATE RULE CACHE".into(), err);
                return;
            }
        };

        use sqlx::Row;
        res.into_iter().for_each(|record| {
            let punishment_type: ActionType = record.get("punishment_type");
            let punish = match punishment_type {
                ActionType::Softban => Punishment::Softban {
                    reason: record.get("reason"),
                    day_clear_amount: record
                        .get::<Option<i16>, _>("day_clear_amount")
                        .unwrap_or(0) as u8,
                    silent: record.get::<Option<bool>, _>("silent").unwrap_or(false),
                },
                ActionType::Ban => Punishment::Ban {
                    reason: record.get("reason"),
                    day_clear_amount: record
                        .get::<Option<i16>, _>("day_clear_amount")
                        .unwrap_or(0) as u8,
                    duration: record.get::<Option<i64>, _>("duration").unwrap_or(0) as u64,
                    silent: record.get::<Option<bool>, _>("silent").unwrap_or(false),
                },
                ActionType::Kick => Punishment::Kick {
                    reason: record.get("reason"),
                    silent: record.get::<Option<bool>, _>("silent").unwrap_or(false),
                },
                ActionType::Mute => Punishment::Mute {
                    reason: record.get("reason"),
                    duration: record.get::<Option<i64>, _>("duration").unwrap_or(0) as u64,
                    silent: record.get::<Option<bool>, _>("silent").unwrap_or(false),
                },
                ActionType::Log => Punishment::Log {
                    reason: record.get("reason"),
                    channel_id: record.get::<Option<i64>, _>("log_channel_id").unwrap_or(0) as u64,
                },
                _ => Punishment::Warn {
                    reason: record.get("reason"),
                    silent: record.get::<Option<bool>, _>("silent").unwrap_or(false),
                },
            };

            let rule = Rule {
                name: record.get("name"),
                id: record.get("id"),
                pattern: record.get("rule"),
                is_regex: record.get("is_regex"),
                guild_id: record.get::<i64, _>("guild_id") as u64,
                punishment: punish,
            };

            match record.get::<String, _>("type").as_str() {
                "ocr" => self.ocr.push(rule),
                _ => {}
            };
        });
    }

    pub fn byte_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.ocr.capacity() * std::mem::size_of::<Rule>()
            + self
                .ocr
                .iter()
                .map(|r| r.byte_footprint() - std::mem::size_of::<Rule>())
                .sum::<usize>()
            + self.recent_triggers.capacity()
                * std::mem::size_of::<((String, u64), Instant)>()
    }
}

pub async fn db_check_image_evaluations(image_hash: &str) -> HashMap<String, bool> {
    let results = sqlx::query(
        "SELECT rule_hash, is_match FROM ocr_image_hashes \
         WHERE image_hash = $1",
    )
    .bind(image_hash)
    .fetch_all(&*SQL)
    .await;

    let mut map = HashMap::new();
    if let Ok(rows) = results {
        use sqlx::Row;
        for row in rows {
            map.insert(row.get("rule_hash"), row.get("is_match"));
        }
    }
    map
}

pub async fn db_record_image_evaluations(evaluations: &[(String, String, bool)]) {
    if evaluations.is_empty() {
        return;
    }

    let mut image_hashes = Vec::new();
    let mut rule_hashes = Vec::new();
    let mut is_matches = Vec::new();

    for (ih, rh, im) in evaluations {
        image_hashes.push(ih.clone());
        rule_hashes.push(rh.clone());
        is_matches.push(*im);
    }

    let _ = sqlx::query(
        "INSERT INTO ocr_image_hashes (image_hash, rule_hash, is_match) \
         SELECT * FROM UNNEST($1::char(64)[], $2::char(64)[], $3::boolean[]) \
         ON CONFLICT (image_hash, rule_hash) DO NOTHING",
    )
    .bind(&image_hashes)
    .bind(&rule_hashes)
    .bind(&is_matches)
    .execute(&*SQL)
    .await;
}

#[derive(Clone, Debug)]
pub enum Punishment {
    Warn {
        reason: String,
        silent: bool,
    },
    Kick {
        reason: String,
        silent: bool,
    },
    Ban {
        reason: String,
        day_clear_amount: u8,
        duration: u64,
        silent: bool,
    },
    Softban {
        reason: String,
        day_clear_amount: u8,
        silent: bool,
    },
    Mute {
        reason: String,
        duration: u64,
        silent: bool,
    },
    Log {
        reason: String,
        channel_id: u64,
    },
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub name: String,
    pub id: String,
    pub pattern: String,
    pub is_regex: bool,
    pub guild_id: u64,
    pub punishment: Punishment,
}

impl Rule {
    pub fn rule_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}_{}", self.is_regex, self.pattern).as_bytes());
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }

    pub fn matches(&self, input: &str) -> bool {
        if self.is_regex {
            Regex::new(&self.pattern)
                .map(|re| re.is_match(input))
                .unwrap_or(false)
        } else {
            fuzzy_substring_match(&self.pattern, input, 0.95)
        }
    }

    pub fn byte_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.name.capacity()
            + self.id.capacity()
            + self.pattern.capacity()
            + match &self.punishment {
                Punishment::Warn { reason, .. }
                | Punishment::Kick { reason, .. }
                | Punishment::Ban { reason, .. }
                | Punishment::Softban { reason, .. }
                | Punishment::Mute { reason, .. }
                | Punishment::Log { reason, .. } => reason.capacity(),
            }
    }
}

fn fuzzy_substring_match(rule: &str, input: &str, threshold: f64) -> bool {
    if input.contains(rule) {
        return true;
    }
    let rule_lower = rule.to_lowercase();
    let input_lower = input.to_lowercase();
    if input_lower.contains(&rule_lower) {
        return true;
    }

    let m = rule_lower.chars().count();
    let n = input_lower.chars().count();
    if m == 0 {
        return true;
    }
    if n == 0 || m > n {
        return false;
    }

    let max_errors = ((1.0 - threshold) * m as f64).ceil() as usize;

    let r_chars: Vec<char> = rule_lower.chars().collect();
    let i_chars: Vec<char> = input_lower.chars().collect();

    let mut dp = vec![0; n + 1];

    for i in 1..=m {
        let mut prev = dp[0];
        dp[0] = i;
        for j in 1..=n {
            let temp = dp[j];
            let cost = if r_chars[i - 1] == i_chars[j - 1] {
                0
            } else {
                1
            };
            dp[j] = std::cmp::min(std::cmp::min(dp[j] + 1, dp[j - 1] + 1), prev + cost);
            prev = temp;
        }
    }

    let min_dist = *dp.iter().skip(1).min().unwrap_or(&m);
    min_dist <= max_errors
}
