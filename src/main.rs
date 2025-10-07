use std::{
    env, fs,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use serenity::{
    Client,
    all::{GatewayIntents, Settings, ShardManager},
    prelude::TypeMapKey,
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{fs::File, io::AsyncReadExt, sync::Mutex, time::sleep};
use tracing::{error, warn};

use crate::{
    config::{Config, Environment},
    event_handler::Handler,
    utils::GuildSettings,
};
use std::process::Command as SystemCommand;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

mod commands;
mod config;
mod constants;
mod database;
mod event_handler;
mod lexer;
mod tasks;
mod transformers;
mod utils;

pub static START_TIME: OnceLock<Instant> = OnceLock::new();
pub static SQL: OnceLock<PgPool> = OnceLock::new();
pub static GUILD_SETTINGS: OnceLock<Mutex<GuildSettings>> = OnceLock::new();
pub static BOT_CONFIG: OnceLock<Environment> = OnceLock::new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt().init();

    #[cfg(target_os = "windows")]
    if let Some(arg) = std::env::args()
        .collect::<Vec<String>>()
        .iter()
        .find(|a| a.starts_with("--update"))
    {
        use std::process::exit;
        use tracing::info;

        info!("Starting update process");
        if let Err(err) = update(arg) {
            warn!("Got error while updating; err = {err:?}");
        }
        exit(0);
    }

    if let Err(err) = cleanup() {
        warn!("Could not clean up update files; err = {err:?}");
    };

    let _ = START_TIME.set(Instant::now());

    let mut file = File::open("./Config.toml")
        .await
        .expect("Could not find Config.toml in project root.");
    let mut contents = String::new();

    if file.read_to_string(&mut contents).await.is_err() {
        panic!("Could not read Config.toml.");
    }

    let config: Config = toml::from_str(contents.as_str())
        .unwrap_or_else(|_| panic!("Could not parse Config.toml."));

    let active_env = match config.bot.env.as_str() {
        "release" => &config.release,
        "dev" => &config.dev,
        _ => panic!("Unknown bot.env, verify bot.env is one of release or dev"),
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

    GUILD_SETTINGS
        .set(Mutex::new(GuildSettings::new()))
        .unwrap();

    BOT_CONFIG.set(active_env.clone()).unwrap();

    let intents = GatewayIntents::all();

    let mut cache_settings = Settings::default();
    cache_settings.max_messages = active_env.msg_cache;
    let handler = Handler::new(active_env.prefix.clone());

    let mut client = Client::builder(&active_env.token, intents)
        .event_handler(handler)
        .cache_settings(cache_settings)
        .await
        .expect("Unable to create client");

    let shard_manager = client.shard_manager.clone();
    client
        .data
        .write()
        .await
        .insert::<ShardManagerContainer>(shard_manager);

    let http = client.http.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(60 * 5)).await;
            tasks::check_expiring_bans(&http).await;
            tasks::check_expiring_timeouts(&http).await;
        }
    });

    if let Err(e) = client.start().await {
        error!("Client error: {e:?}")
    }
}

#[allow(unreachable_code, dead_code)]
fn update(arg: &str) -> std::io::Result<()> {
    let exe = env::current_exe()?;

    let name = "Ouroboros.exe";

    let mut target = exe.parent().unwrap().to_path_buf();
    target.push(name);

    if target.exists() {
        fs::remove_file(&target)?;
    }

    fs::copy(&exe, &target)?;

    let id = arg.split("=").last().unwrap_or("");

    match SystemCommand::new(format!(".{}{}", std::path::MAIN_SEPARATOR, name))
        .arg(format!("--id={id}"))
        .spawn()
    {
        Ok(c) => drop(c),
        Err(e) => error!("Could not spawn new process; err = {e:?}"),
    };

    Ok(())
}

fn cleanup() -> std::io::Result<()> {
    let current_dir = std::env::current_dir()?;

    for entry in fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && let Some(filename) = path.file_name().and_then(|f| f.to_str())
            && filename.starts_with("new_")
            && filename.contains("ouroboros")
        {
            fs::remove_file(&path)?;
            warn!("Deleted file; {}", filename);
        }
    }

    Ok(())
}
