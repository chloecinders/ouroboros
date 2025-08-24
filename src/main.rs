use std::{sync::{Arc, OnceLock}, time::{Duration, Instant}};

use serenity::{all::{GatewayIntents, Settings, ShardManager}, prelude::TypeMapKey, Client};
use tokio::{fs::File, io::AsyncReadExt, time::sleep};
use sqlx::{postgres::PgPoolOptions, query, PgPool};

use crate::{config::Config, database::{GuildSettings, GuildSettingsLog}, event_handler::Handler};

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
mod tasks;
mod constants;

pub static START_TIME: OnceLock<Instant> = OnceLock::new();
pub static SQL: OnceLock<PgPool> = OnceLock::new();
pub static GUILD_SETTINGS: OnceLock<Vec<GuildSettings>> = OnceLock::new();

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

    database::run_migrations().await;

    if let Ok(data) = query!("SELECT * FROM guild_settings").fetch_all(SQL.get().unwrap()).await {
        GUILD_SETTINGS.set(data.into_iter().map(|record| {
            GuildSettings {
                guild_id: record.guild_id as u64,
                log: GuildSettingsLog {
                    channel: record.log_channel.map(|n| n as u64)
                }
            }
        }).collect()).expect("Couldnt set guild_settings global");
    } else {
        panic!("Couldn't fetch guild_settings");
    }

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

    let http = client.http.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(60 * 5)).await;
            tasks::check_expiring_bans(&http).await;
        }
    });

    if let Err(e) = client.start().await {
        eprintln!("Client error: {e:?}")
    }
}
