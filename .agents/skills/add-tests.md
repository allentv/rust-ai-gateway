# Skill: Add Tests

Guide for writing tests for the Rust AI Gateway.

## Test Types

### 1. Unit Tests

Unit tests live inside source files in a `#[cfg(test)] mod tests` block:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = "test";

        // Act
        let result = some_function(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### 2. Async Unit Tests

For async code (common in provider implementations):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_functionality() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### 3. Test Fixtures

For tests that need mock data, define fixtures within the test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_chat_request() -> ChatRequest {
        ChatRequest {
            messages: vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
                name: None,
            }],
            model: "gpt-4".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: false,
            provider: None,
        }
    }
}
```

## Testing Guidelines

1. **Test one thing per test** — Each test should verify a single behavior
2. **Use descriptive names** — Test names should describe the scenario being tested
3. **Arrange-Act-Assert** — Follow the AAA pattern for test structure
4. **Test edge cases** — Empty inputs, boundary values, error conditions
5. **Use `#[tokio::test]`** for async tests
6. **Don't test implementation details** — Test public API behavior
7. **Keep tests independent** — Tests should not depend on each other or shared state

## Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p gateway-core

# Specific test by name
cargo test --workspace test_name

# With output
cargo test --workspace -- --nocapture

# Show test list
cargo test --workspace -- --list
```

## Key Files for Testing

- `crates/gateway-core/src/types.rs` — Test type construction and methods
- `crates/gateway-core/src/error.rs` — Test error mapping to HTTP status codes
- `crates/gateway-core/src/providers/` — Test provider implementations (may need HTTP mocking)
- `crates/gateway-config/src/validation.rs` — Test configuration validation rules
