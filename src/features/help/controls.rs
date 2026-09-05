use serenity::all::{ButtonStyle, Permissions};

use crate::command::error::Result;
use crate::command::{Boxed, help};
use crate::domain::Snowflake;
use crate::platform::discord::interact::{Click, Control, Custom, Reaction, Router, Strangers};
use crate::platform::ui::reply::Button;

pub fn nav(owner: Snowflake, at: usize, total: usize) -> Vec<Button> {
    if total <= 1 {
        return Vec::new();
    }

    let last = total.saturating_sub(1);
    let steps: [(&str, usize, String, bool); 5] = [
        ("first", 0, String::from("<<"), at == 0),
        ("prev", at.saturating_sub(1), String::from("<"), at == 0),
        ("at", at, format!("{}/{total}", at + 1), true),
        ("next", (at + 1).min(last), String::from(">"), at == last),
        ("last", last, String::from(">>"), at == last),
    ];

    steps
        .into_iter()
        .filter_map(|(name, target, label, off)| {
            let id =
                Custom::new("help-page", owner, [name.to_string(), target.to_string()]).render()?;

            Some(Button::new(id, label, ButtonStyle::Secondary).disabled(off))
        })
        .collect()
}

fn turn(click: &Click) -> Boxed<'_, Result<Reaction>> {
    Box::pin(async move {
        let developer = click.app.is_developer(click.interaction.user.id.get());
        let pages = help::pages(&click.app.registry, developer);
        let at = click
            .part(1)
            .and_then(|raw| raw.parse::<usize>().ok())
            .unwrap_or_default()
            .min(pages.len().saturating_sub(1));

        Ok(Reaction::replace(
            help::sheet(&pages, at, click.app.prefix()),
            nav(click.owner(), at, pages.len()),
        ))
    })
}

pub fn register(router: &mut Router) {
    router.add(Control {
        key: "help-page",
        user: Permissions::empty(),
        one_of: Permissions::empty(),
        strangers: Strangers::Fork,
        handle: turn,
    });
}
