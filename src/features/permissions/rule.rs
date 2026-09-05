use serde::{Deserialize, Serialize};

use crate::command::Category;
use crate::domain::Snowflake;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Channel,
    Role,
    Member,
}

impl Scope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Channel => "channel",
            Scope::Role => "role",
            Scope::Member => "member",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "channel" => Some(Scope::Channel),
            "role" => Some(Scope::Role),
            "member" => Some(Scope::Member),
            _ => None,
        }
    }
}

pub fn subject(raw: &str) -> Option<(Scope, Snowflake)> {
    if let Some(id) = raw.strip_prefix("role:") {
        return id.parse().ok().map(|id| (Scope::Role, id));
    }

    if let Some(id) = raw.strip_prefix("channel:") {
        return id.parse().ok().map(|id| (Scope::Channel, id));
    }

    if let Some(id) = raw
        .strip_prefix("member:")
        .or_else(|| raw.strip_prefix("user:"))
    {
        return id.parse().ok().map(|id| (Scope::Member, id));
    }

    let inner = raw.strip_prefix('<')?.strip_suffix('>')?;

    if let Some(id) = inner.strip_prefix("@&") {
        return id.parse().ok().map(|id| (Scope::Role, id));
    }

    if let Some(id) = inner.strip_prefix('#') {
        return id.parse().ok().map(|id| (Scope::Channel, id));
    }

    inner
        .strip_prefix('@')
        .map(|id| id.trim_start_matches('!'))
        .and_then(|id| id.parse().ok())
        .map(|id| (Scope::Member, id))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    Allow,
    Deny,
}

impl Effect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Effect::Allow => "allow",
            Effect::Deny => "deny",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "allow" => Some(Effect::Allow),
            "deny" => Some(Effect::Deny),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Command(String),
    Category(Category),
    Everything,
}

impl Target {
    pub fn parse(raw: &str) -> Self {
        if raw == "*" {
            return Target::Everything;
        }

        match raw.strip_prefix('@').and_then(category) {
            Some(found) => Target::Category(found),
            None => Target::Command(raw.to_lowercase()),
        }
    }

    pub fn render(&self) -> String {
        match self {
            Target::Command(name) => name.clone(),
            Target::Category(category) => format!("@{category}").to_lowercase(),
            Target::Everything => String::from("*"),
        }
    }

    pub fn precision(&self) -> u8 {
        match self {
            Target::Command(_) => 2,
            Target::Category(_) => 1,
            Target::Everything => 0,
        }
    }

    pub fn covers(&self, command: &str, category: Category) -> bool {
        match self {
            Target::Command(name) => name == command,
            Target::Category(target) => *target == category,
            Target::Everything => true,
        }
    }
}

pub fn category(raw: &str) -> Option<Category> {
    crate::command::CATEGORIES
        .into_iter()
        .find(|category| category.to_string().to_lowercase() == raw.to_lowercase())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub id: i64,
    pub scope: Scope,
    pub subject: Snowflake,
    pub target: Target,
    pub effect: Effect,
    pub priority: i32,
}

impl Rule {
    pub fn applies(&self, command: &str, category: Category) -> bool {
        self.target.covers(command, category)
    }

    pub fn describe(&self) -> String {
        format!(
            "{} {} {}",
            self.effect.as_str(),
            self.target.render(),
            self.scope.as_str()
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

impl RuleSet {
    pub fn compile(mut rules: Vec<Rule>) -> Self {
        rules.sort_by(|one, other| {
            other
                .priority
                .cmp(&one.priority)
                .then(other.scope.cmp(&one.scope))
                .then(other.target.precision().cmp(&one.target.precision()))
                .then(one.id.cmp(&other.id))
        });

        Self { rules }
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }
}
