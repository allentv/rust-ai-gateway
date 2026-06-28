# AI Session Context — Current Project State

**Last updated**: 2026-06-28

This file provides a snapshot of the project's current state for AI sessions. Read this file at the start of each session to avoid re-scanning the entire codebase.

---

## Project Overview

**Rust AI Gateway** — A high-throughput, low-latency proxy service that routes requests across multiple AI providers (OpenAI, Anthropic, Google, custom) with unified APIs, rate limiting, caching, and observability.

- **Language**: Rust (edition 2021)
- **Async runtime**: Tokio
- **HTTP framework**: Axum 0.7
- **Workspace**: 4 crates (`gateway-core`, `gateway-api`, `gateway-cli`, `gateway-config`)

---

## Overall Status

| Phase | Status | Summary |
|-------|--------|---------|
| Phase 1: Foundation | ✅ 100% complete | All crates compile, have source files, config files exist, 64 tests passing, clippy clean |
| Phase 2: Core | ~30% complete | Router fully implemented. API handlers exist but chat returns placeholder responses (not wired to router). No SSE streaming. |
| Phase 3: Middleware | ~40% complete | Core middleware structs exist with tests. **None are Tower layers. None are wired into HTTP pipeline.** No OpenTelemetry. |
| Phase 4: Production | ~10% complete | Graceful shutdown implemented. Basic health check exists. No Docker/K8s/CI/CD/docs/integration tests. |

**Critical gap**: The `Router` exists in gateway-core but the API chat handler doesn't use it — it returns a hardcoded echo response.

---

## Test Status

| Crate | Tests | Status |
|-------|-------|--------|
| `gateway-config` | 25 | ✅ All passing (validation, loading, env var resolution) |
| `gateway-core` | 39 | ✅ All passing (types, errors, router, middleware) |
| `gateway-api` | 0 | — No tests yet |
| `gateway-cli` | 0 | — No tests yet |
| **Total** | **64** | ✅ `cargo test --workspace` passes, `cargo clippy --workspace -- -D warnings` clean |

---

## File Inventory

### `gateway-config` (fully implemented for Phase 1)

| File | Status | Lines | Description |
|------|--------|-------|-------------|
| `src/lib.rs` | ✅ | ~5 | Module declarations, re-exports `schema`, `validation` |
| `src/schema.rs` | ✅ | ~150 | All config types: `GatewayConfig`, `ServerConfig`, `ProviderConfig`, `RateLimitConfig`, `RoutingConfig`, `CacheConfig`, `TelemetryConfig`, `MeteringConfig` (derived Default), `MetricType` |
| `src/validation/mod.rs` | ✅ | ~120 | Config loading (YAML/TOML/JSON), env var resolution (`${VAR}`), validation. References tests module. |
| `src/validation/tests.rs` | ✅ | ~350 | 25 comprehensive tests: env var resolution, validate rules, load from YAML/TOML/JSON, auto-detect format, error cases, env resolution in config loading |

### `gateway-core` (partially implemented)

