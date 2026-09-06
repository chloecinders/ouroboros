use crate::domain::Snowflake;
use crate::platform::ui::embed::{Embed, channel_mention, mention};
use crate::platform::ui::tone::Tone;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Presence {
    pub channel: Option<Snowflake>,
    pub mute: bool,
    pub deaf: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Where {
    Joined(Snowflake),
    Left(Snowflake),
    Moved(Snowflake, Snowflake),
    Stayed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Changed {
    pub place: Where,
    pub mute: Option<bool>,
    pub deaf: Option<bool>,
}

pub fn diff(before: Option<Presence>, after: Presence) -> Changed {
    let was = before.unwrap_or_default();

    let place = match (was.channel, after.channel) {
        (None, Some(joined)) => Where::Joined(joined),
        (Some(left), None) => Where::Left(left),
        (Some(from), Some(to)) if from != to => Where::Moved(from, to),
        _ => Where::Stayed,
    };

    Changed {
        place,
        mute: moved(was.mute, after.mute),
        deaf: moved(was.deaf, after.deaf),
    }
}

fn moved(before: bool, after: bool) -> Option<bool> {
    match before == after {
        true => None,
        false => Some(after),
    }
}

impl Changed {
    pub fn is_empty(&self) -> bool {
        self.place == Where::Stayed && self.mute.is_none() && self.deaf.is_none()
    }
}

pub fn entries(target: Snowflake, name: Option<&str>, changed: &Changed) -> Vec<Embed> {
    let mut written = Vec::new();

    if let Some(embed) = movement(target, name, changed.place) {
        written.push(embed);
    }

    if let Some(embed) = imposed(target, name, changed) {
        written.push(embed);
    }

    written
}

fn movement(target: Snowflake, name: Option<&str>, place: Where) -> Option<Embed> {
    let (title, subtitle, tone) = match place {
        Where::Joined(channel) => (
            "VOICE JOINED",
            format!("Channel: {}", channel_mention(channel)),
            Tone::Success,
        ),
        Where::Left(channel) => (
            "VOICE LEFT",
            format!("Channel: {}", channel_mention(channel)),
            Tone::Danger,
        ),
        Where::Moved(from, to) => (
            "VOICE MOVED",
            format!(
                "Channel: {} -> {}",
                channel_mention(from),
                channel_mention(to)
            ),
            Tone::Warn,
        ),
        Where::Stayed => return None,
    };

    Some(
        Embed::new(title)
            .subtitle(format!("Target: {}", mention(target, name)))
            .subtitle(subtitle)
            .tone(tone),
    )
}

fn imposed(target: Snowflake, name: Option<&str>, changed: &Changed) -> Option<Embed> {
    let mut lines = Vec::new();

    if let Some(muted) = changed.mute {
        lines.push(String::from(match muted {
            true => "server muted",
            false => "server unmuted",
        }));
    }

    if let Some(deafened) = changed.deaf {
        lines.push(String::from(match deafened {
            true => "server deafened",
            false => "server undeafened",
        }));
    }

    if lines.is_empty() {
        return None;
    }

    let title = match (changed.mute, changed.deaf) {
        (Some(true), None) => "SERVER MUTED",
        (Some(false), None) => "SERVER UNMUTED",
        (None, Some(true)) => "SERVER DEAFENED",
        (None, Some(false)) => "SERVER UNDEAFENED",
        _ => "SERVER VOICE UPDATE",
    };

    Some(
        Embed::new(title)
            .subtitle(format!("Target: {}", mention(target, name)))
            .body(lines.join("\n"))
            .tone(Tone::Info),
    )
}
