pub mod commands;
pub mod store;
pub mod ui;

use std::sync::Arc;

use serenity::all::Message;

use crate::app::App;
use crate::command::registry::Registry;
use crate::domain::Snowflake;
use crate::platform::observe::report::Origin;
use crate::register;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    User(Snowflake),
    Server(Snowflake),
}

impl Scope {
    pub fn columns(self) -> (Option<i64>, Option<i64>) {
        match self {
            Scope::User(user) => (None, Some(user as i64)),
            Scope::Server(guild) => (Some(guild as i64), None),
        }
    }

    pub fn rebuild(guild: Option<i64>, owner: Option<i64>) -> Option<Self> {
        match (guild, owner) {
            (None, Some(user)) => Some(Scope::User(user as Snowflake)),
            (Some(guild), None) => Some(Scope::Server(guild as Snowflake)),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Scope::User(_) => "user snippet",
            Scope::Server(_) => "server snippet",
        }
    }

    pub fn limit(self) -> i64 {
        match self {
            Scope::User(_) => 50,
            Scope::Server(_) => 200,
        }
    }
}

pub fn nameable(name: &str) -> bool {
    !name.is_empty()
        && name.chars().count() <= 32
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

pub async fn line(app: &App, msg: &Message) -> Option<Arc<str>> {
    let prefix = app.prefix();

    let Some(asked) = msg
        .content
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix(prefix))
        .map(str::trim_start)
    else {
        return Some(Arc::from(msg.content.as_str()));
    };

    let guild = msg.guild_id?;
    let (name, extra) = match asked.find(char::is_whitespace) {
        Some(at) => asked.split_at(at),
        None => (asked, ""),
    };

    if name.is_empty() {
        return None;
    }

    let found = match store::resolve(&app.pool, guild.get(), msg.author.id.get(), name).await {
        Ok(found) => found?,
        Err(failure) => {
            app.reporter.record(
                &failure,
                Origin {
                    command: None,
                    guild: Some(guild.get()),
                    channel: Some(msg.channel_id.get()),
                    user: Some(msg.author.id.get()),
                    message: Some(msg.id.get()),
                },
            );

            return None;
        }
    };

    Some(Arc::from(format!("{prefix}{}{extra}", found.body)))
}

pub fn register(registry: &mut Registry) {
    register!(registry, commands::snippet::Snippets);
}
