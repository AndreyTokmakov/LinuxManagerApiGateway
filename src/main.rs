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
use crate::config::Config;

#[path = "ssh/ssh_connection_pool.rs"] pub mod ssh_connection_pool;
#[path = "api/api.rs"] pub mod api;
mod config;

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

// TODO: Features (App spesific)
//  - Send ZMQ command


// NOTE:
//   How to run: "target/debug/LinuxManagerApiGateway  -f src/config.toml"

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
    let logLevel: Level = Level::from_str(&cfg.logging.level)?;

    tracing_subscriber::fmt()
        .with_max_level(logLevel)
        .init();

    let runner: SshCommandRunner = get_ssh_cmd_runner();
    api::run_server(runner, 52525).await?;

    Ok(())
}
