use config::{Config, ConfigError};
use dotenvy;
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigData {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
    pub model: String,
    pub openai_api_key: String,
    pub openai_base_url: String,
}

fn load_config() -> Result<ConfigData, ConfigError> {
    dotenvy::from_filename("dev.env").ok();
    let config = Config::builder()
        .add_source(config::Environment::default()) //加载环境变量
        .build()?;
    config.try_deserialize()
}

pub static CONFIG: LazyLock<ConfigData> =
    LazyLock::new(|| load_config().expect("Failed to load configuration"));
