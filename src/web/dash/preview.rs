use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

use crate::features::automod::clause;
use crate::web::Shared;
use crate::web::dash::auth::signed;
use crate::web::dash::rejection::{Error, Rejection, misread};

#[derive(Debug, Deserialize)]
pub struct Draft {
    pub source: String,
    pub part: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Reading {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered: Option<String>,
}

pub fn part(asked: Option<&str>) -> clause::Part {
    match asked {
        Some("detection") => clause::Part::Detection,
        Some("response") => clause::Part::Response,
        _ => clause::Part::Whole,
    }
}

pub fn read_as(source: &str, part: clause::Part) -> Reading {
    match clause::parse_as(source, 0, part) {
        Ok(body) => Reading {
            ok: true,
            error: None,
            rendered: Some(clause::render(&body)),
        },
        Err(failure) => Reading {
            ok: false,
            error: Some(misread(failure, "clauses do not parse")),
            rendered: None,
        },
    }
}

pub async fn check(
    State(web): State<Shared>,
    headers: HeaderMap,
    Json(draft): Json<Draft>,
) -> Result<Json<Reading>, Rejection> {
    signed(&web, &headers).await?;

    Ok(Json(read_as(&draft.source, part(draft.part.as_deref()))))
}
