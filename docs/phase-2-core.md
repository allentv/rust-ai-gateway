# Phase 2: Core Functionality — Task Tracker

**Goal**: Implement the core gateway logic: provider abstraction, request routing, HTTP API endpoints, and SSE streaming passthrough.

## Overview

Phase 2 builds on Phase 1's foundation to deliver the core functionality: routing requests to providers, serving HTTP API endpoints, and handling SSE streaming. **The router is fully implemented and wired to the API layer.** HTTP handlers route requests to the appropriate provider via the `Router`. SSE streaming is fully integrated — streaming responses are relayed as SSE events. Provider implementations (OpenAI, Anthropic) have full streaming support and are connected to the API layer.

## Sub-Agent Tasks

### Task 2.1: Implement request routing logic
**Status**: ✅ Complete
**Priority**: 🔴 Critical — core routing is needed for the API to function
**Estimated effort**: Medium

**Objective**: Implement a routing layer that selects the appropriate provider based on configuration and request parameters.

**Checklist**:
- [x] Create `crates/gateway-core/src/router/mod.rs`
  - `Router` struct holds `HashMap<String, Box<dyn Provider>>`, `default_provider`, and `fallback_providers`
  - `Router::new(config: &GatewayConfig)` — creates providers from config, validates default/fallback providers exist
  - `Router::route(request: ChatRequest)` — routes to primary provider, falls back if model not supported or provider not found
  - `Router::get_provider(name: &str)` — returns provider by name
  - `Router::available_providers()` — lists all provider names
  - `Router::available_models()` — lists all models across providers
  - `Router::is_model_supported(model: &str)` — checks if any provider supports a model
  - `Router::create_provider(name, config)` — factory method creating providers by name (openai, anthropic, google, or custom)
- [x] Update `crates/gateway-core/src/lib.rs` to include `pub mod router;`
- [x] Run `cargo build -p gateway-core` and verify it compiles
- [x] Add tests for routing logic (6 tests in `router/mod.rs`)
  - [x] `test_router_creation` — verifies router creates successfully with valid config
  - [x] `test_available_providers` — verifies provider listing
  - [x] `test_available_models` — verifies model listing across providers
  - [x] `test_is_model_supported` — verifies model support check
  - [x] `test_invalid_default_provider` — verifies error on invalid default provider
  - [x] `test_get_provider` — verifies provider lookup

**Not implemented** (deferred):
- `router/strategies.rs` with `RouteStrategy` enum (Default, Fallback, RoundRobin) — not needed yet, simple default+fallback routing is sufficient

**Notes**:
- Uses `Box<dyn Provider>` for provider instances
- Providers are created from `GatewayConfig::providers` using their `api_key` and `base_url`
- Unknown provider names are treated as custom OpenAI-compatible providers

---

### Task 2.2: Implement HTTP API handlers with SSE streaming
**Status**: ✅ Complete
**Priority**: 🔴 Critical — core API functionality
**Estimated effort**: Large

**Objective**: Create functional HTTP handlers for chat completions (with SSE streaming) and health checks.

**Checklist**:
- [x] `crates/gateway-api/src/handlers/health.rs` — returns `200 OK` with status JSON
- [x] Update `crates/gateway-api/src/handlers/chat.rs`
  - [x] Parse `ChatRequest` from JSON body
  - [x] Validate non-empty messages
  - [x] **Route to appropriate provider using `Router`** — calls `router.route()` for non-streaming, `router.route_stream()` for streaming
  - [x] Handle `stream: true` — return SSE stream via `axum::response::sse::Sse`
  - [x] Handle `stream: false` — return `ChatResponse` as OpenAI-compatible JSON
  - [x] Implement SSE event formatting: `data: {chunk}\n\n` with `[DONE]` terminator
  - [x] Handle errors with proper HTTP status codes (via `GatewayError` -> `(StatusCode, Json<Value>)`)
- [x] `crates/gateway-api/src/handlers/mod.rs` — re-exports handlers (including `models`)
- [x] Update `crates/gateway-api/src/lib.rs`
  - [x] Create `AppState` struct holding `Arc<Router>` (from gateway-core)
  - [x] Wire up routes with shared state via `axum::extract::Extension`
- [x] Update `crates/gateway-api/src/main.rs`
  - [x] Initialize `Router` from config and wrap in `Arc<AppState>`
  - [x] Pass `AppState` to the axum router via `Extension`
  - [x] CORS middleware via `tower_http::cors::CorsLayer`
  - [x] Request tracing middleware via `tower_http::trace::TraceLayer`
- [x] Run `cargo build -p gateway-api` and verify it compiles
- [x] Run `cargo clippy -p gateway-api` and fix any warnings

**Current state**: Fully implemented. The chat handler routes requests to the appropriate provider via `Router`. Non-streaming requests return OpenAI-compatible JSON. Streaming requests return SSE events with proper `data:` formatting and a `[DONE]` terminator. Errors are mapped to appropriate HTTP status codes via `GatewayError`.

