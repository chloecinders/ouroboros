use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bot: Bot,
    pub release: Environment,
    pub dev: Environment,
}

#[derive(Debug, Deserialize)]
pub struct Bot {
    pub env: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Environment {
    pub token: String,
    pub prefix: String,
    pub database_url: String,
    pub max_connections: u32,
}
