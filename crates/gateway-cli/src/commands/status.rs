use anyhow::Result;

pub async fn run() -> Result<()> {
    println!("Rust AI Gateway Status");
    println!("=====================");
    println!();
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();

    // Try to load default config
    let config_path = "config/default.yaml";
    match gateway_config::validation::load_config_with_env(config_path) {
        Ok(config) => {
            println!("Configuration: loaded from {}", config_path);
            println!();

            println!("Server:");
            println!("  Host: {}", config.server.host);
            println!("  Port: {}", config.server.port);
            println!("  Workers: {}", config.server.workers);
            println!();

            println!("Providers ({}):", config.providers.len());
            for (name, provider) in &config.providers {
                println!("  {}:", name);
                println!("    Base URL: {}", provider.base_url);
                println!("    Models: {}", provider.models.join(", "));
                if let Some(rate_limit) = &provider.rate_limit {
                    if let Some(rpm) = rate_limit.requests_per_minute {
                        println!("    Rate limit: {} req/min", rpm);
                    }
                    if let Some(tpm) = rate_limit.tokens_per_minute {
                        println!("    Token limit: {} tokens/min", tpm);
                    }
                }
            }
            println!();

            println!("Routing:");
            println!("  Default: {}", config.routing.default_provider);
            println!("  Fallbacks: {:?}", config.routing.fallback_providers);
            println!();

            println!(
                "Cache: {}",
                if config.cache.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            );
            println!(
                "Telemetry: {}",
                if config.telemetry.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            );
            println!(
                "Metering: {}",
                if config.metering.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
        Err(_) => {
            println!("Configuration: not found at {}", config_path);
            println!("  Run 'gateway-cli config validate <path>' to validate a config file");
        }
    }

    Ok(())
}
