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
