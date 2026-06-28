# Phase 1: Foundation & Project Scaffolding — Task Tracker

**Goal**: Establish the project structure, dependencies, and core abstractions.

## Overview

Phase 1 covers workspace setup, core types, provider trait, error handling, configuration system, and CLI scaffolding. This phase is partially complete.

## Sub-Agent Tasks

### Task 1.1: Fix gateway-core compilation (Critical)
**Status**: ⬜ Not Started
**Priority**: 🔴 Critical — blocks all other crates from compiling
**Estimated effort**: Small

**Objective**: Make `gateway-core` compile successfully.

**Checklist**:
- [ ] Create `crates/gateway-core/src/lib.rs` with module declarations
  - Must declare: `pub mod error;`, `pub mod types;`, `pub mod providers;`
  - Future modules: `pub mod router;`, `pub mod middleware;`, `pub mod config;` (commented out until implemented)
- [ ] Create `crates/gateway-core/src/providers/google.rs` (stub)
  - Implement `Provider` trait with placeholder methods
  - Support placeholder model list
  - Use `unimplemented!()` or return `Err(GatewayError::Internal("not implemented".to_string()))` for actual logic
- [ ] Create `crates/gateway-core/src/providers/custom.rs` (stub)
  - Implement `Provider` trait with placeholder methods
  - Support placeholder model list
  - Use `unimplemented!()` or return `Err(GatewayError::Internal("not implemented".to_string()))` for actual logic
- [ ] Run `cargo build -p gateway-core` and verify it compiles
- [ ] Run `cargo clippy -p gateway-core` and fix any warnings

**Notes**:
- The `providers/mod.rs` already has `pub mod google;` and `pub mod custom;` declarations and re-exports — the stub files must match these exports
- `lib.rs` must also re-export public items for other crates to use

---

### Task 1.2: Create gateway-api entry point and basic HTTP server
**Status**: ⬜ Not Started
**Priority**: 🔴 Critical — blocks gateway-api from compiling
**Estimated effort**: Medium

**Objective**: Create the HTTP server with basic endpoints that can start and handle requests.

**Checklist**:
- [ ] Create `crates/gateway-api/src/lib.rs` with module declarations
  - Declare: `pub mod handlers;`, `pub mod middleware;`
  - Expose a `build_router()` function that takes config and returns `axum::Router`
- [ ] Create `crates/gateway-api/src/main.rs` with binary entry point
  - `#[tokio::main] async fn main() -> anyhow::Result<()>`
  - Load config via `gateway_config::validation::load_config_with_env()`
  - Build router and start server with `axum::serve()`
  - Implement graceful shutdown signal (SIGTERM/SIGINT)
- [ ] Create `crates/gateway-api/src/handlers/mod.rs` with re-exports
- [ ] Create `crates/gateway-api/src/handlers/chat.rs`
  - `POST /v1/chat/completions` endpoint
  - Accept `ChatRequest` as JSON body
  - Route to provider (use default provider from config)
  - Return `ChatResponse` as JSON
  - Support streaming via SSE (use `axum::response::sse::Sse`)
- [ ] Create `crates/gateway-api/src/handlers/health.rs`
  - `GET /health` endpoint
  - Return `200 OK` with status JSON
- [ ] Create `crates/gateway-api/src/middleware/mod.rs` (placeholder)
- [ ] Run `cargo build -p gateway-api` and verify it compiles
- [ ] Run `cargo clippy -p gateway-api` and fix any warnings

**Notes**:
- Use `axum::Router` for routing with `tower` middleware
- The `gateway-core` crate provides `Provider` trait and `GatewayError` for HTTP error mapping
- `GatewayError` already implements `From<GatewayError> for (StatusCode, Json<Value>)` for easy error responses
- For now, the chat handler can be a stub that returns a mock response or delegates to a provider

---

### Task 1.3: Create gateway-cli entry point
**Status**: ⬜ Not Started
**Priority**: 🟡 Medium — not blocking other crates
**Estimated effort**: Small

**Objective**: Create the CLI tool with basic subcommands.

**Checklist**:
- [ ] Create `crates/gateway-cli/src/main.rs` with clap-based CLI
  - Use `clap` derive macros for argument parsing
  - Define top-level `Cli` struct with `#[command]` and `#[derive(Parser)]`
  - Define `Commands` enum with subcommands: `Config`, `Status`, `Cache`
  - `#[tokio::main] async fn main() -> anyhow::Result<()>`
- [ ] Create `crates/gateway-cli/src/commands/mod.rs` with module declarations and re-exports
- [ ] Create `crates/gateway-cli/src/commands/config.rs`
  - `config validate <path>` — Validate a config file using `gateway_config::validation::load_config()`
  - `config show <path>` — Show parsed configuration (print as YAML/JSON)
- [ ] Create `crates/gateway-cli/src/commands/status.rs`
  - `status` — Display configured providers, models, and server info from config
