pub mod apply;
pub mod download;

use std::path::Path;
use std::sync::Arc;

use serenity::all::{ChannelId, Http, MessageId};
use tracing::{error, info, warn};

use crate::platform::ui::embed::Embed;
use crate::platform::ui::reply;
use crate::platform::ui::tone::Tone;

pub const BINARY: &str = env!("CARGO_PKG_NAME");

pub fn binary_name() -> String {
    match cfg!(windows) {
        true => format!("{BINARY}.exe"),
        false => String::from(BINARY),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handoff {
    pub channel: u64,
    pub message: u64,
    pub sha: Option<String>,
}

impl Handoff {
    pub fn new(channel: u64, message: u64, sha: impl Into<String>) -> Self {
        Self {
            channel,
            message,
            sha: Some(sha.into()),
        }
    }

    pub fn encode(&self) -> String {
        match &self.sha {
            Some(sha) => format!("{}:{}:{sha}", self.channel, self.message),
            None => format!("{}:{}", self.channel, self.message),
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        let mut parts = raw.trim().split(':');
        let channel = parts.next()?.parse().ok()?;
        let message = parts.next()?.parse().ok()?;
        let sha = parts
            .next()
            .map(str::trim)
            .filter(|sha| !sha.is_empty())
            .map(String::from);

        match parts.next() {
            Some(_) => None,
            None => Some(Self {
                channel,
                message,
                sha,
            }),
        }
    }

    pub fn flag(&self, name: &str) -> String {
        format!("{name}={}", self.encode())
    }

    pub fn from_flag(arg: &str, name: &str) -> Option<Self> {
        let (flag, payload) = arg.split_once('=')?;

        match flag == name {
            true => Self::parse(payload),
            false => None,
        }
    }

    pub fn short(&self) -> Option<&str> {
        self.sha.as_deref().map(|sha| &sha[..sha.len().min(7)])
    }
}

pub fn find(args: impl IntoIterator<Item = String>, name: &str) -> Option<Handoff> {
    args.into_iter()
        .find_map(|arg| Handoff::from_flag(&arg, name))
}

pub fn staged_name() -> String {
    format!("new_{}", binary_name())
}

pub fn is_stale(name: &str) -> bool {
    let name = name.to_ascii_lowercase();

    name.starts_with("new_") && name.contains(&BINARY.to_ascii_lowercase())
}

pub fn cleanup(directory: &Path) {
    let running = std::env::current_exe().ok().and_then(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
    });

    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !path.is_file() || !is_stale(name) || running.as_deref() == Some(name) {
            continue;
        }

        match std::fs::remove_file(&path) {
            Ok(()) => info!("removed the staged build {name}"),
            Err(failure) => warn!("could not remove the staged build {name}; err = {failure}"),
        }
    }
}

pub fn intercept() -> bool {
    if cfg!(windows)
        && let Some(handoff) = find(std::env::args(), "--update")
    {
        info!("installing the staged build over {}", binary_name());

        if let Err(failure) = apply::relay(&handoff) {
            error!("the staged build could not install itself; err = {failure}");
        }

        return true;
    }

    cleanup(Path::new("."));

    false
}

pub fn awaiting(directory: &Path) -> Option<Handoff> {
    let args: Vec<String> = std::env::args().collect();

    if let Some(handoff) = find(args.iter().cloned(), "--id").or_else(|| find(args, "--update")) {
        return Some(handoff);
    }

    let note = directory.join("update.txt");
    let raw = std::fs::read_to_string(&note).ok()?;

    let _ = std::fs::remove_file(&note);

    Handoff::parse(&raw)
}

pub fn installed(handoff: &Handoff) -> Embed {
    Embed::new("UPDATE COMPLETE")
        .maybe_subtitle(handoff.short().map(|sha| format!("Commit: `{sha}`")))
        .tone(Tone::Success)
}

pub async fn report(http: Arc<Http>) {
    let Some(handoff) = awaiting(Path::new(".")) else {
        return;
    };

    info!("reporting a completed update to {}", handoff.channel);

    let sent = ChannelId::new(handoff.channel)
        .send_message(
            &http,
            reply::plain(&installed(&handoff)).reference_message((
                ChannelId::new(handoff.channel),
                MessageId::new(handoff.message),
            )),
        )
        .await;

    if let Err(failure) = sent {
        warn!("could not report the completed update; err = {failure}");
    }
}
