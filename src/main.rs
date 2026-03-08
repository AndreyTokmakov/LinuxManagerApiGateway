#![allow(
    dead_code,
    unused_imports,
    unused_parens,
    unused_variables,
    non_snake_case
)]

use tracing::{info, debug, warn, error, span, Level};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use clap::{Arg, Command, builder::PathBufValueParser, ArgMatches};
use crate::config::{ApiGetawayConfig, Config, HostConfig};

#[path = "ssh/ssh_connection_pool.rs"] pub mod ssh_connection_pool;
#[path = "service/api_service.rs"] pub mod api_service;
#[path = "config/config.rs"] pub mod config;
mod models;

use crate::ssh_connection_pool::get_ssh_cmd_runner;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

// TODO: * * * Basics * * *
//  - Logging
//  - REST API
//  - Configuration file
//     - Logging level
//     - Key Path
//     - User name

// TODO: Features
//  - Services (list / status / stop / start / disable / enbale)
//  - Network: (interfaces, Rx/Tx, Open ports)

// TODO: Features (App specific)
//  - Send ZMQ command

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let matches: ArgMatches = Command::new("LinuxManagementApiGateway")
        .version("0.1.0")
        .about("Teaches argument parsing")
        .arg(Arg::new("config_file")
            .short('f')
            .long("file")
            .help("Configuration file path")
            .value_parser(PathBufValueParser::default()))
        .get_matches();

    let defaultConfigFile: PathBuf = PathBuf::from("config.toml");
    let cfgFile: &PathBuf = matches.get_one("config_file").unwrap_or(&defaultConfigFile);

    let cfg: Config = config::loadConfig(cfgFile).unwrap();

    let apiGateway: ApiGetawayConfig = cfg.api_gateway;
    let hostConfig: HostConfig = cfg.host_config;
    let logLevel: Level = Level::from_str(&cfg.logging.level)?;

    tracing_subscriber::fmt()
        .with_max_level(logLevel)
        .init();

    let runner: SshCommandRunner = get_ssh_cmd_runner(
        &hostConfig.host,
        hostConfig.ssh_port,
        &hostConfig.username,
        &hostConfig.password,
        env::current_dir()?.join(&hostConfig.private_key_path)
    );
    api_service::run_server(&apiGateway.host, apiGateway.port, runner).await?;

    Ok(())
}

// NOTE:
//   Run    : target/debug/LinuxManagerApiGateway  -f resources/config.toml
//   Swagger: http://0.0.0.0:52525/swagger-ui/

