mod cli;
mod core;
mod git;
mod config;
mod external;

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber;

use crate::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    match cli.command {
        Commands::Rules(args) => {
            info!("Running rules command");
            cli::commands::rules::run(args).await?
        }
        Commands::Review(args) => {
            info!("Running review command");
            cli::commands::review::run(args).await?
        }
        Commands::Setup => {
            info!("Running setup command");
            cli::commands::setup::run().await?
        }
    }

    Ok(())
}