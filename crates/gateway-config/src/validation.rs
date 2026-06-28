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
mod tests {
    use super::*;

    #[test]
    fn test_resolve_env_vars() {
        std::env::set_var("TEST_API_KEY", "secret123");
        let input = "api_key: \"${TEST_API_KEY}\"";
        let result = resolve_env_vars(input).unwrap();
        assert_eq!(result, "api_key: \"secret123\"");
        std::env::remove_var("TEST_API_KEY");
    }

    #[test]
    fn test_resolve_env_vars_missing() {
        let input = "api_key: \"${NONEXISTENT_VAR_12345}\"";
        let result = resolve_env_vars(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_env_vars_multiple() {
        std::env::set_var("TEST_KEY_A", "alpha");
        std::env::set_var("TEST_KEY_B", "beta");
        let input = "a: ${TEST_KEY_A}, b: ${TEST_KEY_B}";
        let result = resolve_env_vars(input).unwrap();
        assert_eq!(result, "a: alpha, b: beta");
        std::env::remove_var("TEST_KEY_A");
        std::env::remove_var("TEST_KEY_B");
    }

    #[test]
    fn test_resolve_env_vars_no_vars() {
        let input = "no variables here";
        let result = resolve_env_vars(input).unwrap();
        assert_eq!(result, "no variables here");
    }

    // Helper to create a valid config for testing
    fn valid_config() -> crate::schema::GatewayConfig {
        let yaml = r#"
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4
providers:
  openai:
    api_key: "test-key"
    base_url: "https://api.openai.com/v1"
    models:
      - gpt-4
routing:
  default_provider: "openai"
  fallback_providers: []
cache:
  enabled: true
  ttl_seconds: 3600
  max_size: 10000
"#;
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn test_validate_valid_config() {
        let config = valid_config();
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_validate_invalid_port_zero() {
        let mut config = valid_config();
        config.server.port = 0;
        let err = validate(&config).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("port"));
    }

    #[test]
    fn test_validate_empty_providers() {
        let mut config = valid_config();
        config.providers.clear();
        let err = validate(&config).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_validate_empty_api_key() {
        let mut config = valid_config();
        config.providers.get_mut("openai").unwrap().api_key = String::new();
        let err = validate(&config).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("API key"));
    }

    #[test]
    fn test_validate_empty_base_url() {
        let mut config = valid_config();
        config.providers.get_mut("openai").unwrap().base_url = String::new();
        let err = validate(&config).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("base URL"));
    }

    #[test]
    fn test_validate_missing_default_provider() {
        let mut config = valid_config();
        config.routing.default_provider = "nonexistent".to_string();
        let err = validate(&config).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("Default provider"));
    }

    #[test]
    fn test_validate_missing_fallback_provider() {
        let mut config = valid_config();
        config
            .routing
            .fallback_providers
            .push("nonexistent".to_string());
        let err = validate(&config).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("Fallback provider"));
    }

