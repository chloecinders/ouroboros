use std::{sync::{Arc, OnceLock}, time::Instant};

use serenity::{all::{GatewayIntents, Settings, ShardManager}, prelude::TypeMapKey, Client};
use tokio::{fs::File, io::AsyncReadExt};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{config::Config, event_handler::Handler};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

mod config;
mod event_handler;
mod commands;
mod transformers;
mod lexer;
mod database;
mod utils;
mod constants;

pub static START_TIME: OnceLock<Instant> = OnceLock::new();
pub static SQL: OnceLock<PgPool> = OnceLock::new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .init();

    let _ = START_TIME.set(Instant::now());

    let mut file = File::open("./Config.toml").await.expect("Could not find Config.toml in project root.");
    let mut contents = String::new();

    if let Err(_) = file.read_to_string(&mut contents).await {
        panic!("Could not read Config.toml.");
    }

    let config: Config = toml::from_str(contents.as_str()).unwrap_or_else(|_| panic!("Could not parse Config.toml."));

    let active_env = match config.bot.env.as_str() {
        "release" => &config.release,
        "dev" => &config.dev,
        _ => panic!("Unknown bot.env, verify bot.env is one of release or dev")
    };

    let _ = SQL.set({
        async {
            PgPoolOptions::new()
                .max_connections(active_env.max_connections)
                .connect(&active_env.database_url)
                .await
                .expect("Failed to create database pool, make sure the database url in the config is valid.")
        }.await
    });

    database::run_migrations();

    let intents = GatewayIntents::all();

    let mut cache_settings = Settings::default();
    cache_settings.max_messages = 1000;
    let handler = Handler::new(active_env.prefix.clone());

    let mut client = Client::builder(&active_env.token, intents)
        .event_handler(handler)
        .cache_settings(cache_settings)
        .await
        .expect("Unable to create client");

    let shard_manager = client.shard_manager.clone();
    client.data.write().await.insert::<ShardManagerContainer>(shard_manager);

    if let Err(e) = client.start().await {
        println!("Client error: {e:?}")
    }
}
