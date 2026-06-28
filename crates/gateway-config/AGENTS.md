# gateway-config — Crate Context

This is the configuration crate for the Rust AI Gateway. It provides configuration schemas, validation, and loading for YAML/TOML/JSON configuration files.

## Crate Structure

```
gateway-config/
├── Cargo.toml
└── src/
    ├── lib.rs             # ✅ Module declarations and re-exports
    ├── schema.rs          # ✅ Configuration type definitions
    └── validation.rs      # ✅ Config loading and validation logic
```

## Current Status

**This crate is fully implemented** for Phase 1 — all source files exist and contain working code.

## Dependencies

- `serde` / `serde_json` / `serde_yaml` / `serde_toml` — Serialization for multiple formats
- `thiserror` — Error type definitions
- `anyhow` — Application-level error handling

## Key Types (`schema.rs`)

### `GatewayConfig` (top-level)
- `server: ServerConfig` — Server settings (host, port, workers)
- `providers: HashMap<String, ProviderConfig>` — Provider configurations
- `routing: RoutingConfig` — Default and fallback providers
- `cache: CacheConfig` — Cache settings (enabled, TTL, max size)
- `telemetry: TelemetryConfig` — OpenTelemetry settings
- `metering: MeteringConfig` — Metering/metrics settings

### `ServerConfig`
- `host: String` (default: "0.0.0.0")
- `port: u16` (default: 8080)
- `workers: usize` (default: num_cpus or 4)

### `ProviderConfig`
- `api_key: String`
- `base_url: String`
- `models: Vec<String>`
- `rate_limit: Option<RateLimitConfig>` — Per-provider rate limits

### `RateLimitConfig`
- `requests_per_minute: Option<u32>`
- `tokens_per_minute: Option<u32>`

### `RoutingConfig`
- `default_provider: String`
- `fallback_providers: Vec<String>`

### `CacheConfig`
- `enabled: bool` (default: true)
- `ttl_seconds: u64` (default: 3600)
- `max_size: u64` (default: 10000)

### `TelemetryConfig`
- `enabled: bool` (default: false)
- `endpoint: Option<String>`
- `service_name: String` (default: "ai-gateway")

### `MeteringConfig`
- `enabled: bool` (default: false)
- `metrics: Vec<MetricType>`

### `MetricType` (enum)
- `RequestCount`
- `TokenUsage`
- `CostEstimate`

## Key Functions (`validation.rs`)

### Loading Functions
- `load_from_yaml(path)` — Load from YAML file
- `load_from_toml(path)` — Load from TOML file
- `load_from_json(path)` — Load from JSON file
- `load_config(path)` — Auto-detect format from extension and load
- `load_config_with_env(path)` — Load with environment variable resolution

### Validation
- `validate(config)` — Validate `GatewayConfig` (checks port, providers, routing, cache)
- `resolve_env_vars(config_str)` — Replace `${VAR_NAME}` patterns with env var values

### `ConfigError` (error type)
- `FileRead` — IO error reading config file
- `Parse` — Config parsing error
- `Validation` — Config validation error
- `EnvVarMissing` — Required environment variable not set

## Tests

- `validation.rs` has 2 tests:
  - `test_resolve_env_vars` — Tests env var substitution
  - `test_resolve_env_vars_missing` — Tests missing env var error

## What's Missing / Needs Improvement

1. **More tests** — Additional validation tests for edge cases
2. **Hot-reload support** — Watch for config file changes (planned)
3. **Config merging** — Support for multiple config sources (planned)

## Implementation Plan Reference

See `docs/implementation-plan.md` for the full plan. This crate covers:
- **Phase 1**: Configuration system with YAML/TOML support
- **Phase 3**: Configuration hot-reload (planned)

## Notes for AI Sessions

- This crate is complete for Phase 1 — all source files exist and compile
- The `load_config_with_env()` function is the primary entry point for loading config with env var resolution
- Validation checks: port non-zero, at least one provider, provider API keys non-empty, default provider exists, fallback providers exist, cache TTL non-zero when enabled
- Environment variables are resolved as `${VAR_NAME}` patterns in config file content before parsing
- The schema types use `serde` with `#[serde(default)]` for optional fields and default values