    #[test]
    fn test_validate_cache_zero_ttl_when_enabled() {
        let mut config = valid_config();
        config.cache.enabled = true;
        config.cache.ttl_seconds = 0;
        let err = validate(&config).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("TTL"));
    }

    #[test]
    fn test_validate_cache_zero_ttl_when_disabled() {
        let mut config = valid_config();
        config.cache.enabled = false;
        config.cache.ttl_seconds = 0;
        // Should be OK when cache is disabled
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_load_from_yaml_valid() {
        let dir = std::env::temp_dir().join("gateway_test_yaml");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.yaml");
        let yaml = r#"
server:
  host: "127.0.0.1"
  port: 9090
  workers: 2
providers:
  openai:
    api_key: "sk-test"
    base_url: "https://api.openai.com/v1"
    models:
      - gpt-4
routing:
  default_provider: "openai"
  fallback_providers: []
"#;
        std::fs::write(&path, yaml).unwrap();
        let config = load_from_yaml(&path).unwrap();
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.server.host, "127.0.0.1");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_from_toml_valid() {
        let dir = std::env::temp_dir().join("gateway_test_toml");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.toml");
        let toml = r#"
[server]
host = "127.0.0.1"
port = 9090
workers = 2

[providers.openai]
api_key = "sk-test"
base_url = "https://api.openai.com/v1"
models = ["gpt-4"]

[routing]
default_provider = "openai"
fallback_providers = []
"#;
        std::fs::write(&path, toml).unwrap();
        let config = load_from_toml(&path).unwrap();
        assert_eq!(config.server.port, 9090);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_from_json_valid() {
        let dir = std::env::temp_dir().join("gateway_test_json");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.json");
        let json = r#"{
            "server": {
                "host": "127.0.0.1",
                "port": 9090,
                "workers": 2
            },
            "providers": {
                "openai": {
                    "api_key": "sk-test",
                    "base_url": "https://api.openai.com/v1",
                    "models": ["gpt-4"]
                }
            },
            "routing": {
                "default_provider": "openai",
                "fallback_providers": []
            }
        }"#;
        std::fs::write(&path, json).unwrap();
        let config = load_from_json(&path).unwrap();
        assert_eq!(config.server.port, 9090);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_config_auto_detect_yaml() {
        let dir = std::env::temp_dir().join("gateway_test_auto");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.yaml");
        let yaml = r#"
server:
  host: "0.0.0.0"
  port: 3000
  workers: 1
providers:
  openai:
    api_key: "key"
    base_url: "https://api.openai.com/v1"
routing:
  default_provider: "openai"
"#;
        std::fs::write(&path, yaml).unwrap();
        let config = load_config(&path).unwrap();
        assert_eq!(config.server.port, 3000);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_config_auto_detect_toml() {
        let dir = std::env::temp_dir().join("gateway_test_auto_toml");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        let toml = r#"
[server]
host = "0.0.0.0"
port = 3000
workers = 1

[providers.openai]
api_key = "key"
base_url = "https://api.openai.com/v1"

[routing]
default_provider = "openai"
"#;
        std::fs::write(&path, toml).unwrap();
        let config = load_config(&path).unwrap();
        assert_eq!(config.server.port, 3000);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_config_auto_detect_json() {
        let dir = std::env::temp_dir().join("gateway_test_auto_json");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        let json = r#"{
            "server": { "host": "0.0.0.0", "port": 3000, "workers": 1 },
            "providers": { "openai": { "api_key": "key", "base_url": "https://api.openai.com/v1" } },
            "routing": { "default_provider": "openai" }
        }"#;
        std::fs::write(&path, json).unwrap();
        let config = load_config(&path).unwrap();
        assert_eq!(config.server.port, 3000);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_config_unsupported_extension() {
        let dir = std::env::temp_dir().join("gateway_test_unsupported");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.xml");
        std::fs::write(&path, "<config/>").unwrap();
        let err = load_config(&path).unwrap_err();
        assert!(err.to_string().contains("Unsupported"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_config_no_extension() {
        let dir = std::env::temp_dir().join("gateway_test_noext");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config");
        std::fs::write(&path, "data").unwrap();
        let err = load_config(&path).unwrap_err();
        assert!(err.to_string().contains("No file extension"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_config_file_not_found() {
        let result = load_config("/nonexistent/path/config.yaml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::FileRead(_)));
    }

    #[test]
    fn test_load_from_yaml_invalid_yaml() {
        let dir = std::env::temp_dir().join("gateway_test_bad_yaml");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.yaml");
        std::fs::write(&path, "{{{{invalid yaml}}}}").unwrap();
        let err = load_from_yaml(&path).unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_from_yaml_validation_failure() {
        let dir = std::env::temp_dir().join("gateway_test_invalid_yaml");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("invalid.yaml");
        let yaml = r#"
server:
  host: "0.0.0.0"
  port: 0
  workers: 1
providers: {}
routing:
  default_provider: "none"
"#;
        std::fs::write(&path, yaml).unwrap();
        let err = load_from_yaml(&path).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_config_with_env_resolution() {
        std::env::set_var("TEST_GW_PORT", "7777");
        std::env::set_var("TEST_GW_KEY", "env-secret");
        let dir = std::env::temp_dir().join("gateway_test_env");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("env.yaml");
        let yaml = r#"
server:
  host: "0.0.0.0"
  port: ${TEST_GW_PORT}
  workers: 1
providers:
  openai:
    api_key: "${TEST_GW_KEY}"
    base_url: "https://api.openai.com/v1"
routing:
  default_provider: "openai"
"#;
        std::fs::write(&path, yaml).unwrap();
        let config = load_config_with_env(&path).unwrap();
        assert_eq!(config.server.port, 7777);
        assert_eq!(config.providers["openai"].api_key, "env-secret");
        std::fs::remove_dir_all(&dir).unwrap();
        std::env::remove_var("TEST_GW_PORT");
        std::env::remove_var("TEST_GW_KEY");
    }
}
