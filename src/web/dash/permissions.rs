use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};

use crate::domain::Snowflake;
use crate::features::permissions::rule::{Effect, Rule as Permit, Scope, Target};
use crate::features::permissions::store as permits;
use crate::web::Shared;
use crate::web::dash::auth::administers;
use crate::web::dash::catalog::vocabulary;
use crate::web::dash::rejection::{Error, Rejection};
use crate::web::directory::Entry;

#[derive(Debug, Serialize)]
pub struct Permission {
    pub id: i64,
    pub scope: &'static str,
    #[serde(with = "crate::web::flat")]
    pub subject: Snowflake,
    pub target: String,
    pub effect: &'static str,
    pub priority: i32,
}

impl From<&Permit> for Permission {
    fn from(rule: &Permit) -> Self {
        Self {
            id: rule.id,
            scope: rule.scope.as_str(),
            subject: rule.subject,
            target: rule.target.render(),
            effect: rule.effect.as_str(),
            priority: rule.priority,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Granted {
    pub scope: String,
    pub subject: String,
    pub target: String,
    pub effect: String,
    #[serde(default)]
    pub priority: i32,
}

#[derive(Debug)]
pub struct Intent {
    pub scope: Scope,
    pub subject: Snowflake,
    pub target: Target,
    pub effect: Effect,
    pub priority: i32,
}

fn intended(written: &Granted) -> Result<Intent, Error> {
    let scope = Scope::parse(written.scope.trim())
        .ok_or_else(|| Error::about("expected role, member or channel"))?;

    let subject = written
        .subject
        .trim()
        .parse::<Snowflake>()
        .ok()
        .filter(|id| *id != 0)
        .ok_or_else(|| Error::about("provide a subject id"))?;

    let effect = Effect::parse(written.effect.trim())
        .ok_or_else(|| Error::about("expected allow or deny"))?;

    let target = Target::parse(written.target.trim());

    let addressable = match &target {
        Target::Everything | Target::Category(_) => true,
        Target::Command(name) => vocabulary()
            .commands
            .iter()
            .any(|listed| listed.name == name),
    };

    if !addressable {
        return Err(Error::about("unknown command or category"));
    }

    Ok(Intent {
        scope,
        subject,
        target,
        effect,
        priority: written.priority,
    })
}

fn present(view: &[Entry], id: Snowflake) -> bool {
    view.iter().any(|entry| entry.id == id)
}

pub async fn all(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(guild): Path<Snowflake>,
) -> Result<Json<Vec<Permission>>, Rejection> {
    administers(&web, &headers, guild).await?;

    let permits = permits::all(&web.pool, guild).await?;

    Ok(Json(permits.rules.iter().map(Permission::from).collect()))
}

pub async fn grant(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(guild): Path<Snowflake>,
    Json(written): Json<Granted>,
) -> Result<(StatusCode, Json<Permission>), Rejection> {
    administers(&web, &headers, guild).await?;

    let intent = intended(&written)?;

    let permits = permits::all(&web.pool, guild).await?;

    if permits.len() >= 200 {
        return Err(Rejection::clashes(
            "this server already holds 200 permission rules",
        ));
    }

    let view = web.guilds.view(guild).await.ok_or(Rejection::missing())?;

    let astray = match intent.scope {
        Scope::Role if !present(&view.roles, intent.subject) => {
            Some("that role is not in this server")
        }
        Scope::Channel if !present(&view.channels, intent.subject) => {
            Some("that channel is not in this server")
        }
        _ => None,
    };

    if let Some(missing) = astray {
        return Err(Rejection::unusable(missing));
    }

    let id = permits::add(
        &web.pool,
        guild,
        intent.scope,
        intent.subject,
        &intent.target,
        intent.effect,
        intent.priority,
    )
    .await?;

    web.permits.forget(guild);

    let stored = Permit {
        id,
        scope: intent.scope,
        subject: intent.subject,
        target: intent.target,
        effect: intent.effect,
        priority: intent.priority,
    };

    Ok((StatusCode::CREATED, Json(Permission::from(&stored))))
}

#[derive(Debug, Deserialize)]
pub struct Ranked {
    pub priority: i32,
}

pub async fn retune(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path((guild, id)): Path<(Snowflake, i64)>,
    Json(written): Json<Ranked>,
) -> Result<Json<Permission>, Rejection> {
    administers(&web, &headers, guild).await?;

    let moved = permits::set_priority(&web.pool, guild, id, written.priority)
        .await?
        .ok_or(Rejection::missing())?;

    web.permits.forget(guild);

    Ok(Json(Permission::from(&moved)))
}

pub async fn revoke(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path((guild, id)): Path<(Snowflake, i64)>,
) -> Result<StatusCode, Rejection> {
    administers(&web, &headers, guild).await?;

    match permits::remove(&web.pool, guild, id).await? {
        true => {
            web.permits.forget(guild);

            Ok(StatusCode::NO_CONTENT)
        }
        false => Err(Rejection::missing()),
    }
}
