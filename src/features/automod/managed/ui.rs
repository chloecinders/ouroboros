use crate::features::automod::managed::{Managed, Offer};
use crate::features::automod::rule::{Body, Mode};
use crate::features::automod::{clause, ui as shared};
use crate::platform::text::{duration, truncate};
use crate::platform::ui::embed::Embed;
use crate::platform::ui::tone::Tone;

pub fn page_of(offers: &[Offer], at: usize) -> &[Offer] {
    let start = (at * 5).min(offers.len());
    let end = (start + 5).min(offers.len());

    &offers[start..end]
}

pub fn description_of(managed: &Managed) -> &str {
    match managed.description.trim().is_empty() {
        true => "no description",
        false => managed.description.trim(),
    }
}

fn subscription(offer: &Offer) -> String {
    let Some(subscription) = &offer.subscription else {
        return String::from("not subscribed");
    };

    if offer.managed.mode != Mode::Active {
        return format!("{}, unpublished", subscription.mode);
    }

    subscription.mode.to_string()
}

pub fn action(body: &Body) -> String {
    let mut out = shared::action(&body.outcome);

    if let Some(verb) = body.outcome.punishment_type
        && verb.has_duration()
        && !body.outcome.duration.is_zero()
    {
        out.push_str(&format!(
            " for {}",
            duration::compact(body.outcome.duration)
        ));
    }

    if body.outcome.clear_days > 0 {
        out.push_str(&format!(", clearing {} days", body.outcome.clear_days));
    }

    out
}

pub fn offer(offer: &Offer) -> Embed {
    let embed = Embed::new(format!("MANAGED {}", offer.managed.name.to_uppercase()))
        .subtitle(format!("ID: `{}`", offer.managed.id))
        .subtitle(format!("Subscription: {}", subscription(offer)))
        .lead(description_of(&offer.managed))
        .tone(shared::tone(offer.effective()));

    match &offer.subscription {
        Some(subscription) => {
            let written = embed.quote(answered(&subscription.response));

            match offer.effective() {
                Mode::Active => written,
                Mode::Disabled => written.footnote(prompt(&offer.managed)),
            }
        }
        None => embed.footnote(format!(
            "Run `managed add {}` to subscribe",
            offer.managed.name
        )),
    }
}

pub fn clauses() -> Embed {
    Embed::new("MANAGED RESPONSE CLAUSES")
        .body(clause::summaries(clause::Part::Response))
        .footnote(
            "`managed clauses <clause>` to see more info on a clause. The detection clauses \
            of a managed rule are written by the developers and are not shown",
        )
        .tone(Tone::Info)
}

fn prompt(managed: &Managed) -> String {
    format!(
        "Add a response to triggers using `managed respond {}` | run `managed clauses` for a list of valid response clauses",
        managed.name
    )
}

pub fn answered(response: &Body) -> String {
    let written = clause::render(response);

    match written.trim().is_empty() {
        true => String::from("then nothing"),
        false => written,
    }
}

fn entry(offer: &Offer) -> String {
    format!(
        "**{}**\n-# Rule ID: `{}` | Subscription: {}",
        truncate::clamp(&offer.managed.name, 22).to_uppercase(),
        offer.managed.id,
        subscription(offer)
    )
}

pub fn listing(offers: &[Offer], at: usize) -> Embed {
    if offers.is_empty() {
        return Embed::new("NO MANAGED RULES")
            .footnote("None published yet")
            .tone(Tone::Info);
    }

    let total = offers.len().div_ceil(5).max(1);

    Embed::new("MANAGED RULES")
        .body(
            page_of(offers, at)
                .iter()
                .map(entry)
                .collect::<Vec<String>>()
                .join("\n\n"),
        )
        .footnote(format!(
            "Pick one below, or run `managed show <name>` | page {} of {total}",
            at + 1
        ))
        .tone(Tone::Info)
}

pub fn saved(managed: &Managed, replaced: bool) -> Embed {
    let headline = match replaced {
        true => "MANAGED RULE UPDATED",
        false => "MANAGED RULE CREATED",
    };

    Embed::new(headline)
        .subtitle(format!("ID: `{}`", managed.id))
        .subtitle(format!("Name: {}", managed.name))
        .subtitle(format!("Mode: {}", managed.mode))
        .quote(clause::render(&managed.body))
        .tone(shared::tone(managed.mode))
}

pub fn inspect(managed: &Managed) -> Embed {
    Embed::new(format!("MANAGED {}", managed.name.to_uppercase()))
        .subtitle(format!("ID: `{}`", managed.id))
        .subtitle(format!("Mode: {}", managed.mode))
        .lead(description_of(managed))
        .quote(clause::render(&managed.body))
        .tone(shared::tone(managed.mode))
}
