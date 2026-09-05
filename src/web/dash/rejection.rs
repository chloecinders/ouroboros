use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::command::error::{self, Cause};

#[derive(Debug, Serialize)]
pub struct Error {
    pub problem: String,
    pub start: Option<usize>,
    pub len: Option<usize>,
}

impl Error {
    pub fn about(problem: &str) -> Self {
        Self {
            problem: problem.to_string(),
            start: None,
            len: None,
        }
    }

    pub fn spanning(problem: &str, start: usize, len: usize) -> Self {
        Self {
            problem: problem.to_string(),
            start: Some(start),
            len: Some(len),
        }
    }
}

pub fn misread(raw: error::Error, fallback: &str) -> Error {
    match (raw.span(), raw.label()) {
        (Some(span), Some(label)) => Error::spanning(&label.text, span.start, span.len),
        _ => Error::about(fallback),
    }
}

#[derive(Debug)]
pub struct Rejection {
    status: StatusCode,
    error: Option<Error>,
}

impl Rejection {
    pub const fn plain(status: StatusCode) -> Self {
        Self {
            status,
            error: None,
        }
    }

    pub fn saying(status: StatusCode, error: Error) -> Self {
        Self {
            status,
            error: Some(error),
        }
    }

    pub const fn unauthorized() -> Self {
        Self::plain(StatusCode::UNAUTHORIZED)
    }

    pub const fn forbidden() -> Self {
        Self::plain(StatusCode::FORBIDDEN)
    }

    pub const fn missing() -> Self {
        Self::plain(StatusCode::NOT_FOUND)
    }

    pub const fn broken() -> Self {
        Self::plain(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub const fn upstream() -> Self {
        Self::plain(StatusCode::BAD_GATEWAY)
    }

    pub fn unusable(problem: &str) -> Self {
        Self::saying(StatusCode::UNPROCESSABLE_ENTITY, Error::about(problem))
    }

    pub fn clashes(problem: &str) -> Self {
        Self::saying(StatusCode::CONFLICT, Error::about(problem))
    }
}

impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        match self.error {
            Some(error) => (self.status, Json(error)).into_response(),
            None => self.status.into_response(),
        }
    }
}

impl From<Error> for Rejection {
    fn from(error: Error) -> Self {
        Self::saying(StatusCode::UNPROCESSABLE_ENTITY, error)
    }
}

impl From<error::Error> for Rejection {
    fn from(_: error::Error) -> Self {
        Self::broken()
    }
}

fn taken(failure: &error::Error) -> bool {
    let Some(Cause::Store { source, .. }) = failure.cause() else {
        return false;
    };

    source
        .as_database_error()
        .and_then(|db| db.code())
        .is_some_and(|code| code == "23505")
}

pub fn stored(failure: error::Error, clash: &str) -> Rejection {
    match taken(&failure) {
        true => Rejection::clashes(clash),
        false => Rejection::broken(),
    }
}
