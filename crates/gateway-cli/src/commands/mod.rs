pub mod cache;
pub mod config;
pub mod status;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Validate a configuration file
    Validate {
        /// Path to the configuration file
        path: String,
    },
    /// Show parsed configuration
    Show {
        /// Path to the configuration file
        path: String,
    },
}

#[derive(Subcommand)]
pub enum CacheCommand {
    /// Clear the response cache
    Clear,
    /// Show cache statistics
    Stats,
}
