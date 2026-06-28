# Test Project

Run the project test suite and report results.

## Steps

1. Run `mise run test` to execute all tests
2. If tests fail, analyze the failures and fix the underlying code
3. For a specific crate, use `mise run test-crate <crate-name>`
4. For verbose output, use `mise run test-output`

## Commands

```bash
# Run all tests
mise run test

# Run all tests with output
mise run test-output

# Run tests for a specific crate
mise run test-crate gateway-core
mise run test-crate gateway-config
mise run test-crate gateway-api
mise run test-crate gateway-cli
