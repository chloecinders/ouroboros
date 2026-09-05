use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};

use crate::domain::Snowflake;
use crate::features::automod::managed::Offer;
use crate::features::automod::rule::{Mode, Source};
use crate::features::automod::{clause, managed};
use crate::web::Shared;
use crate::web::dash::auth::administers;
use crate::web::dash::rejection::{Rejection, misread};

#[derive(Debug, Serialize)]
pub struct Offered {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sources: Vec<&'static str>,
    pub offered: &'static str,
    pub mode: Option<&'static str>,
    pub effective: &'static str,
    pub response: String,
    pub action: Option<String>,
}

impl From<&Offer> for Offered {
    fn from(offer: &Offer) -> Self {
        Self {
            id: offer.managed.id.as_str().to_string(),
            name: offer.managed.name.clone(),
            description: managed::ui::description_of(&offer.managed).to_string(),
            sources: offer
                .managed
                .body
                .sources()
                .iter()
                .map(Source::as_str)
                .collect(),
            offered: offer.managed.mode.as_str(),
            mode: offer
                .subscription
                .as_ref()
                .map(|subscription| subscription.mode.as_str()),
            effective: offer.effective().as_str(),
            response: offer
                .subscription
                .as_ref()
                .map(|subscription| subscription.written.clone())
                .unwrap_or_default(),
            action: offer
                .subscription
                .as_ref()
                .map(|subscription| managed::ui::action(&subscription.response)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Adopted {
    pub mode: String,
    pub response: String,
}

async fn load_offer(web: &Shared, guild: Snowflake, id: &str) -> Result<Offer, Rejection> {
    let managed = managed::store::by_id(&web.pool, id)
        .await?
        .ok_or(Rejection::missing())?;

    let subscription = managed::store::subscription(&web.pool, guild, &managed.id).await?;

    match managed.mode == Mode::Active || subscription.is_some() {
        true => Ok(Offer {
            managed,
            subscription,
        }),
        false => Err(Rejection::missing()),
    }
}

pub async fn offers(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path(guild): Path<Snowflake>,
) -> Result<Json<Vec<Offered>>, Rejection> {
    administers(&web, &headers, guild).await?;

    let offers = managed::store::offers(&web.pool, guild).await?;

    Ok(Json(offers.iter().map(Offered::from).collect()))
}

pub async fn subscribe(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path((guild, id)): Path<(Snowflake, String)>,
    Json(written): Json<Adopted>,
) -> Result<Json<Offered>, Rejection> {
    administers(&web, &headers, guild).await?;

    let mut offer = load_offer(&web, guild, &id).await?;

    let mode = Mode::parse(written.mode.trim())
        .ok_or_else(|| Rejection::unusable("expected active or disabled"))?;

    let response = clause::parse_as(&written.response, 0, clause::Part::Response)
        .map_err(|failure| misread(failure, "response does not parse"))?;

    if offer.subscription.is_none() && offer.managed.mode != Mode::Active {
        return Err(Rejection::clashes("rule is unpublished"));
    }

    let rule = offer.managed.id.clone();

    let subscription = async {
        managed::store::subscribe(&web.pool, guild, &rule).await?;
        managed::store::set_guild_mode(&web.pool, guild, &rule, mode).await?;
        managed::store::set_response(&web.pool, guild, &rule, &written.response, &response).await?;

        managed::store::subscription(&web.pool, guild, &rule).await
    }
    .await?;

    web.rules.forget(guild);

    offer.subscription = subscription;

    Ok(Json(Offered::from(&offer)))
}

pub async fn unsubscribe(
    State(web): State<Shared>,
    headers: HeaderMap,
    Path((guild, id)): Path<(Snowflake, String)>,
) -> Result<StatusCode, Rejection> {
    administers(&web, &headers, guild).await?;

    let offer = load_offer(&web, guild, &id).await?;

    match managed::store::unsubscribe(&web.pool, guild, &offer.managed.id).await? {
        true => {
            web.rules.forget(guild);

            Ok(StatusCode::NO_CONTENT)
        }
        false => Err(Rejection::missing()),
    }
}