| File | Status | Lines | Description |
|------|--------|-------|-------------|
| `src/lib.rs` | ✅ | 6 | Module declarations: `config`, `error`, `types`, `providers`, `router`, `middleware` |
| `src/config.rs` | ✅ | ~25 | Thin wrapper around `gateway-config`: `load_config()`, `validate_config()` |
| `src/error/mod.rs` | ✅ | ~110 | `GatewayError` enum (11 variants), HTTP status code mapping, helper constructors. References tests module. |
| `src/error/tests.rs` | ✅ | ~175 | 13 tests: constructor tests, HTTP status code mapping for each variant, Display messages |
| `src/types/mod.rs` | ✅ | ~110 | `Message`, `Role` (System/User/Assistant/Tool), `ChatRequest`, `ChatResponse`, `TokenUsage`, `ChatChunk`, `Delta`, `RequestId`. References tests module. |
| `src/types/tests.rs` | ✅ | ~200 | 13 tests: TokenUsage calculation, RequestId UUID validation, serialization roundtrips, serde defaults, skip_serializing_if behavior |
| `src/providers/traits.rs` | ✅ | ~40 | `Provider` trait: `complete_chat()`, `stream_chat()`, `name()`, `supports_streaming()`, `supported_models()`, `supports_model()` |
| `src/providers/mod.rs` | ✅ | ~10 | Module declarations and re-exports for all providers |
| `src/providers/openai.rs` | ✅ | ~300 | Full implementation. Models: gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-4, gpt-3.5-turbo. SSE streaming via `bytes_stream()`. |
| `src/providers/anthropic.rs` | ✅ | ~325 | Full implementation. Models: claude-sonnet-4-20250514, claude-3-5-sonnet-20241022, claude-3-5-haiku-20241022, claude-3-opus-20240229, claude-3-sonnet-20240229, claude-3-haiku-20240307. SSE streaming with event types. |
| `src/providers/google.rs` | ⚠️ Stub | ~65 | Implements `Provider` trait. Supports gemini-2.0-flash, gemini-2.0-pro, gemini-1.5-flash, gemini-1.5-pro. Returns `Err(Internal)` for API calls. |
| `src/providers/custom.rs` | ⚠️ Stub | ~70 | Generic OpenAI-compatible provider. Configurable model list. Returns placeholder responses. |
| `src/router/mod.rs` | ✅ | ~165 | `Router` struct with `new()`, `route()`, `get_provider()`, `available_providers()`, `available_models()`, `is_model_supported()`. Factory creates providers by name. References tests module. |
| `src/router/tests.rs` | ✅ | ~120 | 6 tests: router creation, available providers/models, model support, invalid config, get provider |
| `src/middleware/mod.rs` | ⚠️ Partial | ~285 | Contains 4 middleware structs (see below). References tests module. **Not Tower layers.** |
| `src/middleware/tests.rs` | ✅ | ~95 | 4 tests: rate limiter creation, cache operations, auth middleware, cost meter |

### Middleware structs in `gateway-core/src/middleware/mod.rs`:

| Struct | Lines | Description |
|--------|-------|-------------|
| `ProviderRateLimiter` | ~50 | Per-provider rate limiting using `governor`. `from_config()`, `check_rate_limit(provider)` |
| `ChatCache` | ~80 | Response cache using `moka::future::Cache`. `get()`, `put()`, `clear()`, `stats()`, hash-based cache key generation |
| `AuthMiddleware` | ~50 | API key validation. `new(api_keys, required)`, `validate_api_key(key)`. Supports optional auth and open mode. |
| `CostMeter` | ~80 | Token usage tracking. `record_request(provider, usage)`, `stats()`. Uses `AtomicU64` + `RwLock`. |

### `gateway-api` (partially implemented)

| File | Status | Lines | Description |
|------|--------|-------|-------------|
| `src/lib.rs` | ✅ | 2 | Module declarations: `pub mod handlers;`, `pub mod middleware;` |
| `src/main.rs` | ✅ | 99 | Full axum server: config loading, CORS, TraceLayer, routes (`/health`, `/v1/chat/completions`), graceful shutdown (ctrl_c + SIGTERM), tracing subscriber |
| `src/handlers/mod.rs` | ✅ | 2 | Re-exports `chat` and `health` |
| `src/handlers/chat.rs` | ⚠️ Stub | 55 | `POST /v1/chat/completions`. Parses `ChatRequest`, validates non-empty messages. **Returns placeholder echo response** — does NOT use Router. Has TODO comment. |
| `src/handlers/health.rs` | ✅ | ~20 | `GET /health` returns `200 OK` with `{"status": "ok"}` |
| `src/middleware/mod.rs` | ✅ | 1 | Empty placeholder module |

