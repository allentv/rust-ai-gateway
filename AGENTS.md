# Rust AI Gateway — AI Session Guidelines

This document provides guidelines for AI sessions working on the Rust AI Gateway project. It covers project conventions, code style, testing requirements, and best practices.

## Project Context

The Rust AI Gateway is a high-throughput, low-latency proxy service built with Rust's async ecosystem. It routes requests across multiple AI providers (OpenAI, Anthropic, etc.) with unified APIs, rate limiting, caching, and observability.

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
│   └── skills/                   # Agent skill definitions
├── AGENTS.md                     # This file
└── RUNBOOK.md                    # Project runbook
```

## Code Style & Conventions

### Rust Edition
- **Edition**: 2021

### Formatting
- Use `cargo fmt` before committing — all code must be formatted
- Run `cargo fmt --check` to verify formatting

### Linting
- Use `cargo clippy --workspace` — fix all warnings before committing
- Run `cargo clippy --workspace -- -D warnings` to enforce

### Error Handling
- Use `thiserror` for library error types
- Use `anyhow` for application-level errors
- All `GatewayError` variants must map to appropriate HTTP status codes

### Async
- Use `tokio` as the async runtime
- Use `async-trait` for async trait methods

### Serialization
- Use `serde` with derive macros for all serializable types

### Logging
- Use `tracing` (not `log`) for all logging and instrumentation

### Naming
- Follow Rust API Guidelines:
  - `snake_case` for functions/variables
  - `CamelCase` for types
  - `SCREAMING_SNAKE_CASE` for constants

## Testing Requirements

### General Rules
- Run `cargo test --workspace` to execute all tests
- Run `cargo test -p <crate-name>` to test a specific crate
- Tests live in `#[cfg(test)] mod tests` blocks within source files
- When modifying code, ensure all existing tests still pass
- Add tests for new functionality where practical

### Test Commands
```bash
# Run all tests
mise run test

# Run all tests with output
mise run test-output

# Run tests for a specific crate
mise run test-crate gateway-core
```

### Test Best Practices
1. **Test isolation**: Each test should be independent and not rely on other tests
2. **Test naming**: Use descriptive test names that explain what is being tested
3. **Test coverage**: Aim for high test coverage, especially for core logic
4. **Test structure**: Use `#[cfg(test)] mod tests` blocks within source files
5. **Test assertions**: Use clear assertions with descriptive messages

## Build & CI

### Build Commands
```bash
# Development build
mise run build

# Release build
mise run build-release

# Clean build artifacts
mise run clean
```

### Linting & Formatting
```bash
# Format all code
mise run fmt

# Check formatting without modifying files
mise run fmt-check

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

## Provider Implementation Pattern

When adding a new provider:

1. **Create a new file** in `crates/gateway-core/src/providers/`
2. **Implement the `Provider` trait** from `traits.rs`
3. **Add the provider to the `mod.rs` re-exports**
4. **Add configuration schema** in `gateway-config/src/schema.rs`
5. **Update validation logic** in `gateway-config/src/validation.rs`

### Provider Trait

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

- Configuration supports both YAML and TOML formats
- Environment variables can be referenced as `${VAR_NAME}`
- Config validation is done in `gateway-config/src/validation.rs`
- Schema definitions are in `gateway-config/src/schema.rs`

### Configuration Example

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4

providers:
  openai:
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://api.openai.com/v1"
    models:
      - gpt-4
      - gpt-3.5-turbo
    rate_limit:
      requests_per_minute: 60
      tokens_per_minute: 100000
```

## Dependencies

- Prefer workspace-level dependency definitions in root `Cargo.toml`
- Use `workspace = true` in individual crate `Cargo.toml` files for shared deps
- Avoid adding unnecessary dependencies — justify each new dependency

## Common Pitfalls

### Streaming
- The gateway must handle SSE streaming without buffering
- Use `reqwest`'s streaming capabilities
- Implement `Stream` for SSE responses
- Use `tokio::sync::mpsc` for streaming relay

