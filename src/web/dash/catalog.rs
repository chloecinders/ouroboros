use std::sync::OnceLock;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde::Serialize;

use crate::command::CATEGORIES;
use crate::command::registry::Registry;
use crate::web::Shared;
use crate::web::dash::auth::signed;
use crate::web::dash::rejection::Rejection;

#[derive(Debug, Serialize)]
pub struct Listed {
    pub name: &'static str,
    pub category: String,
    pub about: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Vocabulary {
    pub categories: Vec<String>,
    pub commands: Vec<Listed>,
}

pub fn vocabulary() -> &'static Vocabulary {
    static READ: OnceLock<Vocabulary> = OnceLock::new();

    READ.get_or_init(|| {
        let mut registry = Registry::new();

        crate::features::register(&mut registry);

        let mut commands: Vec<Listed> = registry
            .all()
            .iter()
            .filter(|entry| !entry.meta.developer)
            .map(|entry| Listed {
                name: entry.meta.name,
                category: entry.meta.category.to_string().to_lowercase(),
                about: entry.meta.short,
            })
            .collect();

        commands.sort_by_key(|listed| listed.name);

        let categories = CATEGORIES
            .iter()
            .map(|category| category.to_string().to_lowercase())
            .filter(|category| commands.iter().any(|listed| &listed.category == category))
            .collect();

        Vocabulary {
            categories,
            commands,
        }
    })
}

pub async fn commands(
    State(web): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<&'static Vocabulary>, Rejection> {
    signed(&web, &headers).await?;

    Ok(Json(vocabulary()))
}
