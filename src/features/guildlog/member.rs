use chrono::{DateTime, Utc};

use crate::domain::Snowflake;
use crate::features::guildlog::attribution::Attribution;
use crate::platform::ui::embed::{Embed, code, mention, role_mention};
use crate::platform::ui::tone::Tone;

pub type Moved<T> = Option<(Option<T>, Option<T>)>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub nick: Option<String>,
    pub roles: Vec<Snowflake>,
    pub timeout: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    Nick,
    Gained(Snowflake),
    Lost(Snowflake),
    Timeout,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Changed {
    pub nick: Moved<String>,
    pub gained: Vec<Snowflake>,
    pub lost: Vec<Snowflake>,
    pub timeout: Moved<DateTime<Utc>>,
}

pub fn diff(before: &Snapshot, after: &Snapshot) -> Changed {
    let gained = missing_from(&after.roles, &before.roles);
    let lost = missing_from(&before.roles, &after.roles);

    Changed {
        nick: moved(&before.nick, &after.nick),
        gained,
        lost,
        timeout: match before.timeout == after.timeout {
            true => None,
            false => Some((before.timeout, after.timeout)),
        },
    }
}

fn missing_from(roles: &[Snowflake], other: &[Snowflake]) -> Vec<Snowflake> {
    let mut found: Vec<Snowflake> = roles
        .iter()
        .filter(|role| !other.contains(role))
        .copied()
        .collect();

    found.sort_unstable();
    found
}

fn moved<T: Clone + PartialEq>(before: &Option<T>, after: &Option<T>) -> Moved<T> {
    match before == after {
        true => None,
        false => Some((before.clone(), after.clone())),
    }
}

impl Changed {
    pub fn is_empty(&self) -> bool {
        self.nick.is_none()
            && self.gained.is_empty()
            && self.lost.is_empty()
            && self.timeout.is_none()
    }

    pub fn parts(&self) -> Vec<Part> {
        let mut parts = Vec::new();

        if self.nick.is_some() {
            parts.push(Part::Nick);
        }

        parts.extend(self.gained.iter().map(|role| Part::Gained(*role)));
        parts.extend(self.lost.iter().map(|role| Part::Lost(*role)));

        if self.timeout.is_some() {
            parts.push(Part::Timeout);
        }

        parts
    }
}

pub fn entry(
    target: Snowflake,
    name: Option<&str>,
    changed: &Changed,
    actor: Attribution,
    actor_name: Option<&str>,
    reason: Option<&str>,
    bot: Snowflake,
) -> Embed {
    let entry = Embed::new("MEMBER UPDATE")
        .subtitle(format!("Target: {}", mention(target, name)))
        .maybe_subtitle(actor.line(bot, actor_name))
        .lead(lines(changed).join("\n"))
        .tone(Tone::Info);

    match reason {
        Some(given) => entry.quote(given),
        None => entry,
    }
}

pub fn joined(
    target: Snowflake,
    name: Option<&str>,
    created: DateTime<Utc>,
    history: i64,
) -> Embed {
    let mut entry = Embed::new("MEMBER JOINED")
        .subtitle(format!("Target: {}", mention(target, name)))
        .subtitle(format!("ID: `{target}`"))
        .body(format!(
            "account created <t:{0}:R> (<t:{0}:f>)",
            created.timestamp()
        ))
        .tone(Tone::Success);

    if history > 0 {
        entry = entry.footnote(format!("{history} previous log entries"));
    }

    entry
}

pub fn left(target: Snowflake, name: Option<&str>) -> Embed {
    Embed::new("MEMBER LEFT")
        .subtitle(format!("Target: {}", mention(target, name)))
        .subtitle(format!("ID: `{target}`"))
        .tone(Tone::Danger)
}

fn lines(changed: &Changed) -> Vec<String> {
    let mut written = Vec::new();

    if let Some((before, after)) = &changed.nick {
        written.push(format!(
            "Nickname: {} -> {}",
            nickname(before.as_deref()),
            nickname(after.as_deref())
        ));
    }

    if !changed.gained.is_empty() {
        written.push(format!("Roles gained: {}", roles(&changed.gained)));
    }

    if !changed.lost.is_empty() {
        written.push(format!("Roles lost: {}", roles(&changed.lost)));
    }

    if let Some((before, after)) = &changed.timeout {
        written.push(timeout(*before, *after));
    }

    match written.is_empty() {
        true => vec![String::from("no recorded changes")],
        false => written,
    }
}

fn nickname(nick: Option<&str>) -> String {
    match nick {
        Some(nick) => code(nick),
        None => String::from("(none)"),
    }
}

fn roles(ids: &[Snowflake]) -> String {
    ids.iter()
        .map(|role| role_mention(*role))
        .collect::<Vec<_>>()
        .join(" ")
}

fn timeout(before: Option<DateTime<Utc>>, after: Option<DateTime<Utc>>) -> String {
    match after {
        Some(until) if until > Utc::now() => {
            format!("Timed out until <t:{}:f>", until.timestamp())
        }
        _ => match before.is_some_and(|until| until > Utc::now()) {
            true => String::from("Timeout removed"),
            false => String::from("Timeout expired"),
        },
    }
}
