use std::path::Path;

use crate::schema::GatewayConfig;

/// Configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    Parse(String),

    #[error("Configuration validation error: {0}")]
    Validation(String),

    #[error("Environment variable not set: {0}")]
    EnvVarMissing(String),
}

/// Load configuration from a YAML file
pub fn load_from_yaml(path: impl AsRef<Path>) -> Result<GatewayConfig, ConfigError> {
    let content = std::fs::read_to_string(path.as_ref())?;
    let config: GatewayConfig =
        serde_yaml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;
    validate(&config)?;
    Ok(config)
}

/// Load configuration from a TOML file
pub fn load_from_toml(path: impl AsRef<Path>) -> Result<GatewayConfig, ConfigError> {
    let content = std::fs::read_to_string(path.as_ref())?;
    let config: GatewayConfig =
        toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;
    validate(&config)?;
    Ok(config)
}

/// Load configuration from a JSON file
pub fn load_from_json(path: impl AsRef<Path>) -> Result<GatewayConfig, ConfigError> {
    let content = std::fs::read_to_string(path.as_ref())?;
    let config: GatewayConfig =
        serde_json::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;
    validate(&config)?;
    Ok(config)
}

/// Auto-detect format from file extension and load configuration
pub fn load_config(path: impl AsRef<Path>) -> Result<GatewayConfig, ConfigError> {
    let path = path.as_ref();
    match path.extension().and_then(|e| e.to_str()) {
        Some("yaml" | "yml") => load_from_yaml(path),
        Some("toml") => load_from_toml(path),
        Some("json") => load_from_json(path),
        Some(ext) => Err(ConfigError::Validation(format!(
            "Unsupported config format: {ext}. Use .yaml, .toml, or .json"
        ))),
        None => Err(ConfigError::Validation(
            "No file extension found. Use .yaml, .toml, or .json".to_string(),
        )),
    }
}

/// Validate the configuration
pub fn validate(config: &GatewayConfig) -> Result<(), ConfigError> {
    // Validate server config
    if config.server.port == 0 {
        return Err(ConfigError::Validation(
            "Server port must be non-zero".to_string(),
        ));
    }

    // Validate providers
    if config.providers.is_empty() {
        return Err(ConfigError::Validation(
            "At least one provider must be configured".to_string(),
        ));
    }

    for (name, provider) in &config.providers {
        if provider.api_key.is_empty() {
            return Err(ConfigError::Validation(format!(
                "Provider '{name}' API key must not be empty"
            )));
        }
        if provider.base_url.is_empty() {
            return Err(ConfigError::Validation(format!(
                "Provider '{name}' base URL must not be empty"
            )));
        }
    }

    // Validate routing
    if !config
        .providers
        .contains_key(&config.routing.default_provider)
    {
        return Err(ConfigError::Validation(format!(
            "Default provider '{}' is not configured",
            config.routing.default_provider
        )));
    }

    for fallback in &config.routing.fallback_providers {
        if !config.providers.contains_key(fallback) {
            return Err(ConfigError::Validation(format!(
                "Fallback provider '{}' is not configured",
                fallback
            )));
        }
    }

    // Validate cache
    if config.cache.ttl_seconds == 0 && config.cache.enabled {
        return Err(ConfigError::Validation(
            "Cache TTL must be non-zero when caching is enabled".to_string(),
        ));
    }

    Ok(())
}

/// Resolve environment variables in configuration values
///
/// Replaces `${VAR_NAME}` patterns with the corresponding environment variable values.
pub fn resolve_env_vars(config_str: &str) -> Result<String, ConfigError> {
    let mut result = config_str.to_string();
    // Find all ${VAR_NAME} patterns
    let mut start = 0;
    while let Some(open_pos) = result[start..].find("${") {
        let abs_pos = start + open_pos;
        if let Some(close_pos) = result[abs_pos..].find('}') {
            let var_name = &result[abs_pos + 2..abs_pos + close_pos];
            match std::env::var(var_name) {
                Ok(value) => {
                    let full_range = abs_pos..abs_pos + close_pos + 1;
                    result.replace_range(full_range, &value);
                    start = abs_pos + value.len();
                }
                Err(_) => {
                    return Err(ConfigError::EnvVarMissing(var_name.to_string()));
                }
            }
        } else {
            break;
        }
    }
    Ok(result)
}

/// Load configuration with environment variable resolution
pub fn load_config_with_env(path: impl AsRef<Path>) -> Result<GatewayConfig, ConfigError> {
    let content = std::fs::read_to_string(path.as_ref())?;
    let resolved = resolve_env_vars(&content)?;

    match path.as_ref().extension().and_then(|e| e.to_str()) {
        Some("yaml" | "yml") => {
            let config: GatewayConfig =
                serde_yaml::from_str(&resolved).map_err(|e| ConfigError::Parse(e.to_string()))?;
            validate(&config)?;
            Ok(config)
        }
        Some("toml") => {
            let config: GatewayConfig =
                toml::from_str(&resolved).map_err(|e| ConfigError::Parse(e.to_string()))?;
            validate(&config)?;
            Ok(config)
        }
        Some("json") => {
            let config: GatewayConfig =
                serde_json::from_str(&resolved).map_err(|e| ConfigError::Parse(e.to_string()))?;
            validate(&config)?;
            Ok(config)
        }
        _ => Err(ConfigError::Validation(
            "Unsupported config format".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests;
