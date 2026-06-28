# Phase 3: Middleware & Features — Task Tracker

**Goal**: Add rate limiting, caching, metering, authentication, and observability middleware.

## Overview

Phase 3 adds the middleware layer that provides rate limiting, response caching, cost metering, authentication, and OpenTelemetry integration. These are implemented as Tower middleware layers that compose into the axum server pipeline. Phase 2 must be complete before starting Phase 3.

**Current status**: Core middleware structs (`ProviderRateLimiter`, `ChatCache`, `AuthMiddleware`, `CostMeter`) are all implemented in `crates/gateway-core/src/middleware/mod.rs` with unit tests. However, **none of them are implemented as Tower middleware layers** and **none are wired into the HTTP pipeline**. They exist as standalone utility structs that could be called from handlers but are not integrated as middleware.

## Sub-Agent Tasks

### Task 3.1: Implement rate limiter middleware
**Status**: ⚠️ Partial (core struct implemented, not a Tower layer)
**Priority**: 🔴 Critical — essential for production use
**Estimated effort**: Medium

**Objective**: Implement token bucket rate limiting per key and per provider using `governor`.

**Checklist**:
- [x] `crates/gateway-core/src/middleware/mod.rs` — `ProviderRateLimiter` struct exists
  - [x] Uses `governor` crate for token bucket rate limiting
  - [x] Supports per-provider rate limiting (requests per minute)
  - [x] `from_config()` creates rate limiters from provider configurations
  - [x] `check_rate_limit(provider)` returns `Ok(())` or `Err(GatewayError::RateLimitExceeded)`
  - [x] Handles missing rate limit config gracefully (no limiter = no limit)
- [x] Unit test: `test_rate_limiter_creation` — verifies limiter creation and check
- [ ] Implement as Tower middleware layer (not yet done)
  - [ ] Create `RateLimitMiddleware` struct implementing `tower::Service`
  - [ ] Create `RateLimitLayer` struct implementing `tower::Layer`
  - [ ] Integrate into the axum router pipeline
  - [ ] Rate limit should return `429 Too Many Requests` with `Retry-After` header
- [ ] Support per-key rate limiting (API key based) — currently only per-provider
- [ ] Add more comprehensive tests
  - [ ] Test that requests are rejected when limit exceeded
  - [ ] Test that rate limit resets after window
  - [ ] Test per-key rate limiting

**Notes**:
- Uses `governor` crate for token bucket rate limiting
- Rate limit config comes from `GatewayConfig::providers[name].rate_limit`
- The `RateLimitConfig` has `requests_per_minute` and `tokens_per_minute`

---

### Task 3.2: Implement cost metering
**Status**: ⚠️ Partial (core struct implemented, not a Tower layer)
**Priority**: 🟡 Medium — useful for billing and monitoring
**Estimated effort**: Medium

**Objective**: Track token usage and calculate costs per request, key, and provider.

**Checklist**:
- [x] `crates/gateway-core/src/middleware/mod.rs` — `CostMeter` struct exists
  - [x] Uses `AtomicU64` for lock-free request/token counting
  - [x] Uses `RwLock<HashMap>` for per-provider request tracking
  - [x] `record_request(provider, usage)` records token usage
  - [x] `stats()` returns aggregated `MeteringStats`
  - [x] `is_enabled()` checks if metering is active
  - [x] Respects `MeteringConfig::enabled` flag
- [x] Unit test: `test_cost_meter` — verifies recording and stats
- [ ] Implement as Tower middleware layer (not yet done)
  - [ ] Create `CostMeterMiddleware` struct implementing `tower::Service`
  - [ ] Create `CostMeterLayer` struct implementing `tower::Layer`
  - [ ] Track token usage from response body
- [ ] Define cost models for each provider
  - [ ] Create `pricing.rs` or similar module
  - [ ] Define pricing per token for each model
  - [ ] Calculate cost from token usage
- [ ] Add more comprehensive tests
  - [ ] Test that cost is calculated correctly
  - [ ] Test that stats are aggregated correctly over multiple requests

**Notes**:
- The `MeteringConfig` has `enabled` flag and `metrics` list
- Metrics include: `RequestCount`, `TokenUsage`, `CostEstimate`

---

### Task 3.3: Implement response caching
**Status**: ⚠️ Partial (core struct implemented, not a Tower layer)
**Priority**: 🟡 Medium — improves performance for repeated requests
**Estimated effort**: Medium

**Objective**: Implement response caching with configurable TTL using `moka`.

**Checklist**:
- [x] `crates/gateway-core/src/middleware/mod.rs` — `ChatCache` struct exists
  - [x] Uses `moka::future::Cache` for high-performance concurrent caching
  - [x] `get(request)` — retrieves cached response if available
  - [x] `put(request, response)` — stores response in cache
  - [x] `clear()` — invalidates all cached entries
  - [x] `stats()` — returns `CacheStats` (enabled, entry_count, weighted_size)
  - [x] `cache_key(request)` — generates hash-based cache key from model, messages, temperature, max_tokens
  - [x] Respects `CacheConfig::enabled` flag
  - [x] TTL and max_size configured from `CacheConfig`
- [x] Unit test: `test_cache_operations` — verifies cache miss, put, hit
- [ ] Implement as Tower middleware layer (not yet done)
  - [ ] Create `CacheMiddleware` struct implementing `tower::Service`
  - [ ] Create `CacheLayer` struct implementing `tower::Layer`
  - [ ] Check cache before making request
  - [ ] Store response in cache after receiving it
  - [ ] Only cache non-streaming responses
- [ ] Add more comprehensive tests
  - [ ] Test TTL expiration
  - [ ] Test cache key uniqueness
  - [ ] Test cache clearing

