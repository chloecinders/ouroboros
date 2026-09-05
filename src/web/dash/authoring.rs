use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};

use crate::domain::ids::RuleId;
use crate::features::automod::rule::{Body, Mode};
use crate::features::automod::{clause, commands, managed};
use crate::platform::text::truncate;
use crate::web::Shared;
use crate::web::dash::auth::authoring_viewer;
use crate::web::dash::rejection::{Error, Rejection, misread, stored};

#[derive(Debug, Serialize)]
pub struct Authored {
    pub id: String,
    pub name: String,
    pub mode: &'static str,
    pub source: String,
    pub description: String,
}

impl From<managed::Managed> for Authored {
    fn from(managed: managed::Managed) -> Self {
        Self {
            id: managed.id.into_inner(),
            name: managed.name,
            mode: managed.mode.as_str(),
            source: managed.source,
            description: managed.description,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Composed {
    pub name: String,
    pub mode: String,
    pub source: String,
    pub description: Option<String>,
}

fn detects(written: &Composed) -> Result<(Mode, Body), Error> {
    let mode =
        Mode::parse(&written.mode).ok_or_else(|| Error::about("expected active or disabled"))?;

    let name = written.name.trim();

    if name.is_empty() {
        return Err(Error::about("provide a rule name"));
    }

    if name.len() > 64 {
        return Err(Error::about("name cannot be longer than 64 characters"));
    }

    if commands::managed::RESERVED.contains(&name.to_lowercase().as_str()) {
        return Err(Error::about("that name is a managed subcommand"));
    }

    let body = clause::parse_as(&written.source, 0, clause::Part::Detection)
        .map_err(|failure| misread(failure, "those clauses do not parse"))?;

    Ok((mode, body))
}

fn described(written: &Composed) -> String {
    truncate::clamp(
        written.description.as_deref().unwrap_or_default().trim(),
        300,
    )
}

pub async fn all(
    State(web): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Vec<Authored>>, Rejection> {
    authoring_viewer(&web, &headers).await?;

    let authored = managed::store::all(&web.pool).await?;

    Ok(Json(authored.into_iter().map(Authored::from).collect()))
}

pub async fn compose(
    State(web): State<Shared>,
    headers: HeaderMap,
    Json(written): Json<Composed>,
) -> Result<(StatusCode, Json<Authored>), Rejection> {
    authoring_viewer(&web, &headers).await?;

    let (mode, body) = detects(&written)?;

    let name = written.name.trim().to_string();

    if managed::store::find(&web.pool, &name).await?.is_some() {
        return Err(Rejection::clashes("a managed rule already has that name"));
    }

    let composed = managed::Managed {
        id: managed::generate(),
        name,
        description: described(&written),
        mode,
        source: written.source.clone(),
        body,
    };

    managed::store::save(&web.pool, &composed)
        .await
        .map_err(|failure| stored(failure, "a managed rule already has that name"))?;

    Ok((StatusCode::CREATED, Json(Authored::from(composed))))
}

pub async fn revise(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(written): Json<Composed>,
) -> Result<Json<Authored>, Rejection> {
    authoring_viewer(&web, &headers).await?;

    let existing = managed::store::by_id(&web.pool, &id)
        .await?
        .ok_or(Rejection::missing())?;

    let (mode, body) = detects(&written)?;

    let revised = managed::Managed {
        id: existing.id,
        name: written.name.trim().to_string(),
        description: described(&written),
        mode,
        source: written.source.clone(),
        body,
    };

    match managed::store::rewrite(&web.pool, &revised).await {
        Ok(true) => {}
        Ok(false) => return Err(Rejection::missing()),
        Err(failure) => return Err(stored(failure, "a managed rule already has that name")),
    }

    web.rules.forget_everywhere();

    Ok(Json(Authored::from(revised)))
}

pub async fn delete(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, Rejection> {
    authoring_viewer(&web, &headers).await?;

    match managed::store::delete_by_id(&web.pool, &RuleId::from(id)).await? {
        true => {
            web.rules.forget_everywhere();

            Ok(StatusCode::NO_CONTENT)
        }
        false => Err(Rejection::missing()),
    }
}
