# Phase 3: Middleware & Features — Task Tracker

**Goal**: Add rate limiting, caching, metering, authentication, and observability middleware.

## Overview

Phase 3 adds the middleware layer that provides rate limiting, response caching, cost metering, authentication, and OpenTelemetry integration. These are implemented as Tower middleware layers that compose into the axum server pipeline. Phase 2 must be complete before starting Phase 3.

## Sub-Agent Tasks

### Task 3.1: Implement rate limiter middleware
**Status**: ⬜ Not Started
**Priority**: 🔴 Critical — essential for production use
**Estimated effort**: Medium

**Objective**: Implement token bucket rate limiting per key and per provider using `governor`.

**Checklist**:
- [ ] Create `crates/gateway-core/src/middleware/mod.rs`
  - Declare modules: `pub mod rate_limiter;`, `pub mod cost_meter;`, `pub mod cache;`, `pub mod auth;`, `pub mod telemetry;`
  - Re-export public types
- [ ] Create `crates/gateway-core/src/middleware/rate_limiter.rs`
  - Implement `RateLimiter` struct using `governor`
  - Support per-provider rate limiting (requests per minute, tokens per minute)
  - Support per-key rate limiting (API key based)
  - Implement `is_allowed(key: &str) -> bool` method
  - Implement `get_remaining_tokens(key: &str) -> u32` method
  - Implement `get_remaining_requests(key: &str) -> u32` method
- [ ] Implement as Tower middleware layer
  - Create `RateLimitMiddleware` struct
  - Implement `tower::Layer` for `RateLimitLayer`
  - Implement `tower::Service` for `RateLimitMiddleware`
  - Rate limit should return `429 Too Many Requests` with `Retry-After` header
- [ ] Update `crates/gateway-core/src/lib.rs` to include `pub mod middleware;`
- [ ] Add tests for rate limiting
  - Test that requests are allowed within limit
  - Test that requests are rejected when limit exceeded
  - Test that rate limit resets after window
  - Test per-provider rate limiting
  - Test per-key rate limiting

**Notes**:
- Use `governor` crate for token bucket rate limiting
- Rate limit config comes from `GatewayConfig::providers[name].rate_limit`
- The `RateLimitConfig` has `requests_per_minute` and `tokens_per_minute`

---

### Task 3.2: Implement cost metering
**Status**: ⬜ Not Started
**Priority**: 🟡 Medium — useful for billing and monitoring
**Estimated effort**: Medium

**Objective**: Track token usage and calculate costs per request, key, and provider.

**Checklist**:
- [ ] Create `crates/gateway-core/src/middleware/cost_meter.rs`
  - Implement `CostMeter` struct
  - Implement `track_usage(provider: &str, usage: &TokenUsage)` method
  - Implement `get_usage_stats() -> UsageStats` method
  - Implement `get_provider_stats(provider: &str) -> ProviderStats` method
  - Implement `get_key_stats(key: &str) -> KeyStats` method
- [ ] Implement as Tower middleware layer
  - Create `CostMeterMiddleware` struct
  - Implement `tower::Layer` for `CostMeterLayer`
  - Implement `tower::Service` for `CostMeterMiddleware`
  - Track token usage from response headers or response body
- [ ] Define cost models for each provider
  - Create `pricing.rs` or similar module
  - Define pricing per token for each model
  - Calculate cost from token usage
- [ ] Add tests for cost metering
  - Test that usage is tracked correctly
  - Test that cost is calculated correctly
  - Test that stats are aggregated correctly

**Notes**:
- The `MeteringConfig` has `enabled` flag and `metrics` list
- Metrics include: `RequestCount`, `TokenUsage`, `CostEstimate`
- Use `std::sync::Mutex` or `RwLock` for tracking stats

---

### Task 3.3: Implement response caching
**Status**: ⬜ Not Started
**Priority**: 🟡 Medium — improves performance for repeated requests
**Estimated effort**: Medium

**Objective**: Implement response caching with configurable TTL using `moka`.

**Checklist**:
- [ ] Create `crates/gateway-core/src/middleware/cache.rs`
  - Implement `ResponseCache` struct using `moka`
  - Implement `get(cache_key: &str) -> Option<ChatResponse>` method
  - Implement `put(cache_key: &str, response: &ChatResponse)` method
  - Implement `generate_cache_key(request: &ChatRequest) -> String` method
  - Implement `clear()` method
  - Implement `stats() -> CacheStats` method
- [ ] Implement as Tower middleware layer
  - Create `CacheMiddleware` struct
  - Implement `tower::Layer` for `CacheLayer`
  - Implement `tower::Service` for `CacheMiddleware`
  - Check cache before making request
  - Store response in cache after receiving it
  - Only cache non-streaming responses
