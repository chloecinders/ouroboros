use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};

use crate::domain::Snowflake;
use crate::domain::ids::RuleId;
use crate::features::automod::rule::{Author, Body, Mode, Rule};
use crate::features::automod::{clause, commands, store};
use crate::web::Shared;
use crate::web::dash::auth::administers;
use crate::web::dash::rejection::{Error, Rejection, misread, stored};

#[derive(Debug, Serialize)]
pub struct Saved {
    pub id: String,
    pub name: String,
    pub mode: &'static str,
    pub source: String,
}

impl From<Rule> for Saved {
    fn from(rule: Rule) -> Self {
        Self {
            id: rule.id.into_inner(),
            name: rule.name,
            mode: rule.mode.as_str(),
            source: rule.source,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WrittenRule {
    pub name: String,
    pub mode: String,
    pub source: String,
}

fn compile(written: &WrittenRule) -> Result<(Mode, Body), Error> {
    let mode =
        Mode::parse(&written.mode).ok_or_else(|| Error::about("expected active or disabled"))?;

    let name = written.name.trim();

    if name.is_empty() {
        return Err(Error::about("no rule name provided"));
    }

    if name.len() > 64 {
        return Err(Error::about("name can not be longer than 64 characters"));
    }

    if commands::rule::RESERVED.contains(&name.to_lowercase().as_str()) {
        return Err(Error::about("name can not be a rule subcommand"));
    }

    let body = clause::parse(&written.source, 0)
        .map_err(|failure| misread(failure, "clauses do not parse"))?;

    Ok((mode, body))
}

pub async fn all(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(guild): Path<Snowflake>,
) -> Result<Json<Vec<Saved>>, Rejection> {
    administers(&web, &headers, guild).await?;

    let loaded = store::all(&web.pool, guild).await?;

    Ok(Json(loaded.into_iter().map(Saved::from).collect()))
}

pub async fn create(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(guild): Path<Snowflake>,
    Json(written): Json<WrittenRule>,
) -> Result<(StatusCode, Json<Saved>), Rejection> {
    administers(&web, &headers, guild).await?;

    let existing = store::all(&web.pool, guild).await?;

    if existing.len() >= 200 {
        return Err(Rejection::clashes("can not have more than 200 rules"));
    }

    let wanted = written.name.trim().to_lowercase();

    if existing
        .iter()
        .any(|rule| rule.name.to_lowercase() == wanted)
    {
        return Err(Rejection::clashes("duplicate rule name"));
    }

    let (mode, body) = compile(&written)?;

    let rule = Rule {
        id: RuleId::generate(),
        guild,
        name: written.name.trim().to_string(),
        mode,
        author: Author::Guild,
        source: written.source,
        body,
    };

    let id = store::save(&web.pool, &rule)
        .await
        .map_err(|failure| stored(failure, "duplicate rule name"))?;

    web.rules.forget(guild);

    Ok((
        StatusCode::CREATED,
        Json(Saved {
            id: id.into_inner(),
            name: rule.name,
            mode: rule.mode.as_str(),
            source: rule.source,
        }),
    ))
}

pub async fn amend(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path((guild, id)): Path<(Snowflake, String)>,
    Json(written): Json<WrittenRule>,
) -> Result<Json<Saved>, Rejection> {
    administers(&web, &headers, guild).await?;

    let existing = match store::by_id(&web.pool, &id).await? {
        Some(found) if found.guild == guild => found,
        _ => return Err(Rejection::missing()),
    };

    let compiled = compile(&written);

    let mode = Mode::parse(&written.mode)
        .ok_or_else(|| Rejection::unusable("expected active or disabled"))?;

    let body = match compiled {
        Ok((_, parsed)) => parsed,
        Err(error) if mode == Mode::Active => return Err(error.into()),
        Err(_) => existing.body,
    };

    let rule = Rule {
        id: existing.id,
        guild,
        name: written.name.trim().to_string(),
        mode,
        author: Author::Guild,
        source: written.source,
        body,
    };

    match store::update(&web.pool, &rule).await {
        Ok(true) => {}
        Ok(false) => return Err(Rejection::missing()),
        Err(failure) => return Err(stored(failure, "duplicate name")),
    }

    web.rules.forget(guild);

    Ok(Json(Saved::from(rule)))
}

pub async fn delete(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path((guild, id)): Path<(Snowflake, String)>,
) -> Result<StatusCode, Rejection> {
    administers(&web, &headers, guild).await?;

    match store::by_id(&web.pool, &id).await? {
        Some(found) if found.guild == guild => {}
        _ => return Err(Rejection::missing()),
    }

    match store::delete_by_id(&web.pool, &RuleId::from(id)).await? {
        true => {
            web.rules.forget(guild);

            Ok(StatusCode::NO_CONTENT)
        }
        false => Err(Rejection::missing()),
    }
}
