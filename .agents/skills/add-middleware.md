# Skill: Add New Middleware

Guide for adding new middleware to the gateway.

## Overview

Middleware in the gateway is built on the `tower` ecosystem. Middleware is used for cross-cutting concerns like rate limiting, authentication, caching, and telemetry.

## Middleware Types

### 1. Tower Service Middleware

For request/response transformation, implement a Tower `Layer` and `Service`:

```rust
use std::task::{Context, Poll};
use tower::{Layer, Service};
use axum::http::Request;

#[derive(Clone)]
pub struct MyMiddlewareLayer;

impl<S> Layer<S> for MyMiddlewareLayer {
    type Service = MyMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MyMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct MyMiddleware<S> {
    inner: S,
}

impl<S, ReqBody> Service<Request<ReqBody>> for MyMiddleware<S>
where
    S: Service<Request<ReqBody>> + Send + Clone + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            // Pre-processing: runs before the inner service
            let response = inner.call(req).await?;
            // Post-processing: runs after the inner service
            Ok(response)
        })
    }
}
```

### 2. Axum Extractor Middleware

For route-specific middleware, use axum extractors and handlers.

## Implementation Steps

1. Create a new file in the appropriate location
2. Implement the middleware logic
3. Apply it in the axum router setup
4. Add tests

## Key Files

- `crates/gateway-api/src/` — API layer where middleware is applied
- `crates/gateway-core/src/` — Core middleware implementations
