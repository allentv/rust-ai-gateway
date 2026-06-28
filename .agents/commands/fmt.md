# Format Code

Format all Rust code in the workspace using `cargo fmt`.

## Steps

1. Run `mise run fmt` to format all code
2. Run `mise run fmt-check` to verify formatting is correct

## Commands

```bash
# Format all code
mise run fmt

# Check formatting without modifying files
mise run fmt-check

# Format only a specific crate
mise run fmt-crate gateway-core