- [ ] Create `crates/gateway-cli/src/commands/cache.rs` (placeholder)
  - `cache clear` — Placeholder (future implementation)
  - `cache stats` — Placeholder (future implementation)
- [ ] Run `cargo build -p gateway-cli` and verify it compiles
- [ ] Run `cargo clippy -p gateway-cli` and fix any warnings

**Notes**:
- The `gateway-config` crate provides `load_config_with_env()` and `validate()` for config operations
- This is a binary crate (not a library), so it only needs `main.rs` and `commands/`

---

### Task 1.4: Create default configuration files
**Status**: ⬜ Not Started
**Priority**: 🟡 Medium — useful for testing and examples
**Estimated effort**: Small

**Objective**: Create example configuration files that can be used for development and testing.

**Checklist**:
- [ ] Create `config/default.yaml` with example configuration
  - Include `server`, `providers`, `routing`, `cache`, `telemetry`, `metering` sections
  - Use environment variable placeholders: `${OPENAI_API_KEY}`, `${ANTHROPIC_API_KEY}`
- [ ] Create `config/example.yaml` with a more detailed example
  - Include all configuration options with comments
  - Show different provider configurations
- [ ] Run `cargo test -p gateway-config` and verify config loading works

---

### Task 1.5: Add tests for gateway-core types and error handling
**Status**: ⬜ Not Started
**Priority**: 🟡 Medium — ensures correctness of core types
**Estimated effort**: Small

**Objective**: Add unit tests for the core types and error modules.

**Checklist**:
- [ ] Add tests to `crates/gateway-core/src/types.rs`
  - Test `TokenUsage::new()` calculates total correctly
  - Test `RequestId::new()` produces valid UUID
  - Test `RequestId::as_str()` returns correct string
  - Test `RequestId::Display` implementation
  - Test `Message` serialization/deserialization
  - Test `ChatRequest` serialization/deserialization
  - Test `ChatResponse` serialization/deserialization
  - Test `ChatChunk` serialization/deserialization
- [ ] Add tests to `crates/gateway-core/src/error.rs`
  - Test `GatewayError::provider()` constructor
  - Test `GatewayError::provider_with_source()` constructor
  - Test `From<GatewayError> for (StatusCode, Json<Value>)` for each variant
  - Test that status codes are correct for each error variant
- [ ] Run `cargo test -p gateway-core` and verify all tests pass

---

### Task 1.6: Add tests for gateway-config validation
**Status**: ⬜ Not Started
**Priority**: 🟡 Medium — ensures config validation is correct
**Estimated effort**: Small

**Objective**: Add comprehensive tests for configuration validation.

**Checklist**:
- [ ] Add tests to `crates/gateway-config/src/validation.rs`
  - Test `load_from_yaml()` with valid YAML
  - Test `load_from_toml()` with valid TOML
  - Test `load_from_json()` with valid JSON
  - Test `load_config()` auto-detects format from extension
  - Test `validate()` catches invalid port (0)
  - Test `validate()` catches empty providers
  - Test `validate()` catches missing API keys
  - Test `validate()` catches missing default provider
  - Test `validate()` catches missing fallback providers
  - Test `validate()` catches zero TTL when cache enabled
  - Test `load_config_with_env()` with environment variables
  - Test `load_config_with_env()` with missing environment variables
- [ ] Run `cargo test -p gateway-config` and verify all tests pass

---

### Task 1.7: Create gateway-core config.rs (config loading in core)
**Status**: ⬜ Not Started
**Priority**: 🟢 Low — not critical for Phase 1
**Estimated effort**: Small

**Objective**: Create a configuration loading module in `gateway-core` that wraps `gateway-config` for use within the core library.

**Checklist**:
- [ ] Create `crates/gateway-core/src/config.rs`
  - Provide a `load_config()` function that wraps `gateway_config::validation::load_config_with_env()`
  - Provide a `validate_config()` function
- [ ] Update `crates/gateway-core/src/lib.rs` to include `pub mod config;`
- [ ] Run `cargo build -p gateway-core` and verify it compiles

---

## Dependencies & Ordering

```
Task 1.1 (Fix gateway-core) ──┐
Task 1.2 (gateway-api) ───────┤── Can be parallelized
Task 1.3 (gateway-cli) ───────┘
Task 1.4 (Config files) ────── Independent
Task 1.5 (Core tests) ──────── Depends on Task 1.1
Task 1.6 (Config tests) ────── Independent (gateway-config already compiles)
Task 1.7 (Core config.rs) ──── Depends on Task 1.1
```

## Success Criteria

- [ ] `cargo build --workspace` compiles successfully
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] All four crates have source files (no more "MISSING" files)
- [ ] `gateway-api` binary can start (even if in stub mode)
- [ ] `gateway-cli` binary can parse arguments
