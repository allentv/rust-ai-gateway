# Rust AI Gateway

A high-throughput, low-latency proxy for AI provider APIs. Route requests across OpenAI, Anthropic, Google, and other providers with built-in rate limiting, cost metering, and response caching.

## Overview

The AI Gateway sits between your application and AI providers, acting as a single unified endpoint. It handles provider abstraction, request routing, token/cost tracking, and optional response caching — so your application code only talks to one API.

**Key capabilities:**

- **Provider abstraction** — unified interface for OpenAI, Anthropic, Google, and custom providers
- **Streaming support** — transparent SSE streaming passthrough with no buffering overhead
- **Rate limiting** — per-key and per-provider token bucket rate limiting
- **Cost metering** — track token usage and estimated cost per request, key, and provider
- **Response caching** — deduplicate identical requests with configurable TTL and cache keys
- **YAML/TOML configuration** — declarative config for providers, routes, and limits
- **Dashboards** - Use OpenTelemetry to get the usage data and surface them in grafana dashboards