### Error Mapping
- All `GatewayError` variants must map to appropriate HTTP status codes
- Implement `From<GatewayError> for (StatusCode, Json<Value>)`

### Provider Isolation
- Each provider implementation should be self-contained
- Do not depend on other providers

### Config Validation
- Configuration must be validated at startup, not at request time
- Use `gateway-config/src/validation.rs` for validation logic

## Performance Considerations

1. **Zero-copy parsing** - Use `bytes::Bytes` for request/response bodies
2. **Connection pooling** - `reqwest` connection pool for provider API calls
3. **Memory efficiency** - `Arc`, `RwLock`, and `ArcStr` for shared state
4. **Async I/O** - Non-blocking I/O with tokio
5. **Streaming** - Byte-streaming responses without buffering
6. **Concurrency** - Leverage Rust's fearless concurrency with async

## Development Workflow

### For AI Sessions

1. **Understand the task**: Read the task description and understand the requirements
2. **Explore the codebase**: Use `read_file`, `search_files`, and `list_files` to understand the project structure
3. **Check existing code**: Look for similar patterns and implementations
4. **Implement changes**: Make changes following the code style and conventions
5. **Test changes**: Run `mise run test` to verify changes
6. **Lint and format**: Run `mise run lint` and `mise run fmt` to ensure code quality
7. **Commit changes**: Follow the project's commit message conventions

### Common Tasks

#### Adding a New Provider
1. Create a new file in `crates/gateway-core/src/providers/`
2. Implement the `Provider` trait
3. Add the provider to `mod.rs`
4. Add configuration schema in `gateway-config/src/schema.rs`
5. Update validation in `gateway-config/src/validation.rs`
6. Add tests for the new provider

#### Adding Middleware
1. Create a new file in `crates/gateway-core/src/middleware/`
2. Implement the middleware using Tower's middleware pattern
3. Add the middleware to the server pipeline
4. Add tests for the middleware

#### Fixing Bugs
1. Understand the bug and reproduce it
2. Locate the relevant code
3. Fix the issue
4. Add tests to prevent regression
5. Run `mise run check` to verify

## Agent Commands

### Build
```bash
# Development build
mise run build

# Release build
mise run build-release
```

### Check
```bash
# Full health check
mise run check
```

### Format
```bash
# Format all code
mise run fmt

# Check formatting
mise run fmt-check
```

### Test
```bash
# Run all tests
mise run test

# Run tests with output
mise run test-output

# Run tests for specific crate
mise run test-crate gateway-core
```

### Lint
```bash
# Run clippy
mise run lint
```

## Skills

### Adding Middleware
- Create a new file in `crates/gateway-core/src/middleware/`
- Implement the middleware using Tower's middleware pattern
- Add the middleware to the server pipeline
- Add tests for the middleware

### Adding Provider
- Create a new file in `crates/gateway-core/src/providers/`
- Implement the `Provider` trait from `traits.rs`
- Add the provider to `mod.rs` re-exports
- Add configuration schema in `gateway-config/src/schema.rs`
- Update validation in `gateway-config/src/validation.rs`

### Adding Tests
- Add tests in `#[cfg(test)] mod tests` blocks within source files
- Use descriptive test names
- Test edge cases and error conditions
- Ensure tests are independent and isolated

### Debugging Errors
- Check error messages and stack traces
- Use `cargo clippy --workspace` for lint warnings
- Use `cargo fmt --check` for formatting issues
- Use `cargo test --workspace` for test failures

## Version Information

- **Rust version**: 1.96.0
- **Cargo version**: 1.96.0
- **Rustfmt version**: 1.9.0-stable
- **Clippy version**: 0.1.96

## Tools

- **mise**: Toolchain manager (version 2026.5.16 or later)
- **cargo**: Rust package manager
- **rustfmt**: Code formatter
- **clippy**: Linter

---

*This document provides guidelines for AI sessions working on the Rust AI Gateway project. Keep it updated as the project evolves.*
