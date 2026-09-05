use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::all::ChannelId;

use crate::domain::Snowflake;
use crate::domain::logtype::{ALL, LogType};
use crate::features::errorlog::store as error_store;
use crate::features::settings::store as settings;
use crate::web::Shared;
use crate::web::dash::auth::administers;
use crate::web::dash::rejection::Rejection;

#[derive(Debug, Serialize)]
pub struct Definition {
    pub kind: &'static str,
    pub title: &'static str,
    pub about: &'static str,
    #[serde(with = "crate::web::flat::maybe")]
    pub channel: Option<Snowflake>,
}

pub async fn logs(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(guild): Path<Snowflake>,
) -> Result<Json<Vec<Definition>>, Rejection> {
    administers(&web, &headers, guild).await?;

    let routed = settings::routes(&web.pool, guild).await?;

    Ok(Json(
        ALL.iter()
            .map(|kind| Definition {
                kind: kind.as_str(),
                title: kind.title(),
                about: kind.description(),
                channel: routed
                    .iter()
                    .find(|(routed, _)| routed == kind)
                    .map(|(_, channel)| channel.get()),
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct Routed {
    pub kind: String,
    pub channel: Option<String>,
}

pub async fn route(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(guild): Path<Snowflake>,
    Json(written): Json<Routed>,
) -> Result<StatusCode, Rejection> {
    administers(&web, &headers, guild).await?;

    let kind =
        LogType::parse(&written.kind).ok_or_else(|| Rejection::unusable("unknown log kind"))?;

    let Some(asked) = written
        .channel
        .as_deref()
        .filter(|channel| !channel.is_empty())
    else {
        settings::unroute(&web.pool, guild, kind).await?;

        web.settings.forget(guild);

        return Ok(StatusCode::NO_CONTENT);
    };

    let channel = asked
        .parse::<Snowflake>()
        .ok()
        .filter(|id| *id != 0)
        .ok_or_else(|| Rejection::unusable("expected a channel id"))?;

    let view = web.guilds.view(guild).await.ok_or(Rejection::missing())?;

    if !view.channels.iter().any(|listed| listed.id == channel) {
        return Err(Rejection::unusable("channel is not in this server"));
    }

    settings::route_many(&web.pool, guild, &[kind], ChannelId::new(channel)).await?;

    web.settings.forget(guild);

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct Trouble {
    pub id: i64,
    pub headline: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub delivered: bool,
    pub at: DateTime<Utc>,
}

pub async fn errors(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(guild): Path<Snowflake>,
) -> Result<Json<Vec<Trouble>>, Rejection> {
    administers(&web, &headers, guild).await?;

    let recent = error_store::recent(&web.pool, guild, 100).await?;

    Ok(Json(
        recent
            .into_iter()
            .map(|row| Trouble {
                id: row.id,
                headline: row.headline,
                detail: row.detail,
                delivered: row.delivered,
                at: row.occurred_at,
            })
            .collect(),
    ))
}
