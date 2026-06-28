# Rust AI Gateway — Runbook

## Project Overview

The Rust AI Gateway is a high-throughput, low-latency proxy service built with Rust's async ecosystem. It routes requests across multiple AI providers (OpenAI, Anthropic, etc.) with unified APIs, rate limiting, caching, and observability.

## Quick Start

### Prerequisites

- Rust toolchain (managed by mise)
- mise (version 2026.5.16 or later)

### Setup

```bash
# Install mise toolchain
mise install

# Verify installation
mise run build
```

## Development Commands

All commands are managed via `mise run`. Here are the available tasks:

### Building

```bash
# Development build
mise run build

# Release build
mise run build-release

# Clean build artifacts
mise run clean
```

### Testing

```bash
# Run all tests
mise run test

# Run all tests with output
mise run test-output

# Run tests for a specific crate
mise run test-crate gateway-core
mise run test-crate gateway-config
mise run test-crate gateway-api
mise run test-crate gateway-cli
```

### Linting & Formatting

```bash
# Format all code
mise run fmt

# Check formatting without modifying files
mise run fmt-check

# Format a specific crate
mise run fmt-crate gateway-core

# Run clippy linter
mise run lint

# Full health check (lint + fmt-check + test)
mise run check
```

### Other Commands

```bash
# Update dependencies
mise run update

# Generate documentation
mise run doc
```

## Workspace Structure

```
rust-ai-gateway/
├── Cargo.toml                    # Workspace root
├── .mise.toml                    # mise configuration (toolchain + tasks)
├── crates/
│   ├── gateway-core/             # Core library: providers, routing, middleware
│   ├── gateway-api/              # HTTP API layer (axum handlers)
│   ├── gateway-cli/              # CLI tool for configuration and management
│   └── gateway-config/           # Configuration schemas and validation
├── docs/
│   ├── implementation-plan.md    # Detailed implementation plan
│   └── initial-idea.md           # Initial project idea
├── .agents/
│   ├── rules.md                  # Project rules and conventions
│   ├── commands/                 # Agent command definitions
│   │   ├── build.md
│   │   ├── check.md
│   │   ├── fmt.md
│   │   └── test.md
│   └── skills/                   # Agent skill definitions
│       ├── add-middleware.md
│       ├── add-provider.md
│       ├── add-tests.md
│       └── debug-errors.md
├── AGENTS.md                     # AI session guidelines
└── RUNBOOK.md                    # This file
```

## Architecture

### Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| HTTP Server | `axum` | Ergonomic, Tower-based, excellent performance |
| Async Runtime | `tokio` | De facto standard for Rust async |
| Serialization | `serde` + `serde_yaml`/`serde_toml` | De facto standard for Rust configuration |
| HTTP Client | `reqwest` with streaming | Mature, supports streaming and connection pooling |
| Rate Limiting | `governor` | Token bucket rate limiter with backpressure |
| Caching | `moka` | High-performance concurrent cache |
| CLI Parsing | `clap` | Industry standard for Rust CLI tools |
| Error Handling | `thiserror` + `anyhow` | Structured errors with context |
| Logging | `tracing` + `tracing-subscriber` | Structured, async-aware logging |

### Core Traits

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete_chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError>;
    async fn stream_chat(&self, request: ChatRequest) -> Result<ChatStream, ProviderError>;
    fn name(&self) -> &str;
    fn supports_streaming(&self) -> bool;
}
```

## Configuration

Configuration supports both YAML and TOML formats. Environment variables can be referenced as `${VAR_NAME}`.

### Key Configuration Sections

- **Server**: Host, port, workers
- **Providers**: API keys, base URLs, models, rate limits
- **Routing**: Default provider, fallback providers
- **Cache**: Enabled, TTL, max size
- **Telemetry**: Endpoint, service name
- **Metering**: Metrics tracking

### Configuration Files

- Schema definitions: `crates/gateway-config/src/schema.rs`
- Validation logic: `crates/gateway-config/src/validation.rs`

## Troubleshooting

### Build Errors

**Issue**: `error: failed to load manifest for workspace member`

**Solution**: Ensure all crates have proper source files. Check that `src/lib.rs` or `src/main.rs` exists for each crate.

**Issue**: `error: no targets specified in the manifest`

**Solution**: Add `src/lib.rs` for library crates or `src/main.rs` for binary crates.

### Test Failures

**Issue**: Tests fail after code changes

**Solution**: 
1. Run `mise run check` to get a full health check
2. Run `mise run test` to see which tests fail
3. Fix the underlying code issues

**Issue**: Tests are slow

**Solution**: 
1. Run `mise run test-crate <crate-name>` to test only specific crates
2. Use `mise run test-output` to see detailed output

### Linting Issues

**Issue**: Clippy warnings

**Solution**: 
1. Run `mise run lint` to see all warnings
2. Fix warnings before committing
3. Common issues: unused imports, unnecessary clones, type complexity

**Issue**: Formatting issues

**Solution**: 
1. Run `mise run fmt` to auto-format
2. Run `mise run fmt-check` to verify

### Dependency Issues

**Issue**: Dependency conflicts

**Solution**: 
1. Run `mise run update` to update dependencies
2. Check workspace `Cargo.toml` for dependency versions
3. Use `workspace = true` for shared dependencies

## Performance Considerations

1. **Zero-copy parsing** - Use `bytes::Bytes` for request/response bodies
2. **Connection pooling** - `reqwest` connection pool for provider API calls
3. **Memory efficiency** - `Arc`, `RwLock`, and `ArcStr` for shared state
4. **Async I/O** - Non-blocking I/O with tokio
5. **Streaming** - Byte-streaming responses without buffering
6. **Concurrency** - Leverage Rust's fearless concurrency with async

## CI/CD

### GitHub Actions

- **CI Pipeline**: `.github/workflows/ci.yml`
- **Release Automation**: `.github/workflows/release.yml`

### Docker

- **Dockerfile**: `Dockerfile`
- **Docker Compose**: `docker-compose.yml`

## Common Pitfalls

- **Streaming**: The gateway must handle SSE streaming without buffering — use `reqwest`'s streaming capabilities
- **Error mapping**: All `GatewayError` variants must map to appropriate HTTP status codes via the `From<GatewayError> for (StatusCode, Json<Value>)` implementation
- **Provider isolation**: Each provider implementation should be self-contained and not depend on other providers
- **Config validation**: Configuration must be validated at startup, not at request time

## Development Workflow

1. **Local development**: `mise run build` then `cargo run --bin gateway-api`
2. **Testing**: `mise run test`
3. **Linting**: `mise run lint`
4. **Formatting**: `mise run fmt`
5. **Build**: `mise run build-release`
6. **Docker**: `docker build -t rust-ai-gateway .`

---

*This runbook provides a comprehensive guide for working with the Rust AI Gateway. Keep it updated as the project evolves.*
