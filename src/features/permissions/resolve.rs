use crate::command::Category;
use crate::domain::Snowflake;
use crate::features::permissions::rule::{Effect, Rule, RuleSet, Scope};

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub member: Snowflake,
    pub roles: &'a [(Snowflake, i64)],
    pub channel: Snowflake,
    pub command: &'a str,
    pub category: Category,
    pub is_developer: bool,
    pub is_owner: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Basis {
    Developer,
    Owner,
    Rule { id: i64, description: String },
    Unwritten,
}

impl Basis {
    pub fn describe(&self) -> String {
        match self {
            Basis::Developer => String::from("developer override"),
            Basis::Owner => String::from("guild owner"),
            Basis::Rule { id, description } => format!("{description} (#{id})"),
            Basis::Unwritten => String::from("command default"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Allowed { basis: Basis },
    Denied { basis: Basis },
    Default,
}

pub fn resolve(set: &RuleSet, request: &Request) -> Decision {
    if request.is_developer {
        return Decision::Allowed {
            basis: Basis::Developer,
        };
    }

    if request.is_owner {
        return Decision::Allowed {
            basis: Basis::Owner,
        };
    }

    let mut ranked: Vec<&Rule> = set
        .rules
        .iter()
        .filter(|rule| rule.applies(request.command, request.category))
        .filter(|rule| match rule.scope {
            Scope::Member => rule.subject == request.member,
            Scope::Channel => rule.subject == request.channel,
            Scope::Role => request.roles.iter().any(|(id, _)| *id == rule.subject),
        })
        .collect();

    ranked.sort_by_key(|rule| std::cmp::Reverse(precedence(rule, request)));

    let Some(winner) = ranked.first() else {
        return Decision::Default;
    };

    let basis = Basis::Rule {
        id: winner.id,
        description: winner.describe(),
    };

    match winner.effect {
        Effect::Deny => Decision::Denied { basis },
        Effect::Allow => Decision::Allowed { basis },
    }
}

fn precedence(rule: &Rule, request: &Request) -> (i32, u8, u8, i64) {
    let scope = match rule.scope {
        Scope::Member => 2,
        Scope::Role => 1,
        Scope::Channel => 0,
    };

    let position = request
        .roles
        .iter()
        .find(|(id, _)| *id == rule.subject)
        .map(|(_, position)| *position)
        .unwrap_or_default();

    (rule.priority, scope, rule.target.precision(), position)
}
