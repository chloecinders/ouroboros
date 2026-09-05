use crate::features::permissions::resolve::Basis;
use crate::features::permissions::rule::{Effect, Rule, RuleSet, Scope};
use crate::platform::ui::embed::Embed;
use crate::platform::ui::tone::Tone;

pub fn mention(scope: Scope, subject: u64) -> String {
    match scope {
        Scope::Role => format!("<@&{subject}>"),
        Scope::Member => format!("<@{subject}>"),
        Scope::Channel => format!("<#{subject}>"),
    }
}

fn line(rule: &Rule) -> String {
    format!(
        "`#{}` {} `{}` for {}",
        rule.id,
        rule.effect.as_str(),
        rule.target.render(),
        mention(rule.scope, rule.subject)
    )
}

pub fn listing(set: &RuleSet) -> Embed {
    if set.is_empty() {
        return unwritten();
    }

    Embed::new("PERMISSION RULES")
        .body(
            set.rules
                .iter()
                .map(|rule| match rule.priority {
                    0 => line(rule),
                    priority => format!("{} at priority {priority}", line(rule)),
                })
                .collect::<Vec<String>>()
                .join("\n"),
        )
        .tone(Tone::Info)
}

pub fn written(rule: &Rule) -> Embed {
    let headline = match rule.effect {
        Effect::Allow => "PERMISSION GRANTED",
        Effect::Deny => "PERMISSION DENIED",
    };

    Embed::new(headline)
        .subtitle(format!("Priority: {}", rule.priority))
        .body(line(rule))
        .tone(match rule.effect {
            Effect::Allow => Tone::Success,
            Effect::Deny => Tone::Warn,
        })
}

pub fn ranked(rule: &Rule) -> Embed {
    Embed::new("PRIORITY SET")
        .subtitle(format!("Priority: {}", rule.priority))
        .body(line(rule))
        .tone(Tone::Success)
}

pub fn unwritten() -> Embed {
    Embed::new("NO RULES FOUND")
        .body(Basis::Unwritten.describe())
        .footnote("This server does not have any permission rules.")
        .tone(Tone::Info)
}
