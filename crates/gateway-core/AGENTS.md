# gateway-core — Crate Context

This is the core library crate for the Rust AI Gateway. It contains provider implementations, error types, shared types, and (planned) routing/middleware logic.

## Crate Structure

```
gateway-core/
├── Cargo.toml
└── src/
    ├── lib.rs              # ❌ MISSING — needs to be created
    ├── error.rs            # ✅ GatewayError enum with HTTP status code mapping
    ├── types.rs            # ✅ Core types: ChatRequest, ChatResponse, Message, Role, TokenUsage, ChatChunk, Delta, RequestId
    └── providers/
        ├── mod.rs          # ✅ Module declarations and re-exports
        ├── traits.rs       # ✅ Provider trait: complete_chat, stream_chat, name, supports_streaming, supported_models
        ├── openai.rs       # ✅ OpenAI provider — full implementation with streaming
        ├── anthropic.rs    # ✅ Anthropic provider — full implementation with streaming
        ├── google.rs       # ❌ MISSING — referenced in mod.rs but not implemented
        └── custom.rs       # ❌ MISSING — referenced in mod.rs but not implemented
```

## Dependencies

Key dependencies: `tokio`, `reqwest` (with streaming), `async-trait`, `futures`, `serde`, `serde_json`, `thiserror`, `tracing`, `chrono`, `uuid`, `governor`, `moka`.

Also depends on: `gateway-config` (sibling crate).

## Key Types

### Error Types (`error.rs`)

- `GatewayError` — Main error enum with variants: `Provider`, `ProviderNotFound`, `ModelNotSupported`, `Timeout`, `RateLimitExceeded`, `Authentication`, `Configuration`, `Serialization`, `Network`, `StreamClosed`, `Internal`
- Implements `From<GatewayError> for (StatusCode, Json<Value>)` for HTTP error mapping
- Helper constructors: `GatewayError::provider()`, `GatewayError::provider_with_source()`

### Core Types (`types.rs`)

- `Message` — Chat message with `role: Role`, `content: String`, `name: Option<String>`
- `Role` — Enum: `System`, `User`, `Assistant`, `Tool`
- `ChatRequest` — Request with `messages`, `model`, `max_tokens`, `temperature`, `stream`, `provider`
- `ChatResponse` — Response with `id`, `content`, `usage`, `model`, `provider`, `created_at`
- `TokenUsage` — Token counts: `prompt_tokens`, `completion_tokens`, `total_tokens`
- `ChatChunk` — Streaming chunk: `id`, `delta: Delta`, `finish_reason`, `usage`
- `Delta` — Streaming delta: `role`, `content`
- `RequestId` — UUID-based request identifier with `new()`, `as_str()`, `Display`

### Provider Trait (`providers/traits.rs`)

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete_chat(&self, request: ChatRequest) -> Result<ChatResponse, GatewayError>;
    async fn stream_chat(&self, request: ChatRequest) -> Result<BoxStream<'static, Result<ChatChunk, GatewayError>>, GatewayError>;
    fn name(&self) -> &str;
    fn supports_streaming(&self) -> bool { true }  // default impl
    fn supported_models(&self) -> Vec<&str>;
    fn supports_model(&self, model: &str) -> bool { ... }  // default impl
}
```

### Provider Implementations

- **OpenAI** (`openai.rs`) — 302 lines, supports `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo`, `gpt-4`, `gpt-3.5-turbo`. Full SSE streaming with `bytes_stream()` + `filter_map`. Uses `reqwest::Client`.
- **Anthropic** (`anthropic.rs`) — 334 lines, supports `claude-sonnet-4-20250514`, `claude-3-5-sonnet-20241022`, etc. Handles system message extraction, SSE streaming with event types (`content_block_delta`, `message_stop`, etc.). Uses `reqwest::Client`.

## What's Missing / Needs Implementation

1. **`lib.rs`** — Must create module declarations for `error`, `types`, `providers` (and future `router`, `middleware`, `config`)
2. **`providers/google.rs`** — Google provider implementation (stub or full)
3. **`providers/custom.rs`** — Custom provider implementation (stub or full)
4. **`router/`** — Request routing logic (planned, not started)
5. **`middleware/`** — Rate limiter, cost meter, cache, auth, telemetry (planned, not started)
6. **`config.rs`** — Configuration loading within core (planned, not started)
7. **Tests** — No unit tests exist for any of the current modules

## Implementation Plan Reference

See `docs/implementation-plan.md` for the full plan. This crate covers:

- **Phase 1**: Core types, provider trait, error handling (partially done)
- **Phase 2**: Provider implementations, routing (providers done, routing not started)
- **Phase 3**: Middleware (not started)

## Notes for AI Sessions

- The `providers/mod.rs` references `google` and `custom` modules that don't exist — this will cause compilation errors until they are created
- The `lib.rs` file is missing — the crate won't compile without it
- When adding new modules, add them to `lib.rs` and update `mod.rs` re-exports
- All provider implementations follow the same pattern: convert internal types to provider-specific API format, make HTTP request, parse response, return unified types
- Streaming uses `futures::stream::BoxStream` and `reqwest`'s `bytes_stream()`