**Notes**:
- Uses `moka` crate for high-performance concurrent caching
- Cache config comes from `GatewayConfig::cache`
- `CacheConfig` has `enabled`, `ttl_seconds`, `max_size`

---

### Task 3.4: Implement authentication middleware
**Status**: ⚠️ Partial (core struct implemented, not a Tower layer)
**Priority**: 🟡 Medium — essential for production use
**Estimated effort**: Small

**Objective**: Implement API key validation middleware.

**Checklist**:
- [x] `crates/gateway-core/src/middleware/mod.rs` — `AuthMiddleware` struct exists
  - [x] `new(api_keys, required)` — creates auth middleware with key list and required flag
  - [x] `validate_api_key(key)` — validates an API key against the list
  - [x] Returns `Err(GatewayError::Authentication)` for invalid/missing keys
  - [x] Supports optional auth (when `required: false`, missing key is allowed)
  - [x] When `api_keys` is empty, any key is accepted (open mode)
- [x] Unit test: `test_auth_middleware` — verifies valid key, invalid key, missing key required, missing key optional
- [ ] Implement as Tower middleware layer (not yet done)
  - [ ] Create `AuthLayer` struct implementing `tower::Layer`
  - [ ] Create `AuthMiddleware` struct implementing `tower::Service`
  - [ ] Check `Authorization: Bearer <key>` header
  - [ ] Return `401 Unauthorized` if key is invalid
- [ ] Add integration with gateway-api handlers
  - [ ] Extract API key from request headers in handlers

**Notes**:
- For now, API keys can be hardcoded or from config
- In production, use a database or external service for API key validation

---

### Task 3.5: Implement OpenTelemetry integration
**Status**: ❌ Not Started
**Priority**: 🟢 Low — observability is important but not critical for Phase 3
**Estimated effort**: Large

**Objective**: Integrate OpenTelemetry for tracing, metrics, and logging.

**Checklist**:
- [ ] Create `crates/gateway-core/src/middleware/telemetry.rs`
  - [ ] Implement `TelemetryMiddleware` struct
  - [ ] Implement `init_telemetry(config: &TelemetryConfig) -> Result<()>` function
  - [ ] Implement `get_tracer() -> Tracer` function
  - [ ] Implement `get_meter() -> Meter` function
- [ ] Implement as Tower middleware layer
  - [ ] Create `TelemetryLayer` struct
  - [ ] Implement `tower::Layer` for `TelemetryLayer`
  - [ ] Implement `tower::Service` for `TelemetryMiddleware`
  - [ ] Trace incoming requests
  - [ ] Trace outgoing provider requests
  - [ ] Record latency, token usage, error counts
- [ ] Add metrics
  - [ ] Request count per provider
  - [ ] Token usage per provider
  - [ ] Latency per provider
  - [ ] Error rate per provider
- [ ] Add tests for telemetry
  - [ ] Test that traces are generated
  - [ ] Test that metrics are recorded

**Notes**:
- Use `opentelemetry` crate for OpenTelemetry integration
- Use `tracing` for structured logging
- Telemetry config comes from `GatewayConfig::telemetry`

---

### Task 3.6: Wire up middleware in gateway-api
**Status**: ❌ Not Started
**Priority**: 🔴 Critical — middleware is useless until wired up
**Estimated effort**: Medium

**Objective**: Integrate all middleware layers into the gateway-api server pipeline.

**Checklist**:
- [ ] Update `crates/gateway-api/src/lib.rs`
  - [ ] Add `RateLimitLayer` to the router
  - [ ] Add `AuthLayer` to the router
  - [ ] Add `CacheLayer` to the router
  - [ ] Add `CostMeterLayer` to the router
  - [ ] Add `TelemetryLayer` to the router
  - [ ] Create `AppState` with all middleware instances
- [ ] Update `crates/gateway-api/src/main.rs`
  - [ ] Initialize telemetry on startup
  - [ ] Initialize rate limiter from config
  - [ ] Initialize cache from config
  - [ ] Initialize cost meter from config
- [ ] Add tests for middleware integration
  - [ ] Test that middleware is applied correctly
  - [ ] Test that middleware works with handlers

**Notes**:
- All core middleware structs exist but need to be wrapped as Tower layers
- The `main.rs` already sets up the axum server — middleware layers need to be added to the router chain

---

## Dependencies & Ordering

```
Task 3.1 (Rate limiter) ──── ⚠️ Core struct done, Tower layer needed
Task 3.2 (Cost metering) ── ⚠️ Core struct done, Tower layer needed
Task 3.3 (Cache) ─────────── ⚠️ Core struct done, Tower layer needed
Task 3.4 (Auth) ──────────── ⚠️ Core struct done, Tower layer needed
Task 3.5 (Telemetry) ────── ❌ Not started
Task 3.6 (Wire up) ──────── ❌ Not started — depends on all other tasks
```

**Key insight**: Tasks 3.1-3.4 have their core logic implemented but need to be wrapped as Tower `Service`/`Layer` types. Task 3.5 (Telemetry) has no implementation yet. Task 3.6 depends on all others being complete.

## Success Criteria

- [ ] Rate limiter prevents excessive requests (**struct exists, not wired as middleware**)
- [ ] Cost metering tracks token usage correctly (**struct exists, not wired as middleware**)
- [ ] Cache stores and retrieves responses correctly (**struct exists, not wired as middleware**)
- [ ] Authentication middleware rejects invalid API keys (**struct exists, not wired as middleware**)
- [ ] OpenTelemetry traces and metrics are generated (**not started**)
- [ ] All middleware layers are properly wired up in the server pipeline (**not started**)
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
