# Rust AI Gateway

A high-throughput, low-latency proxy for AI provider APIs. Route requests across OpenAI, Anthropic, and other providers with built-in rate limiting, cost metering, and response caching.

## Overview

The AI Gateway sits between your application and AI providers, acting as a single unified endpoint. It handles provider abstraction, request routing, token/cost tracking, and optional response caching — so your application code only talks to one API.

**Key capabilities:**

- **Provider abstraction** — unified interface for OpenAI, Anthropic, and custom providers
- **Streaming support** — transparent SSE streaming passthrough with no buffering overhead
- **Rate limiting** — per-key and per-provider token bucket rate limiting via `governor`
- **Cost metering** — track token usage and estimated cost per request, key, and provider
- **Response caching** — deduplicate identical requests with configurable TTL and cache keys
- **YAML/TOML configuration** — declarative config for providers, routes, and limits
- **Observability** — OpenTelemetry integration for tracing, metrics, and logging

## Architecture

The project follows a Cargo workspace layout with four crates, each with a clear responsibility:

```
rust-ai-gateway/
├── Cargo.toml                        # Workspace root
├── crates/
│   ├── gateway-core/                 # Core library: providers, types, errors, routing
│   ├── gateway-api/                  # HTTP API layer (axum handlers, server entry point)
│   ├── gateway-cli/                  # CLI tool for config and management
│   └── gateway-config/               # Configuration schemas and validation
├── config/                           # Default configuration files
├── docs/                             # Design documents
│   ├── initial-idea.md
│   └── implementation-plan.md
└── .agents/                          # AI agent commands and skills
```

### Crate Overview

| Crate | Description |
|-------|-------------|
| `gateway-core` | Core domain types, provider trait definitions, error types, and provider implementations (OpenAI, Anthropic) |
| `gateway-api` | Axum-based HTTP server with endpoints for chat completions and health checks |
| `gateway-cli` | Command-line interface for configuration validation and management |
| `gateway-config` | YAML/TOML configuration schema definitions and validation logic |

### Technology Stack

