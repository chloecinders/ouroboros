use std::sync::OnceLock;

use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};

use crate::web::Shared;

include!(concat!(env!("OUT_DIR"), "/chunks.rs"));

static PAGE: OnceLock<String> = OnceLock::new();

pub async fn dashboard(State(web): State<Shared>) -> Html<&'static str> {
    let stamped = PAGE.get_or_init(|| {
        let id = web
            .oauth
            .as_ref()
            .map(|oauth| oauth.client_id.as_str())
            .unwrap_or_default();

        include_str!(concat!(env!("OUT_DIR"), "/dash.html")).replace("__AEGIS_CLIENT_ID__", id)
    });

    Html(stamped.as_str())
}

pub async fn chunk(Path((app, file)): Path<(String, String)>) -> Response {
    let wanted = format!("{app}/{file}");

    let Some((_, body)) = CHUNKS.iter().find(|(name, _)| *name == wanted) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let caching = match file == "main.js" {
        true => "no-cache",
        false => "public, max-age=604800",
    };

    (
        [
            (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, caching),
        ],
        *body,
    )
        .into_response()
}

pub async fn runtime() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=604800"),
        ],
        include_str!(concat!(env!("OUT_DIR"), "/activity.js")),
    )
        .into_response()
}

pub async fn font(Path(name): Path<String>) -> Response {
    let bytes: &'static [u8] = match name.as_str() {
        "archivo.woff2" => include_bytes!("../../web/shared/fonts/archivo.woff2"),
        "jetbrains-mono.woff2" => include_bytes!("../../web/shared/fonts/jetbrains-mono.woff2"),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    (
        [
            (header::CONTENT_TYPE, "font/woff2"),
            (header::CACHE_CONTROL, "public, max-age=604800"),
        ],
        bytes,
    )
        .into_response()
}
