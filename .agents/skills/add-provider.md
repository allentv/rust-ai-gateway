# Skill: Add New Provider

Guide for adding a new AI provider integration to the gateway.

## Overview

Each provider is implemented as a struct that implements the `Provider` trait defined in `crates/gateway-core/src/providers/traits.rs`. Providers are self-contained and isolated from each other.

## Steps

### 1. Create the Provider Implementation

Create a new file at `crates/gateway-core/src/providers/<provider_name>.rs`:

```rust
use async_trait::async_trait;
use crate::error::GatewayError;
use crate::providers::traits::Provider;
use crate::types::{ChatRequest, ChatResponse, ChatChunk};

pub struct <ProviderName>Provider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl <ProviderName>Provider {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Provider for <ProviderName>Provider {
    async fn complete_chat(&self, request: ChatRequest) -> Result<ChatResponse, GatewayError> {
        // Transform ChatRequest to provider-specific format
        // Make HTTP request to provider API
        // Transform response back to ChatResponse
        todo!()
    }

    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<ChatChunk, GatewayError>>, GatewayError> {
        // Transform ChatRequest to provider-specific format
        // Make streaming HTTP request
        // Parse SSE events into ChatChunk
        todo!()
    }

    fn name(&self) -> &str {
        "<provider_name>"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}
```

### 2. Register the Provider

Add the module to `crates/gateway-core/src/providers/mod.rs`:

```rust
pub mod <provider_name>;
```

### 3. Add Configuration Schema

Add provider-specific configuration to `crates/gateway-config/src/schema.rs` in the `ProviderConfig` enum or struct.

### 4. Add Validation

Add validation rules for the new provider in `crates/gateway-config/src/validation.rs`.

### 5. Write Tests

Add unit tests in the provider file and integration tests as needed. Run with:

```bash
cargo test -p gateway-core
```

## Key Files

- `crates/gateway-core/src/providers/traits.rs` — Provider trait definition
- `crates/gateway-core/src/providers/openai.rs` — Reference implementation (OpenAI)
- `crates/gateway-core/src/providers/anthropic.rs` — Reference implementation (Anthropic)
- `crates/gateway-core/src/types.rs` — Shared domain types
- `crates/gateway-core/src/error.rs` — Error types
- `crates/gateway-config/src/schema.rs` — Configuration schema