| Component | Technology |
|-----------|------------|
| HTTP Server | [axum](https://github.com/tokio-rs/axum) |
| Async Runtime | [tokio](https://github.com/tokio-rs/tokio) |
| HTTP Client | [reqwest](https://github.com/seanmonstar/reqwest) (streaming + rustls-tls) |
| Serialization | [serde](https://github.com/serde-rs/serde) + serde_yaml / serde_toml / serde_json |
| Rate Limiting | [governor](https://github.com/antifuchs/governor) |
| Caching | [moka](https://github.com/gregory-m/moka) |
| Middleware | [tower](https://github.com/tower-rs/tower) + [tower-http](https://github.com/tower-rs/tower-http) |
| CLI | [clap](https://github.com/clap-rs/clap) |
| Error Handling | [thiserror](https://github.com/dtolnay/thiserror) + [anyhow](https://github.com/dtolnay/anyhow) |
| Logging | [tracing](https://github.com/tokio-rs/tracing) + tracing-subscriber |

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.96+ (edition 2021)
- [Cargo](https://doc.rust-lang.org/cargo/) (included with Rust)

### Build

```bash
# Build the entire workspace
cargo build

# Build in release mode
cargo build --release
```

### Run

```bash
# Run the API server
cargo run --bin gateway-api

# Run the CLI
cargo run --bin gateway-cli -- --help
```

### Test

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p gateway-config
```

## Configuration

The gateway is configured via YAML or TOML files. Below is an example configuration:

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

  anthropic:
    api_key: "${ANTHROPIC_API_KEY}"
    base_url: "https://api.anthropic.com/v1"
    models:
      - claude-3-opus
      - claude-3-sonnet
    rate_limit:
      requests_per_minute: 30

routing:
  default_provider: "openai"
  fallback_providers:
    - anthropic

cache:
  enabled: true
  ttl_seconds: 3600
  max_size: 10000

telemetry:
  enabled: true
  endpoint: "http://localhost:4317"
  service_name: "ai-gateway"
```

Environment variables can be referenced using the `${VAR_NAME}` syntax in configuration files.

## API

The gateway exposes an OpenAI-compatible chat completions endpoint:

### Chat Completions

```
POST /v1/chat/completions
```

**Request body:**

```json
{
  "model": "gpt-4",
  "messages": [
    { "role": "system", "content": "You are a helpful assistant." },
    { "role": "user", "content": "Hello, world!" }
  ],
  "max_tokens": 1024,
  "temperature": 0.7,
  "stream": false
}
```

**Response:**

```json
{
  "id": "req_abc123",
  "content": "Hello! How can I help you today?",
  "usage": {
    "prompt_tokens": 25,
    "completion_tokens": 12,
    "total_tokens": 37
  },
  "model": "gpt-4",
  "provider": "openai",
  "created_at": "2026-06-28T08:51:00Z"
}
```

### Streaming (SSE)

Set `"stream": true` in the request body to receive Server-Sent Events:

```
POST /v1/chat/completions
Content-Type: application/json

{"model": "gpt-4", "messages": [{"role": "user", "content": "Hi"}], "stream": true}
```

### Health Check

```
GET /health
```

## Error Handling

The gateway returns structured error responses with appropriate HTTP status codes:

| Error | HTTP Status | Description |
|-------|-------------|-------------|
| `ProviderNotFound` | 404 | Requested provider is not configured |
| `ModelNotSupported` | 400 | Model not supported by the provider |
| `Authentication` | 401 | Invalid or missing API key |
| `RateLimitExceeded` | 429 | Rate limit exceeded |
| `Timeout` | 504 | Request to upstream provider timed out |
| `Provider` | 502 | Error from upstream provider |
| `Configuration` | 500 | Gateway configuration error |

## Development

### Project Structure

```
crates/
├── gateway-core/src/
│   ├── lib.rs                  # Core library entry point
│   ├── config.rs               # Config loading wrapper
│   ├── types/mod.rs            # ChatRequest, ChatResponse, Message, Role, TokenUsage, etc.
│   ├── types/tests.rs          # Type serialization and unit tests
│   ├── error/mod.rs            # GatewayError enum with HTTP status mapping
│   ├── error/tests.rs          # Error constructor and HTTP mapping tests
│   ├── providers/
│   │   ├── traits.rs           # Provider trait definition
│   │   ├── openai.rs           # OpenAI API implementation
│   │   ├── anthropic.rs        # Anthropic API implementation
│   │   ├── google.rs           # Google (stub) implementation
│   │   ├── custom.rs           # Custom provider (stub) implementation
│   │   └── mod.rs              # Provider module exports
│   ├── router/
│   │   ├── mod.rs              # Router: provider selection and request routing
│   │   └── tests.rs            # Router unit tests
│   └── middleware/
│       ├── mod.rs              # Rate limiter, cache, auth, cost meter
│       └── tests.rs            # Middleware unit tests
├── gateway-config/src/
│   ├── lib.rs                  # Module declarations
│   ├── schema.rs               # Configuration schema structs
│   ├── validation/mod.rs       # Config validation logic
│   └── validation/tests.rs     # Config validation tests (25 tests)
├── gateway-api/
│   └── src/
│       ├── main.rs             # Axum server entry point
│       ├── handlers/           # Chat completions, health check endpoints
│       └── middleware/         # HTTP middleware (placeholder)
└── gateway-cli/
    └── src/
        ├── main.rs             # CLI entry point (clap)
        └── commands/           # Config, status, cache subcommands
config/
├── default.yaml                # Default configuration
└── example.yaml                # Detailed example with comments
```

### Useful Commands

```bash
# Build the workspace
cargo build

# Build in release mode
cargo build --release

# Run all tests
cargo test --workspace

# Run clippy lints
cargo clippy --workspace

# Format code
cargo fmt

# Run the server
cargo run --bin gateway-api

# Run the CLI
cargo run --bin gateway-cli

# Check compilation without building
cargo check --workspace
```

## Implementation Status

This project is under active development. Here is the current implementation status:

### Phase 1: Foundation ✅
- [x] Cargo workspace setup
- [x] Core types (`ChatRequest`, `ChatResponse`, `Message`, `Role`, `TokenUsage`, `ChatChunk`, `Delta`, `RequestId`)
- [x] Error types with HTTP status mapping
- [x] Provider trait definition with async streaming support
- [x] Configuration schema and validation (YAML/TOML/JSON)
- [x] OpenAI provider implementation (with SSE streaming)
- [x] Anthropic provider implementation (with SSE streaming)
- [x] Google and Custom provider stubs
- [x] Router with provider selection and fallback
- [x] Middleware structs (rate limiter, cache, auth, cost meter)
- [x] HTTP API server with axum (health check, chat endpoint)
- [x] CLI with config validation and status display
- [x] Default and example configuration files
- [x] 64 unit tests, clippy clean

### Phase 2: Core Functionality 🔧
- [x] HTTP API endpoints (axum) — basic endpoints exist
- [x] Request routing logic — Router implemented
- [ ] SSE streaming passthrough (providers support it, API handler not wired)
- [ ] Wire Router into chat handler (currently returns placeholder)

### Phase 3: Middleware & Features 🔧
- [x] Rate limiting (governor) — struct exists, not wired as Tower layer
- [x] Cost metering — struct exists, not wired
- [x] Response caching (moka) — struct exists, not wired
- [ ] OpenTelemetry integration
- [x] Authentication middleware — struct exists, not wired
- [ ] Configuration hot-reload

### Phase 4: Production Readiness 🔧
- [x] Graceful shutdown
- [x] Health checks (basic `/health` endpoint)
- [ ] Readiness probe (`/ready`)
- [ ] Docker and Kubernetes deployment
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Comprehensive documentation
- [ ] Integration tests

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests (`cargo test --workspace`)
5. Run lints (`cargo clippy --workspace && cargo fmt`)
6. Commit your changes
7. Push to the branch (`git push origin feature/my-feature`)
8. Open a Pull Request

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
