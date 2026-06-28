use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub server: ServerConfig,
    pub providers: HashMap<String, ProviderConfig>,
    pub routing: RoutingConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub metering: MeteringConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_workers")]
    pub workers: usize,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_workers() -> usize {
    num_cpus().unwrap_or(4)
}

fn num_cpus() -> Option<usize> {
    std::thread::available_parallelism().ok().map(|n| n.get())
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,
}

/// Rate limiting configuration for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: Option<u32>,
    pub tokens_per_minute: Option<u32>,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub default_provider: String,
    #[serde(default)]
    pub fallback_providers: Vec<String>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
    #[serde(default = "default_max_size")]
    pub max_size: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: default_ttl(),
            max_size: default_max_size(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_ttl() -> u64 {
    3600
}

fn default_max_size() -> u64 {
    10_000
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            service_name: default_service_name(),
        }
    }
}

fn default_service_name() -> String {
    "ai-gateway".to_string()
}

/// Metering configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeteringConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub metrics: Vec<MetricType>,
}

/// Supported metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    RequestCount,
    TokenUsage,
    CostEstimate,
}
