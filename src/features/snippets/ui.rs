use crate::features::snippets::Scope;
use crate::features::snippets::store::Snippet;
use crate::platform::text::truncate;
use crate::platform::ui::embed::Embed;
use crate::platform::ui::tone::Tone;

fn section(title: &str, snippets: &[&Snippet], prefix: &str) -> Option<String> {
    if snippets.is_empty() {
        return None;
    }

    let listed: Vec<String> = snippets
        .iter()
        .map(|snippet| {
            format!(
                "`{prefix}{prefix}{}` -> `{}`",
                snippet.name,
                truncate::clamp(&snippet.body, 80).replace('`', "'")
            )
        })
        .collect();

    Some(format!("**{title}**\n{}", listed.join("\n")))
}

pub fn listing(snippets: &[Snippet], prefix: &str) -> Embed {
    if snippets.is_empty() {
        return Embed::new("NO SNIPPETS")
            .footnote(format!(
                "{prefix}snippet add <name> <command> to create snippets | {prefix}snippet help for more information"
            ))
            .tone(Tone::Info);
    }

    let (owned, server): (Vec<&Snippet>, Vec<&Snippet>) = snippets
        .iter()
        .partition(|snippet| matches!(snippet.scope, Scope::User(_)));

    let parts: Vec<String> = [
        section("USER SNIPPETS", &owned, prefix),
        section("SERVER SNIPPETS", &server, prefix),
    ]
    .into_iter()
    .flatten()
    .collect();

    Embed::new("SNIPPETS")
        .body(parts.join("\n\n"))
        .footnote("User snippets are available to you across servers")
        .tone(Tone::Info)
}

pub fn shown(snippet: &Snippet, prefix: &str) -> Embed {
    Embed::new(snippet.name.to_uppercase())
        .subtitle(format!("Scope: {}", snippet.scope.label()))
        .subtitle(format!("Command: `{prefix}{prefix}{}`", snippet.name))
        .quote(format!("{prefix}{}", snippet.body))
        .tone(Tone::Info)
}

pub fn saved(snippet: &Snippet, replaced: bool, prefix: &str) -> Embed {
    Embed::new(match replaced {
        true => "SNIPPET REPLACED",
        false => "SNIPPET SAVED",
    })
    .subtitle(format!("Scope: {}", snippet.scope.label()))
    .subtitle(format!("Command: `{prefix}{prefix}{}`", snippet.name))
    .quote(format!("{prefix}{}", snippet.body))
    .tone(Tone::Success)
}

pub fn deleted(name: &str, scope: Scope) -> Embed {
    Embed::new("SNIPPET DELETED")
        .subtitle(format!("Scope: {}", scope.label()))
        .subtitle(format!("Name: {name}"))
        .tone(Tone::Success)
}
