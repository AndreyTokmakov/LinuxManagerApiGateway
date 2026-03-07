#![allow(
    dead_code,
    unused_imports,
    unused_parens,
    unused_variables,
    non_snake_case
)]

use tracing::{info, debug, warn, error, span, Level};
use tracing_subscriber::fmt::format::FmtSpan;

#[path = "ssh/ssh_connection_pool.rs"] pub mod ssh_connection_pool;
#[path = "api/api.rs"] pub mod api;


use crate::ssh_connection_pool::get_ssh_cmd_runner;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

// TODO:
//  - Logging
//  - REST API

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG) // set log level
        .init();

    // ssh_connection_pool::test_all().await?;

    let runner: SshCommandRunner = get_ssh_cmd_runner();
    api::run_server(runner, 52525).await?;
    Ok(())
}