### `gateway-cli` (fully implemented for Phase 1)

| File | Status | Lines | Description |
|------|--------|-------|-------------|
| `src/main.rs` | ✅ | ~80 | Full clap CLI: `config` (validate/show), `status`, `cache` (clear/stats placeholders) |
| `src/commands/mod.rs` | ✅ | ~5 | Module declarations |
| `src/commands/config.rs` | ✅ | ~40 | `config validate <path>` and `config show <path>` |
| `src/commands/status.rs` | ✅ | ~30 | Shows providers, models, server info from config |
| `src/commands/cache.rs` | ⚠️ | ~20 | Placeholder subcommands |

### Configuration Files

| File | Status | Description |
|------|--------|-------------|
| `config/default.yaml` | ✅ | Default config with server, OpenAI + Anthropic providers, routing, cache, telemetry, metering. Uses `${ENV_VAR}` placeholders. |
| `config/example.yaml` | ✅ | Detailed example with comments showing all configuration options including Google provider and custom provider examples. |

---

## Key Types Quick Reference

### Request/Response flow types (`gateway-core/src/types/mod.rs`)
```
ChatRequest { messages: Vec<Message>, model: String, max_tokens: Option<u32>,
              temperature: Option<f32>, stream: bool, provider: Option<String> }
ChatResponse { id: String, content: String, usage: TokenUsage, model: String,
               provider: String, created_at: DateTime<Utc> }
Message { role: Role, content: String, name: Option<String> }
Role = System | User | Assistant | Tool
TokenUsage { prompt_tokens: u32, completion_tokens: u32, total_tokens: u32 }
ChatChunk { id: String, delta: Delta, finish_reason: Option<String>, usage: Option<TokenUsage> }
Delta { role: Option<String>, content: Option<String> }
```

### Provider trait (`gateway-core/src/providers/traits.rs`)
```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete_chat(&self, request: ChatRequest) -> Result<ChatResponse, GatewayError>;
    async fn stream_chat(&self, request: ChatRequest) -> Result<BoxStream<'static, Result<ChatChunk, GatewayError>>, GatewayError>;
    fn name(&self) -> &str;
    fn supports_streaming(&self) -> bool { true }
    fn supported_models(&self) -> Vec<&str>;
    fn supports_model(&self, model: &str) -> bool { ... }
}
```

### GatewayError variants (`gateway-core/src/error/mod.rs`)
`Provider`, `ProviderNotFound`, `ModelNotSupported`, `Timeout`, `RateLimitExceeded`, `Authentication`, `Configuration`, `Serialization`, `Network`, `StreamClosed`, `Internal`

Each maps to an HTTP status code via `From<GatewayError> for (StatusCode, Json<Value>)`.

### Config types (`gateway-config/src/schema.rs`)
`GatewayConfig` → `ServerConfig` (host, port, workers), `providers: HashMap<String, ProviderConfig>`, `RoutingConfig` (default_provider, fallback_providers), `CacheConfig` (enabled, ttl_seconds, max_size), `TelemetryConfig`, `MeteringConfig`

---

## What Works Now

1. **Workspace compiles** — `cargo build --workspace` succeeds
2. **64 tests passing** — `cargo test --workspace` succeeds, `cargo clippy --workspace -- -D warnings` clean
3. **Config loading** — YAML/TOML/JSON with `${ENV_VAR}` resolution and validation
4. **Config files exist** — `config/default.yaml` and `config/example.yaml` with full documentation
5. **Router** — Creates providers from config, routes requests with fallback, validates model support
6. **OpenAI/Anthropic providers** — Full `complete_chat()` and `stream_chat()` implementations with SSE parsing
7. **HTTP server** — Starts, serves routes, handles graceful shutdown
8. **Health endpoint** — `GET /health` returns 200
9. **Chat endpoint** — `POST /v1/chat/completions` accepts requests but returns echo/placeholder
10. **CLI** — Config validation, status display
11. **Middleware structs** — Rate limiter, cache, auth, cost meter all have working logic with tests
12. **Tests separated** — All test modules are in dedicated `tests.rs` files for easier maintenance

