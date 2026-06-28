//! Configuration loading and validation for gateway-core.
//!
//! This module provides a thin wrapper around `gateway-config` for use within the core library.

use gateway_config::schema::GatewayConfig;
use gateway_config::validation::ConfigError;

/// Load configuration from a file path with environment variable resolution.
///
/// This wraps `gateway_config::validation::load_config_with_env()`.
pub fn load_config(path: impl AsRef<std::path::Path>) -> Result<GatewayConfig, ConfigError> {
    gateway_config::validation::load_config_with_env(path)
}

/// Validate a `GatewayConfig`.
///
/// This wraps `gateway_config::validation::validate()`.
pub fn validate_config(config: &GatewayConfig) -> Result<(), ConfigError> {
    gateway_config::validation::validate(config)
}
