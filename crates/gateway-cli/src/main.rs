use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

#[derive(Parser)]
#[command(
    name = "gateway-cli",
    about = "CLI tool for the Rust AI Gateway",
    version,
    long_about = "Command-line interface for managing and interacting with the Rust AI Gateway. \
                  Supports configuration validation, status checking, and cache management."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate and inspect configuration files
    Config {
        #[command(subcommand)]
        command: commands::ConfigCommand,
    },
    /// Show gateway status and configured providers
    Status,
    /// Manage the response cache
    Cache {
        #[command(subcommand)]
        command: commands::CacheCommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let _log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_PKG_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    match cli.command {
        Commands::Config { command } => commands::config::run(command).await,
        Commands::Status => commands::status::run().await,
        Commands::Cache { command } => commands::cache::run(command).await,
    }
}
