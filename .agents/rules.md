# Rust AI Gateway — Project Rules

## Project Context

This is a Rust workspace project implementing a high-throughput AI/LLM gateway proxy. The gateway routes requests across multiple AI providers (OpenAI, Anthropic, etc.) with unified APIs, rate limiting, caching, and observability.

## Workspace Structure

- **Workspace root**: `Cargo.toml` at project root defines 4 member crates
- **`gateway-core`**: Core library with provider traits, domain types, error types, and provider implementations
- **`gateway-api`**: Axum-based HTTP server (binary crate)
- **`gateway-cli`**: CLI tool using clap (binary crate)
- **`gateway-config`**: Configuration schemas and validation (library crate)

## Code Style & Conventions

- **Rust edition**: 2021
- **Formatter**: Use `cargo fmt` before committing — all code must be formatted
- **Linter**: Use `cargo clippy --workspace` — fix all warnings before committing
- **Error handling**: Use `thiserror` for library error types, `anyhow` for application-level errors
- **Async**: Use `tokio` as the async runtime, `async-trait` for async trait methods
- **Serialization**: Use `serde` with derive macros for all serializable types
- **Logging**: Use `tracing` (not `log`) for all logging and instrumentation
- **Naming**: Follow Rust API Guidelines — snake_case for functions/variables, CamelCase for types, SCREAMING_SNAKE_CASE for constants

## Testing

- Run `cargo test --workspace` to execute all tests
- Run `cargo test -p <crate-name>` to test a specific crate
- **Tests must always be in separate files** — never write inline `#[cfg(test)] mod tests { ... }` blocks in source files. Instead:
  - Place tests in a sibling `tests.rs` file (e.g., `foo.rs` → `foo/mod.rs` + `foo/tests.rs`, or `bar/mod.rs` → `bar/tests.rs`)
  - Use `#[cfg(test)] mod tests;` at the bottom of the source file to reference the test module
  - The test file should use `use super::*;` to import everything from the parent module
- When modifying code, ensure all existing tests still pass
- Add tests for new functionality where practical

## Build & CI

- `cargo build` — development build
- `cargo build --release` — optimized release build
- `cargo check --workspace` — quick compilation check
- `cargo clippy --workspace` — lint check
- `cargo fmt --check` — formatting check (use `cargo fmt` to fix)

## Provider Implementation Pattern

When adding a new provider:
1. Create a new file in `crates/gateway-core/src/providers/`
2. Implement the `Provider` trait from `traits.rs`
3. Add the provider to the `mod.rs` re-exports
4. Add configuration schema in `gateway-config/src/schema.rs`
5. Update validation logic in `gateway-config/src/validation.rs`

## Configuration

- Configuration supports both YAML and TOML formats
- Environment variables can be referenced as `${VAR_NAME}`
- Config validation is done in `gateway-config/src/validation.rs`
- Schema definitions are in `gateway-config/src/schema.rs`

## Dependencies

- Prefer workspace-level dependency definitions in root `Cargo.toml`
- Use `workspace = true` in individual crate `Cargo.toml` files for shared deps
- Avoid adding unnecessary dependencies — justify each new dependency

## Common Pitfalls

- **Streaming**: The gateway must handle SSE streaming without buffering — use `reqwest`'s streaming capabilities
- **Error mapping**: All `GatewayError` variants must map to appropriate HTTP status codes via the `From<GatewayError> for (StatusCode, Json<Value>)` implementation
- **Provider isolation**: Each provider implementation should be self-contained and not depend on other providers
- **Config validation**: Configuration must be validated at startup, not at request time
