# Check Project

Run a full project health check: compile, lint, format check, and test.

## Steps

1. Run `mise run lint` to check for lint issues
2. Run `mise run fmt-check` to verify formatting
3. Run `mise run test` to run all tests

## Command

```bash
mise run check
```
This runs lint, fmt-check, and test in sequence.