## What Doesn't Work Yet

1. **Chat handler doesn't use Router** — Returns placeholder instead of routing to providers
2. **No SSE streaming in API** — Provider `stream_chat()` exists but isn't called from any handler
3. **Middleware not wired as Tower layers** — Structs exist but aren't in the HTTP middleware pipeline
4. **No `/v1/models` endpoint**
5. **No `/ready` readiness probe**
6. **No OpenTelemetry**
7. **No Docker/K8s/CI**
8. **No integration tests**
9. **No tests for gateway-api or gateway-cli**
10. **No tests for provider implementations** (only router tests verify provider creation)

---

## Immediate Next Steps (Prioritized)

### 1. Wire Router into Chat Handler (unlocks core functionality)
- Create `AppState` struct with `Router` (wrapped in `Arc`)
- Initialize `Router` in `main.rs` from config
- Update `chat.rs` handler to use `Router::route()` instead of returning placeholder
- This makes `POST /v1/chat/completions` actually call OpenAI/Anthropic

### 2. Add SSE Streaming to Chat Handler
- When `request.stream == true`, call `Router` (needs stream variant) or `Provider::stream_chat()`
- Map `ChatChunk` stream to SSE events
- Return `Sse<impl Stream<Item = Result<Event, Infallible>>>`

### 3. Wire Middleware as Tower Layers
- Wrap `ProviderRateLimiter`, `ChatCache`, `AuthMiddleware`, `CostMeter` as Tower `Service`/`Layer` types
- Add them to the router in `main.rs`

### 4. Add More Tests
- Integration tests for API endpoints (start server, send requests)
- Tests for provider implementations (mock HTTP responses)
- CLI command tests
- Provider trait tests with mock providers

---

## Build & Test Commands

```bash
# Build
mise run build              # Development build
mise run build-release      # Release build

# Test
mise run test               # All tests (64 passing)
mise run test-output        # Tests with output
mise run test-crate gateway-core   # Specific crate

# Lint & Format
mise run lint               # cargo clippy --workspace -- -D warnings
mise run fmt                # cargo fmt
mise run fmt-check          # cargo fmt --check
mise run check              # Full health check (lint + fmt + test)

# Individual cargo commands
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all
cargo fmt --check
```

---

## Dependencies

All workspace dependencies are defined in root `Cargo.toml`:
- `tokio` (full), `axum` 0.7 (macros), `reqwest` 0.12 (stream, json, rustls-tls)
- `serde`/`serde_json`/`serde_yaml`/`toml`, `thiserror`, `anyhow`
- `tracing`/`tracing-subscriber` (env-filter, json)
- `async-trait`, `clap` 4 (derive), `tower` 0.4, `tower-http` 0.5 (cors, trace)
- `futures`, `bytes`, `uuid` (v4), `chrono` (serde), `url`
- `governor` 0.6, `moka` 0.12 (future)

---

## Architecture Notes

- **Provider isolation**: Each provider is self-contained, implements `Provider` trait
- **Router pattern**: `Router` holds `Box<dyn Provider>` map, selects by config or request
- **Error flow**: Provider errors → `GatewayError` → HTTP status code via `From` impl
- **Streaming**: Providers use `reqwest::bytes_stream()` + `futures::StreamExt` to parse SSE
- **Config flow**: YAML file → `load_config_with_env()` → `GatewayConfig` → `Router::new()` → providers
- **Test structure**: Each module has a separate `tests.rs` file referenced via `#[cfg(test)] mod tests;`
- **API flow** (current): Request → parse JSON → placeholder response (Router not wired)
- **API flow** (target): Request → parse JSON → `Router::route()` → `Provider::complete_chat()` → response
