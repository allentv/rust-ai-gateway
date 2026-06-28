# Phase 1: Foundation & Project Scaffolding — Task Tracker

**Goal**: Establish the project structure, dependencies, and core abstractions.

## Overview

Phase 1 covers workspace setup, core types, provider trait, error handling, configuration system, and CLI scaffolding. **Most tasks are complete.** All four crates have source files and the workspace compiles. Configuration files and additional tests remain to be added.

## Sub-Agent Tasks

### Task 1.1: Fix gateway-core compilation (Critical)
**Status**: ✅ Complete
**Priority**: 🔴 Critical — blocks all other crates from compiling
**Estimated effort**: Small

**Objective**: Make `gateway-core` compile successfully.

**Checklist**:
- [x] Create `crates/gateway-core/src/lib.rs` with module declarations
  - Declares: `pub mod error;`, `pub mod types;`, `pub mod providers;`, `pub mod router;`, `pub mod middleware;`
- [x] Create `crates/gateway-core/src/providers/google.rs` (stub)
  - Implements `Provider` trait with placeholder methods
  - Supports placeholder model list (`gemini-pro`)
  - Returns `Err(GatewayError::Internal(...))` for actual API calls
- [x] Create `crates/gateway-core/src/providers/custom.rs` (stub)
  - Implements `Provider` trait with configurable model list
  - Returns placeholder responses for actual API calls
- [x] Verified: `cargo build -p gateway-core` compiles

**Notes**:
- `lib.rs` also re-exports provider types and error/types modules for other crates

---

### Task 1.2: Create gateway-api entry point and basic HTTP server
**Status**: ✅ Complete
**Priority**: 🔴 Critical — blocks gateway-api from compiling
**Estimated effort**: Medium

**Objective**: Create the HTTP server with basic endpoints that can start and handle requests.

**Checklist**:
- [x] Create `crates/gateway-api/src/lib.rs` with module declarations
  - Declares: `pub mod handlers;`, `pub mod middleware;`
- [x] Create `crates/gateway-api/src/main.rs` with binary entry point
  - Full `#[tokio::main]` entrypoint with config loading via `gateway_config::validation::load_config_with_env()`
  - Loads config from CLI-provided path (defaults to `config/default.yaml`)
  - CORS middleware (allow all origins), TraceLayer for HTTP logging
  - Graceful shutdown on Ctrl+C / SIGTERM
  - Tracing subscriber with env filter
- [x] Create `crates/gateway-api/src/handlers/mod.rs` with re-exports
- [x] Create `crates/gateway-api/src/handlers/chat.rs`
  - `POST /v1/chat/completions` endpoint accepts `ChatRequest` as JSON
  - Validates non-empty messages
  - **Currently returns a placeholder/echo response** (not yet routed to providers)
- [x] Create `crates/gateway-api/src/handlers/health.rs`
  - `GET /health` endpoint returns `200 OK` with status JSON
- [x] Create `crates/gateway-api/src/middleware/mod.rs` (placeholder)

**Notes**:
- Chat handler returns a placeholder response — actual provider routing is in Phase 2
- Graceful shutdown is already implemented (Phase 2 dependency)

---

### Task 1.3: Create gateway-cli entry point
**Status**: ✅ Complete
**Priority**: 🟡 Medium — not blocking other crates
**Estimated effort**: Small

**Objective**: Create the CLI tool with basic subcommands.

**Checklist**:
- [x] Create `crates/gateway-cli/src/main.rs` with clap-based CLI
  - Full `#[tokio::main]` entrypoint
  - Uses `clap` derive macros with `Cli` struct and `Commands` enum
  - Subcommands: `Config`, `Status`, `Cache`
- [x] Create `crates/gateway-cli/src/commands/mod.rs` with module declarations and re-exports
- [x] Create `crates/gateway-cli/src/commands/config.rs`
  - `config validate <path>` — validates a config file using `gateway_config::validation::load_config_with_env()`
  - `config show <path>` — shows parsed configuration as JSON
- [x] Create `crates/gateway-cli/src/commands/status.rs`
  - `status` — displays configured providers, models, and server info from config
- [x] Create `crates/gateway-cli/src/commands/cache.rs`
  - `cache clear` — placeholder for future implementation
  - `cache stats` — placeholder for future implementation

---

### Task 1.4: Create default configuration files
**Status**: ❌ Not Started
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
**Status**: ❌ Not Started
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
**Status**: ⚠️ Partial (2 tests exist)
**Priority**: 🟡 Medium — ensures config validation is correct
**Estimated effort**: Small

**Objective**: Add comprehensive tests for configuration validation.

**Checklist**:
- [x] Basic env var resolution tests exist (`test_resolve_env_vars`, `test_resolve_env_vars_missing`)
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
- [ ] Run `cargo test -p gateway-config` and verify all tests pass

---

### Task 1.7: Create gateway-core config.rs (config loading in core)
**Status**: ❌ Not Started
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

- [x] `cargo build --workspace` compiles successfully
- [ ] `cargo clippy --workspace -- -D warnings` passes (needs verification)
- [ ] `cargo test --workspace` passes (needs verification)
- [x] All four crates have source files (no more "MISSING" files)
- [x] `gateway-api` binary can start (even if in stub mode)
- [x] `gateway-cli` binary can parse arguments
