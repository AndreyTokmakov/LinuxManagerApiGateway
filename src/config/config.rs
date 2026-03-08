use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use utoipa::openapi::security::Password;

#[derive(Debug, Deserialize)]
pub struct ApiGetawayConfig
{
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ConnectorConfig
{
    pub pool_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig
{
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct HostConfig
{
    pub host: String,
    pub ssh_port: u16,
    pub username: String,
    pub password: String,
    pub private_key_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Config
{
    pub api_gateway: ApiGetawayConfig,
    pub host_config: HostConfig,
    pub connector: ConnectorConfig,
    pub logging: LoggingConfig,
}

pub fn loadConfig(path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>>
{
    let content: String = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}