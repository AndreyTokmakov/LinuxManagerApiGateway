use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config
{
    pub server: ServerConfig,
    pub connector: ConnectorConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig
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

pub fn loadConfig(path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>>
{
    let content: String = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}