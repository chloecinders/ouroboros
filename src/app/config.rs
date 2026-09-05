use std::path::Path;

use serde::Deserialize;

pub const PATH: &str = "./Config.toml";
pub const DEFAULT_WEB_URL: &str = "http://localhost:3000";

#[derive(Debug, Deserialize)]
pub struct File {
    pub bot: Bot,
    pub release: Environment,
    pub dev: Option<Environment>,
}

#[derive(Debug, Deserialize)]
pub struct Bot {
    pub env: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Environment {
    pub token: String,
    pub prefix: String,
    pub database_url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub dev_ids: Option<Vec<u64>>,
    pub whitelist_enabled: Option<bool>,
    pub whitelist: Option<Vec<u64>>,
    pub repository: Option<String>,
    pub github_token: Option<String>,
    pub webhook: Option<String>,
    pub web_port: Option<u16>,
    pub web_url: Option<String>,
    pub discord_client_id: Option<String>,
    pub discord_client_secret: Option<String>,
    pub edit_window_secs: Option<u64>,
    #[serde(default)]
    pub s3: Option<S3>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct S3 {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub public_base_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("could not read {PATH}: {0}")]
    Unreadable(#[from] std::io::Error),
    #[error("could not parse {PATH}: {0}")]
    Malformed(#[from] toml::de::Error),
    #[error("bot.env is {0}, expected release or dev")]
    UnknownEnvironment(String),
    #[error("bot.env is dev but no [dev] section exists")]
    MissingEnvironment,
}

pub fn load() -> Result<Environment, Failure> {
    let raw = std::fs::read_to_string(Path::new(PATH))?;
    let file: File = toml::from_str(&raw)?;

    match file.bot.env.as_str() {
        "release" => Ok(file.release),
        "dev" => file.dev.ok_or(Failure::MissingEnvironment),
        other => Err(Failure::UnknownEnvironment(other.to_string())),
    }
}

impl Environment {
    pub fn is_developer(&self, user: u64) -> bool {
        self.dev_ids.as_ref().is_some_and(|ids| ids.contains(&user))
    }

    pub fn allows_guild(&self, guild: u64) -> bool {
        if !self.whitelist_enabled.unwrap_or(false) {
            return true;
        }

        self.whitelist
            .as_ref()
            .is_some_and(|allowed| allowed.contains(&guild))
    }

    pub fn edit_window(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.edit_window_secs.unwrap_or(300) as i64)
    }

    pub fn site(&self) -> String {
        self.web_url
            .as_deref()
            .unwrap_or(DEFAULT_WEB_URL)
            .trim_end_matches('/')
            .to_string()
    }
}