- [ ] Add tests for caching
  - Test cache hit returns cached response
  - Test cache miss makes request to provider
  - Test TTL expiration
  - Test cache key generation
  - Test cache clearing

**Notes**:
- Use `moka` crate for high-performance concurrent caching
- Cache config comes from `GatewayConfig::cache`
- `CacheConfig` has `enabled`, `ttl_seconds`, `max_size`
- Cache key should be based on request content (messages, model, temperature, etc.)

---

### Task 3.4: Implement authentication middleware
**Status**: ⬜ Not Started
**Priority**: 🟡 Medium — essential for production use
**Estimated effort**: Small

**Objective**: Implement API key validation middleware.

**Checklist**:
- [ ] Create `crates/gateway-core/src/middleware/auth.rs`
  - Implement `AuthMiddleware` struct
  - Implement `validate_api_key(key: &str) -> bool` method
  - Implement `get_api_key(request: &HttpRequest) -> Option<String>` method
- [ ] Implement as Tower middleware layer
  - Create `AuthLayer` struct
  - Implement `tower::Layer` for `AuthLayer`
  - Implement `tower::Service` for `AuthMiddleware`
  - Check `Authorization: Bearer <key>` header
  - Return `401 Unauthorized` if key is invalid
- [ ] Add tests for authentication
  - Test that valid API key is accepted
  - Test that invalid API key is rejected
  - Test that missing API key is rejected

**Notes**:
- For now, API keys can be hardcoded or from config
- In production, use a database or external service for API key validation

---

### Task 3.5: Implement OpenTelemetry integration
**Status**: ⬜ Not Started
**Priority**: 🟢 Low — observability is important but not critical for Phase 3
**Estimated effort**: Large

**Objective**: Integrate OpenTelemetry for tracing, metrics, and logging.

**Checklist**:
- [ ] Create `crates/gateway-core/src/middleware/telemetry.rs`
  - Implement `TelemetryMiddleware` struct
  - Implement `init_telemetry(config: &TelemetryConfig) -> Result<()>` function
  - Implement `get_tracer() -> Tracer` function
  - Implement `get_meter() -> Meter` function
- [ ] Implement as Tower middleware layer
  - Create `TelemetryLayer` struct
  - Implement `tower::Layer` for `TelemetryLayer`
  - Implement `tower::Service` for `TelemetryMiddleware`
  - Trace incoming requests
  - Trace outgoing provider requests
  - Record latency, token usage, error counts
- [ ] Add metrics
  - Request count per provider
  - Token usage per provider
  - Latency per provider
  - Error rate per provider
- [ ] Add tests for telemetry
  - Test that traces are generated
  - Test that metrics are recorded

**Notes**:
- Use `opentelemetry` crate for OpenTelemetry integration
- Use `tracing` for structured logging
- Telemetry config comes from `GatewayConfig::telemetry`

---

### Task 3.6: Wire up middleware in gateway-api
**Status**: ⬜ Not Started
**Priority**: 🔴 Critical — middleware is useless until wired up
**Estimated effort**: Medium

**Objective**: Integrate all middleware layers into the gateway-api server pipeline.

**Checklist**:
- [ ] Update `crates/gateway-api/src/lib.rs`
  - Add `RateLimitLayer` to the router
  - Add `AuthLayer` to the router
  - Add `CacheLayer` to the router
  - Add `CostMeterLayer` to the router
  - Add `TelemetryLayer` to the router
  - Create `AppState` with all middleware instances
- [ ] Update `crates/gateway-api/src/main.rs`
  - Initialize telemetry on startup
  - Initialize rate limiter from config
  - Initialize cache from config
  - Initialize cost meter from config
- [ ] Add tests for middleware integration
  - Test that middleware is applied correctly
  - Test that middleware works with handlers

---

## Dependencies & Ordering

```
Task 3.1 (Rate limiter) ──── Can be done first
Task 3.2 (Cost metering) ── Can be done in parallel
Task 3.3 (Cache) ─────────── Can be done in parallel
Task 3.4 (Auth) ──────────── Can be done in parallel
Task 3.5 (Telemetry) ────── Can be done in parallel
Task 3.6 (Wire up) ──────── Depends on all other tasks
```

## Success Criteria

- [ ] Rate limiter prevents excessive requests
- [ ] Cost metering tracks token usage correctly
- [ ] Cache stores and retrieves responses correctly
- [ ] Authentication middleware rejects invalid API keys
- [ ] OpenTelemetry traces and metrics are generated
- [ ] All middleware layers are properly wired up in the server pipeline
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
