# Phase 2: Core Functionality — Task Tracker

**Goal**: Implement the core gateway logic: provider abstraction, request routing, HTTP API endpoints, and SSE streaming passthrough.

## Overview

Phase 2 builds on Phase 1's foundation to deliver the core functionality: routing requests to providers, serving HTTP API endpoints, and handling SSE streaming. This phase requires Phase 1 to be complete (all crates must compile).

## Sub-Agent Tasks

### Task 2.1: Implement request routing logic
**Status**: ⬜ Not Started
**Priority**: 🔴 Critical — core routing is needed for the API to function
**Estimated effort**: Medium

**Objective**: Implement a routing layer that selects the appropriate provider based on configuration and request parameters.

**Checklist**:
- [ ] Create `crates/gateway-core/src/router/mod.rs`
  - Define `Router` struct that holds provider instances and routing config
  - Implement `Router::new(config: &GatewayConfig)` — creates providers from config
  - Implement `Router::route(request: &ChatRequest) -> &dyn Provider` — selects provider
  - Implement `Router::get_provider(name: &str) -> Result<&dyn Provider, GatewayError>`
- [ ] Create `crates/gateway-core/src/router/strategies.rs`
  - Implement `RouteStrategy` enum: `Default`, `Fallback`, `RoundRobin`
  - Implement `resolve_provider()` that returns the correct provider based on strategy
- [ ] Update `crates/gateway-core/src/lib.rs` to include `pub mod router;`
- [ ] Add `pub use router::Router;` re-export
- [ ] Run `cargo build -p gateway-core` and verify it compiles
- [ ] Add tests for routing logic
  - Test default provider selection
  - Test fallback provider selection
  - Test provider not found error
  - Test model not supported error

**Notes**:
- Use `Arc<dyn Provider>` for shared provider instances
- The `Router` should be wrapped in `Arc` for shared state across handlers
- Providers are created from `GatewayConfig::providers` using their `api_key` and `base_url`

---

### Task 2.2: Implement HTTP API handlers with SSE streaming
**Status**: ⬜ Not Started
**Priority**: 🔴 Critical — core API functionality
**Estimated effort**: Large

**Objective**: Create functional HTTP handlers for chat completions (with SSE streaming) and health checks.

**Checklist**:
- [ ] Update `crates/gateway-api/src/handlers/chat.rs`
  - Implement `POST /v1/chat/completions` handler
  - Parse `ChatRequest` from JSON body
  - Route to appropriate provider using `Router`
  - Handle `stream: true` — return SSE stream via `axum::response::sse::Sse`
  - Handle `stream: false` — return `ChatResponse` as JSON
  - Implement SSE event formatting: `data: {chunk}\n\n`
  - Handle errors with proper HTTP status codes (via `GatewayError` -> `(StatusCode, Json<Value>)`)
- [ ] Update `crates/gateway-api/src/handlers/health.rs`
  - Return `200 OK` with `{"status": "ok", "providers": [...]}`
- [ ] Update `crates/gateway-api/src/handlers/mod.rs`
  - Re-export handlers
- [ ] Update `crates/gateway-api/src/lib.rs`
  - Wire up routes: `.route("/v1/chat/completions", post(chat_handler))`, `.route("/health", get(health_handler))`
  - Add CORS middleware via `tower_http::cors::CorsLayer`
  - Add request tracing middleware via `tower_http::trace::TraceLayer`
  - Create `AppState` struct holding `Router` (from gateway-core) and config
- [ ] Run `cargo build -p gateway-api` and verify it compiles
- [ ] Run `cargo clippy -p gateway-api` and fix any warnings

**Notes**:
- SSE streaming: Use `axum::response::sse::Sse` with a `Stream` adapter
- For streaming, the handler should:
  1. Call `provider.stream_chat(request)` to get a `BoxStream`
  2. Map each `ChatChunk` to an SSE event string: `data: {json}\n\n`
  3. Return `Sse<impl Stream<Item = Result<Event, Infallible>>>`
- For non-streaming, return `Json<ChatResponse>` directly
- The `AppState` should be shared via `Extension` or `State` in axum

---

### Task 2.3: Implement request/response transformation
**Status**: ⬜ Not Started
**Priority**: 🟡 Medium — normalizes responses across providers
**Estimated effort**: Medium

**Objective**: Create a transformation layer that normalizes provider-specific request/response formats to the unified gateway format.

