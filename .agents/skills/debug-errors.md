nt # Skill: Debug Errors

Guide for debugging errors in the gateway.

## Error Types

The gateway uses `GatewayError` (defined in `crates/gateway-core/src/error.rs`) as its unified error type. Each variant maps to a specific HTTP status code.

## Error Variants

| Variant | HTTP Status | Typical Cause |
|---------|-------------|---------------|
| `Provider(String, Option<Box<dyn Error>>)` | 502 | Upstream provider returned an error |
| `ProviderNotFound(String)` | 404 | Requested provider not in config |
| `ModelNotSupported(String, String)` | 400 | Model not supported by provider |
| `Timeout(String)` | 504 | Request to provider timed out |
| `RateLimitExceeded(String)` | 429 | Rate limit hit |
| `Authentication(String)` | 401 | Invalid/missing API key |
| `Configuration(String)` | 500 | Invalid gateway configuration |
| `Serialization(String)` | 500 | JSON/deserialization error |
| `Network(reqwest::Error)` | 502 | Network-level error connecting to provider |
| `StreamClosed` | 500 | SSE stream was unexpectedly closed |
| `Internal(String)` | 500 | Unexpected internal error |

## Debugging Steps

1. **Check the error variant** — The HTTP status code narrows down the cause
2. **Check provider logs** — Provider errors often include upstream error details
3. **Check configuration** — Configuration errors are caught at startup but can occur at runtime
4. **Check network connectivity** — Network errors indicate issues reaching the provider
5. **Check rate limits** — Rate limit errors indicate the configured limits are too low

## Key Files

- `crates/gateway-core/src/error.rs` — Error type definitions and HTTP status mapping
- `crates/gateway-core/src/providers/` — Provider-specific error handling
- `crates/gateway-config/src/validation.rs` — Configuration validation errors

## Common Fixes

- **Provider errors**: Check the upstream provider's status page, verify API key, check model availability
- **Authentication errors**: Verify `api_key` is set correctly in config and env vars are resolved
- **Rate limit errors**: Increase rate limits in config or implement request queuing
- **Timeout errors**: Check network connectivity, increase timeout values, check provider status
- **Configuration errors**: Run config validation: `cargo run --bin gateway-cli -- validate <config-file>`
