use serenity::all::{ButtonStyle, ChannelId, Permissions};

use crate::command::Boxed;
use crate::command::error::Result;
use crate::domain::Snowflake;
use crate::domain::logtype::{ALL, LogType};
use crate::features::settings::store;
use crate::platform::discord::interact::{
    Click, Control, Custom, Reaction, Router, Strangers, stale,
};
use crate::platform::ui::reply::{Button, Choice, Menu, Panel};

pub fn panel(owner: Snowflake, known: &[(LogType, ChannelId)], channel: ChannelId) -> Panel {
    let Some(id) = Custom::new("log-pick", owner, [channel.get().to_string()]).render() else {
        return Panel::from(bulk(owner, channel));
    };

    Panel::new(
        Menu::new(id, "Types to log here", choices(known, channel)),
        bulk(owner, channel),
    )
}

fn choices(known: &[(LogType, ChannelId)], channel: ChannelId) -> Vec<Choice> {
    ALL.iter()
        .map(|kind| {
            let routed = known
                .iter()
                .find(|(routed, _)| routed == kind)
                .map(|(_, destination)| *destination);

            let description = match routed {
                Some(destination) if destination == channel => {
                    String::from("Already logged in this channel")
                }
                Some(_) => String::from("Logged elsewhere"),
                None => String::from(kind.description()),
            };

            Choice::new(kind.as_str(), kind.title(), description)
        })
        .collect()
}

fn bulk(owner: Snowflake, channel: ChannelId) -> Vec<Button> {
    let at = channel.get().to_string();
    let offered: [(&str, &str, ButtonStyle); 4] = [
        ("keep", "Keep", ButtonStyle::Primary),
        ("all", "All", ButtonStyle::Secondary),
        ("reset", "Reset", ButtonStyle::Danger),
        ("close", "Close", ButtonStyle::Secondary),
    ];

    offered
        .into_iter()
        .filter_map(|(action, label, style)| {
            let id = Custom::new("log-bulk", owner, [at.clone(), action.to_string()]).render()?;

            Some(Button::new(id, label, style))
        })
        .collect()
}

fn about(click: &Click, at: usize) -> Option<(Snowflake, ChannelId)> {
    let guild = click.guild()?;
    let channel = click.part(at)?.parse::<u64>().ok()?;

    Some((guild, ChannelId::new(channel)))
}

fn chose(click: &Click) -> Boxed<'_, Result<Reaction>> {
    Box::pin(async move {
        let Some((guild, channel)) = about(click, 0) else {
            return Ok(Reaction::private(stale()));
        };

        let picked: Vec<LogType> = click
            .chosen()
            .into_iter()
            .filter_map(LogType::parse)
            .collect();

        store::route_many(&click.app.pool, guild, &picked, channel).await?;
        click.app.settings.forget(guild);

        Ok(Reaction::Dismiss)
    })
}

fn answered(click: &Click) -> Boxed<'_, Result<Reaction>> {
    Box::pin(async move {
        let (Some((guild, channel)), Some(action)) = (about(click, 0), click.part(1)) else {
            return Ok(Reaction::private(stale()));
        };

        match action {
            "close" => return Ok(Reaction::Dismiss),
            "keep" => {
                let before = store::routes(&click.app.pool, guild).await?;
                let unclaimed: Vec<LogType> = ALL
                    .into_iter()
                    .filter(|kind| !before.iter().any(|(routed, _)| routed == kind))
                    .collect();

                store::route_many(&click.app.pool, guild, &unclaimed, channel).await?;
            }
            "all" => store::route_many(&click.app.pool, guild, &ALL, channel).await?,
            "reset" => store::clear_channel(&click.app.pool, guild, channel).await?,
            _ => return Ok(Reaction::private(stale())),
        }

        click.app.settings.forget(guild);

        Ok(Reaction::Dismiss)
    })
}

pub fn register(router: &mut Router) {
    router.add(Control {
        key: "log-pick",
        user: Permissions::MANAGE_GUILD,
        one_of: Permissions::empty(),
        strangers: Strangers::Deny,
        handle: chose,
    });

    router.add(Control {
        key: "log-bulk",
        user: Permissions::MANAGE_GUILD,
        one_of: Permissions::empty(),
        strangers: Strangers::Deny,
        handle: answered,
    });
}
