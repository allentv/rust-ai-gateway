# gateway-api — Crate Context

This is the HTTP API layer crate for the Rust AI Gateway. It provides the axum-based HTTP server, handlers, and middleware for the gateway service.

## Crate Structure

```
gateway-api/
├── Cargo.toml             # ✅ Exists — binary crate with [[bin]] name = "gateway-api"
└── src/
    ├── main.rs            # ❌ MISSING — needs to be created (binary entry point)
    ├── lib.rs             # ❌ MISSING — needs to be created (lib root)
    ├── handlers/          # ❌ MISSING — HTTP handler modules
    │   ├── mod.rs
    │   ├── chat.rs        # Chat completions endpoint
    │   └── health.rs      # Health check endpoint
    └── middleware/         # ❌ MISSING — HTTP middleware
        └── mod.rs
```

## Current Status

**This crate has NO source files** — only a `Cargo.toml` exists. Everything in `src/` needs to be created.

## Dependencies

- `tokio` — Async runtime
- `axum` (0.7) — HTTP framework with macros
- `serde` / `serde_json` — Serialization
- `tracing` / `tracing-subscriber` — Logging
- `tower` / `tower-http` — Middleware framework
- `clap` (4) — CLI argument parsing
- `anyhow` — Application-level error handling
- `gateway-core` — Core library (providers, types, errors)
- `gateway-config` — Configuration schemas and validation

## Binary Configuration

The crate is configured as a binary crate with `[[bin]] name = "gateway-api"` and `path = "src/main.rs"`.

## Planned Functionality

### HTTP Endpoints

1. **`POST /v1/chat/completions`** — Chat completions (OpenAI-compatible API)
   - Accepts `ChatRequest` as JSON body
   - Supports both regular and streaming responses
   - Routes to the appropriate provider based on request/config
2. **`GET /health`** — Health check endpoint
   - Returns `200 OK` with status information
3. **`GET /v1/models`** — List available models (planned)

### Middleware Stack

- CORS middleware (via `tower-http`)
- Request tracing middleware (via `tower-http`)
- Rate limiting middleware (planned, from `gateway-core`)
- Authentication middleware (planned, from `gateway-core`)

### Server Configuration

- Reads `GatewayConfig` from `gateway-config`
- Configurable host, port, and worker count
- Graceful shutdown on SIGTERM/SIGINT

## Implementation Pattern

```rust
// src/main.rs pattern (planned)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = gateway_config::validation::load_config_with_env("config/default.yaml")?;
    let app = build_router(config)?;
    let listener = tokio::net::TcpListener::bind(...).await?;
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
    Ok(())
}

// src/handlers/chat.rs pattern (planned)
pub async fn chat_completion(
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Route to provider, return response
}
```

## What's Missing / Needs Implementation

1. **`main.rs`** — Binary entry point with tokio runtime, config loading, and server startup
2. **`lib.rs`** — Module declarations for handlers, middleware
3. **`handlers/chat.rs`** — Chat completions endpoint with streaming support
4. **`handlers/health.rs`** — Health check endpoint
5. **`handlers/mod.rs`** — Handler module re-exports
6. **`middleware/mod.rs`** — Middleware module declarations
7. **Tests** — Integration tests for HTTP endpoints

## Implementation Plan Reference

See `docs/implementation-plan.md` for the full plan. This crate covers:

- **Phase 2**: HTTP API endpoints, SSE streaming passthrough
- **Phase 3**: Middleware integration (rate limiting, auth, telemetry)

## Notes for AI Sessions

- This crate has no source files yet — `src/` directory doesn't exist
- The `Cargo.toml` already has all required dependencies
- Must create `lib.rs` before `main.rs` to avoid compilation errors
- Use `axum::Router` for routing with `tower` middleware
- The gateway should expose an OpenAI-compatible API (`/v1/chat/completions`)
- Streaming responses should use SSE (Server-Sent Events) via `axum::response::sse::Sse`
- The `gateway-core` crate provides `Provider` trait and `GatewayError` for HTTP error mapping
- `GatewayError` already implements `From<GatewayError> for (StatusCode, Json<Value>)` for easy error responses