**Notes**:
- SSE streaming uses `axum::response::sse::Sse` with `Stream` adapter
- Streaming flow:
  1. `router.route_stream(request)` returns a `BoxStream<Result<ChatChunk, GatewayError>>`
  2. Each `ChatChunk` is mapped to an SSE event with OpenAI-compatible JSON format
  3. Stream ends with a `data: [DONE]` event
  4. Errors mid-stream are sent as error events (HTTP status can't change after streaming starts)
- Non-streaming returns `Json<Value>` with OpenAI-compatible response format
- `AppState` is shared via `axum::extract::Extension(Arc<AppState>)`

---

### Task 2.3: Implement request/response transformation
**Status**: ❌ Not Started
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
**Status**: ✅ Complete
**Priority**: 🟡 Medium — allows automatic provider selection
**Estimated effort**: Small

**Objective**: Allow the gateway to automatically select a provider based on the model requested in the `ChatRequest`.

**Checklist**:
- [x] Update `crates/gateway-core/src/router/mod.rs`
  - [x] `Router::is_model_supported(model)` method exists
  - [x] `Router::route()` iterates through providers and finds one that supports the model
  - [x] Returns `GatewayError::ModelNotSupported` if no provider supports the model
- [x] Update `crates/gateway-api/src/handlers/chat.rs`
  - [x] Uses model-based provider selection when `request.provider` is `None` (handled by `router.route()` / `router.route_stream()` via `resolve_provider()`)
  - [x] Uses specified provider when `request.provider` is `Some`
- [ ] Add tests for model-based provider selection
  - [ ] Test that correct provider is selected for known model
  - [ ] Test that error is returned for unknown model

**Notes**: The router's `route()` method handles model-based selection with fallback. The chat handler delegates to the router, which uses `resolve_provider()` to select the appropriate provider based on the request's `provider` field and `model` field.

---

### Task 2.5: Add OpenAI-compatible /v1/models endpoint
**Status**: ✅ Complete
**Priority**: 🟢 Low — useful but not critical
**Estimated effort**: Small

**Objective**: Implement a `/v1/models` endpoint that lists all available models across providers.

**Checklist**:
- [x] Create `crates/gateway-api/src/handlers/models.rs`
  - [x] Implement `GET /v1/models` endpoint
  - [x] Return list of available models from all providers
  - [x] Return model metadata (id, object, owned_by/provider)
- [x] Add `available_models_with_providers()` to `Router` in `gateway-core`
- [x] Update `crates/gateway-api/src/handlers/mod.rs` to include `models` module
- [x] Add route `/v1/models` in `main.rs`

**Notes**: Uses `Router::available_models_with_providers()` to get model-provider pairs. Returns OpenAI-compatible list format.

---

### Task 2.6: Implement OpenAI provider with full streaming support
**Status**: ⚠️ Partial (implemented, no tests)
**Priority**: 🟢 Low — OpenAI provider already exists, this is for improvements
**Estimated effort**: Medium

**Objective**: Improve the existing OpenAI provider implementation.

**Checklist**:
- [x] `crates/gateway-core/src/providers/openai.rs` — full implementation (302 lines)
  - [x] Supports `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo`, `gpt-4`, `gpt-3.5-turbo`
  - [x] Full SSE streaming with `bytes_stream()` + `filter_map`
  - [x] Uses `reqwest::Client`
- [ ] Review and improve `crates/gateway-core/src/providers/openai.rs`
  - [ ] Add error handling for edge cases
  - [ ] Improve streaming reliability
  - [ ] Add connection timeout configuration
- [ ] Add tests for the OpenAI provider
  - [ ] Test `complete_chat()` with mock HTTP responses
  - [ ] Test `stream_chat()` with mock streaming responses
  - [ ] Test model validation
  - [ ] Test error handling

---

### Task 2.7: Implement Anthropic provider with full streaming support
**Status**: ⚠️ Partial (implemented, no tests)
**Priority**: 🟢 Low — Anthropic provider already exists, this is for improvements
**Estimated effort**: Medium

**Objective**: Improve the existing Anthropic provider implementation.

**Checklist**:
- [x] `crates/gateway-core/src/providers/anthropic.rs` — full implementation (334 lines)
  - [x] Supports `claude-sonnet-4-20250514`, `claude-3-5-sonnet-20241022`, `claude-3-opus`, etc.
  - [x] Handles system message extraction
  - [x] Full SSE streaming with event types (`content_block_delta`, `message_stop`, etc.)
  - [x] Uses `reqwest::Client`
- [ ] Review and improve `crates/gateway-core/src/providers/anthropic.rs`
  - [ ] Add error handling for edge cases
  - [ ] Improve streaming reliability
  - [ ] Add connection timeout configuration
- [ ] Add tests for the Anthropic provider
  - [ ] Test `complete_chat()` with mock HTTP responses
  - [ ] Test `stream_chat()` with mock streaming responses
  - [ ] Test model validation
  - [ ] Test error handling

---

## Dependencies & Ordering

```
Task 2.1 (Router) ──────────── ✅ COMPLETE
Task 2.2 (HTTP handlers) ──── ✅ COMPLETE
Task 2.3 (Transform) ──────── Deferred — providers do their own transformation
Task 2.4 (Model selection) ── ✅ COMPLETE
Task 2.5 (Models endpoint) ── ✅ COMPLETE
Task 2.6 (OpenAI improvements) ── Can be parallelized
Task 2.7 (Anthropic improvements) ── Can be parallelized
```

## Success Criteria

- [x] `cargo build --workspace` compiles successfully
- [x] `POST /v1/chat/completions` returns a valid response from a real provider
- [x] `POST /v1/chat/completions` with `stream: true` returns SSE events
- [x] `GET /health` returns a valid response
- [x] Provider routing works correctly (default + fallback) — **router fully wired to API**
- [x] Error responses have correct HTTP status codes (via `GatewayError` -> `(StatusCode, Json<Value>)`)
- [x] `GET /v1/models` returns list of available models across providers
