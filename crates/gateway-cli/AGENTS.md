# gateway-cli — Crate Context

This is the CLI tool crate for the Rust AI Gateway. It provides command-line interface for configuration validation, status checking, and cache management.

## Crate Structure

```
gateway-cli/
├── Cargo.toml             # ✅ Exists — binary crate with [[bin]] name = "gateway-cli"
└── src/
    ├── main.rs            # ❌ MISSING — needs to be created (binary entry point)
    └── commands/          # ❌ MISSING — CLI subcommands
        ├── mod.rs
        ├── config.rs      # Config validation subcommand
        ├── status.rs      # Status subcommand
        └── cache.rs       # Cache management subcommand
```

## Current Status

**This crate has NO source files** — only a `Cargo.toml` exists. Everything in `src/` needs to be created.

## Dependencies

- `tokio` — Async runtime
- `clap` (4, with `derive` feature) — CLI argument parsing
- `serde` / `serde_json` / `serde_yaml` — Serialization
- `anyhow` — Application-level error handling
- `tracing` / `tracing-subscriber` — Logging
- `gateway-config` — Configuration schemas and validation

## Binary Configuration

The crate is configured as a binary crate with `[[bin]] name = "gateway-cli"` and `path = "src/main.rs"`.

## Planned Functionality

### CLI Subcommands
1. **`config`** — Validate configuration file
   - `config validate <path>` — Validate a config file (YAML/TOML/JSON)
   - `config show <path>` — Show parsed configuration
2. **`status`** — Show gateway status
   - `status` — Display configured providers, models, and server info
3. **`cache`** — Cache management (planned)
   - `cache clear` — Clear the response cache
   - `cache stats` — Show cache statistics

## Implementation Pattern

```rust
// src/main.rs pattern (planned)
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gateway-cli", about = "Rust AI Gateway CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Config { /* ... */ },
    Status { /* ... */ },
    Cache { /* ... */ },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Config { /* ... */ } => { /* ... */ }
        Commands::Status { /* ... */ } => { /* ... */ }
        Commands::Cache { /* ... */ } => { /* ... */ }
    }
    Ok(())
}
```

## What's Missing / Needs Implementation

1. **`main.rs`** — Binary entry point with clap CLI definition
2. **`commands/mod.rs`** — Module declarations and re-exports
3. **`commands/config.rs`** — Config validation subcommand
4. **`commands/status.rs`** — Status subcommand
5. **`commands/cache.rs`** — Cache management subcommand (planned)
6. **Tests** — Integration tests for CLI commands

## Implementation Plan Reference

See `docs/implementation-plan.md` for the full plan. This crate covers:
- **Phase 1**: Basic CLI with clap for configuration validation
- **Phase 4**: Production readiness tools

## Notes for AI Sessions

- This crate has no source files yet — `src/` directory doesn't exist
- The `Cargo.toml` already has all required dependencies
- Must create `lib.rs` or `main.rs` first
- Use `clap` derive macros for CLI argument parsing
- The `gateway-config` crate provides `load_config_with_env()` and `validate()` for config operations
- This is a binary crate (not a library), so it only needs `main.rs` and `commands/`