**Checklist**:
- [ ] Create `crates/gateway-core/src/providers/transform.rs`
  - Implement `Transform` trait: `fn transform_request(req: &ChatRequest) -> serde_json::Value`
  - Implement `fn transform_response(resp: serde_json::Value) -> ChatResponse`
  - Implement `fn transform_stream_chunk(chunk: serde_json::Value) -> ChatChunk`
- [ ] Update `crates/gateway-core/src/lib.rs` to include the transform module
- [ ] Add tests for transformation logic
  - Test OpenAI request transformation
  - Test Anthropic request transformation
  - Test response normalization
  - Test stream chunk transformation

**Notes**:
- Each provider currently does its own transformation in its implementation
- The transformation layer should be a shared utility that providers can use
- This is optional for now — providers can continue to do their own transformation

---

### Task 2.4: Add provider selection based on request model
**Status**: ⬜ Not Started
**Priority**: 🟡 Medium — allows automatic provider selection
**Estimated effort**: Small

**Objective**: Allow the gateway to automatically select a provider based on the model requested in the `ChatRequest`.

**Checklist**:
- [ ] Update `crates/gateway-core/src/router/mod.rs`
  - Add `select_provider_for_model(model: &str)` method
  - Iterate through providers and find one that supports the model
  - Return error if no provider supports the model
- [ ] Update `crates/gateway-api/src/handlers/chat.rs`
  - Use the model-based provider selection if `request.provider` is `None`
  - Use the specified provider if `request.provider` is `Some`
- [ ] Add tests for model-based provider selection
  - Test that correct provider is selected for known model
  - Test that error is returned for unknown model

---

### Task 2.5: Add OpenAI-compatible /v1/models endpoint
**Status**: ⬜ Not Started
**Priority**: 🟢 Low — useful but not critical
**Estimated effort**: Small

**Objective**: Implement a `/v1/models` endpoint that lists all available models across providers.

**Checklist**:
- [ ] Create `crates/gateway-api/src/handlers/models.rs`
  - Implement `GET /v1/models` endpoint
  - Return list of available models from all providers
  - Return model metadata (provider, name, supported features)
- [ ] Update `crates/gateway-api/src/lib.rs` to add the route
- [ ] Add tests for the models endpoint

---

### Task 2.6: Implement OpenAI provider with full streaming support
**Status**: ⬜ Not Started
**Priority**: 🟢 Low — OpenAI provider already exists, this is for improvements
**Estimated effort**: Medium

**Objective**: Improve the existing OpenAI provider implementation.

**Checklist**:
- [ ] Review and improve `crates/gateway-core/src/providers/openai.rs`
  - Add error handling for edge cases
  - Improve streaming reliability
  - Add connection timeout configuration
- [ ] Add tests for the OpenAI provider
  - Test `complete_chat()` with mock HTTP responses
  - Test `stream_chat()` with mock streaming responses
  - Test model validation
  - Test error handling

---

### Task 2.7: Implement Anthropic provider with full streaming support
**Status**: ⬜ Not Started
**Priority**: 🟢 Low — Anthropic provider already exists, this is for improvements
**Estimated effort**: Medium

**Objective**: Improve the existing Anthropic provider implementation.

**Checklist**:
- [ ] Review and improve `crates/gateway-core/src/providers/anthropic.rs`
  - Add error handling for edge cases
  - Improve streaming reliability
  - Add connection timeout configuration
- [ ] Add tests for the Anthropic provider
  - Test `complete_chat()` with mock HTTP responses
  - Test `stream_chat()` with mock streaming responses
  - Test model validation
  - Test error handling

---

## Dependencies & Ordering

```
Task 2.1 (Router) ──────────── Must be done before Task 2.2
Task 2.2 (HTTP handlers) ──── Depends on Task 2.1
Task 2.3 (Transform) ──────── Can be parallelized with Tasks 2.1-2.2
Task 2.4 (Model selection) ── Depends on Task 2.1
Task 2.5 (Models endpoint) ── Depends on Task 2.2
Task 2.6 (OpenAI improvements) ── Can be parallelized
Task 2.7 (Anthropic improvements) ── Can be parallelized
```

## Success Criteria

- [ ] `cargo build --workspace` compiles successfully
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `POST /v1/chat/completions` returns a valid response
- [ ] `POST /v1/chat/completions` with `stream: true` returns SSE events
- [ ] `GET /health` returns a valid response
- [ ] Provider routing works correctly (default + fallback)
- [ ] Error responses have correct HTTP status codes
