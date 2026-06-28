use anyhow::{Context, Result};
use crate::commands::ConfigCommand;

pub async fn run(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Validate { path } => validate_config(&path).await,
        ConfigCommand::Show { path } => show_config(&path).await,
    }
}

async fn validate_config(path: &str) -> Result<()> {
    tracing::info!("Validating configuration file: {}", path);

    match gateway_config::validation::load_config_with_env(path) {
        Ok(config) => {
            println!("✓ Configuration is valid");
            println!("  Server: {}:{}", config.server.host, config.server.port);
            println!("  Providers: {}", config.providers.len());
            for (name, provider) in &config.providers {
                println!("    - {}: {} models", name, provider.models.len());
            }
            println!("  Default provider: {}", config.routing.default_provider);
            println!("  Fallback providers: {:?}", config.routing.fallback_providers);
            println!("  Cache enabled: {}", config.cache.enabled);
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Configuration is invalid: {}", e);
            std::process::exit(1);
        }
    }
}

async fn show_config(path: &str) -> Result<()> {
    tracing::info!("Loading configuration from: {}", path);

    let config = gateway_config::validation::load_config_with_env(path)
        .context("Failed to load configuration")?;

    println!("Configuration from: {}", path);
    println!();

    // Display as YAML
    let yaml = serde_yaml::to_string(&config)
        .context("Failed to serialize configuration")?;
    println!("{}", yaml);

    Ok(())
}
