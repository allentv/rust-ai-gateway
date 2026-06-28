use std::time::Duration;

use super::*;
use crate::types::{ChatRequest, Message, Role};

fn create_provider() -> AnthropicProvider {
    AnthropicProvider::new(
        "test-api-key".to_string(),
        "https://api.anthropic.com/v1".to_string(),
    )
}

fn create_request(model: &str) -> ChatRequest {
    ChatRequest {
        messages: vec![Message {
            role: Role::User,
            content: "Hello".to_string(),
            name: None,
        }],
        model: model.to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        provider: None,
    }
}

#[test]
fn test_provider_name() {
    let provider = create_provider();
    assert_eq!(provider.name(), "anthropic");
}

#[test]
fn test_supports_streaming() {
    let provider = create_provider();
    assert!(provider.supports_streaming());
}

#[test]
fn test_supported_models() {
    let provider = create_provider();
    let models = provider.supported_models();
    assert!(models.contains(&"claude-sonnet-4-20250514"));
    assert!(models.contains(&"claude-3-5-sonnet-20241022"));
    assert!(models.contains(&"claude-3-5-haiku-20241022"));
    assert!(models.contains(&"claude-3-opus-20240229"));
    assert!(models.contains(&"claude-3-sonnet-20240229"));
    assert!(models.contains(&"claude-3-haiku-20240307"));
    assert_eq!(models.len(), 6);
}

#[test]
fn test_supports_model_known() {
    let provider = create_provider();
    assert!(provider.supports_model("claude-3-5-sonnet-20241022"));
    assert!(provider.supports_model("claude-3-opus-20240229"));
}

#[test]
fn test_supports_model_unknown() {
    let provider = create_provider();
    assert!(!provider.supports_model("gpt-4o"));
    assert!(!provider.supports_model("nonexistent-model"));
}

#[test]
fn test_complete_chat_unsupported_model() {
    let provider = create_provider();
    let request = create_request("unsupported-model");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(provider.complete_chat(request));

    assert!(result.is_err());
    match result.unwrap_err() {
        GatewayError::ModelNotSupported { model, provider: p } => {
            assert_eq!(model, "unsupported-model");
            assert_eq!(p, "anthropic");
        }
        other => panic!("Expected ModelNotSupported, got: {:?}", other),
    }
}

#[test]
fn test_stream_chat_unsupported_model() {
    let provider = create_provider();
    let request = create_request("unsupported-model");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(provider.stream_chat(request));

    match result {
        Err(GatewayError::ModelNotSupported { model, provider: p }) => {
            assert_eq!(model, "unsupported-model");
            assert_eq!(p, "anthropic");
        }
        other => panic!("Expected ModelNotSupported, got stream result"),
    }
}

#[test]
fn test_convert_messages() {
    let messages = vec![
        Message {
            role: Role::System,
            content: "You are a helper".to_string(),
            name: None,
        },
        Message {
            role: Role::User,
            content: "Hi".to_string(),
            name: Some("user1".to_string()),
        },
        Message {
            role: Role::Assistant,
            content: "Hello!".to_string(),
            name: None,
        },
        Message {
            role: Role::Tool,
            content: "tool result".to_string(),
            name: None,
        },
    ];

    let (system, converted) = AnthropicProvider::convert_messages(&messages);
    assert_eq!(system, Some("You are a helper".to_string()));
    assert_eq!(converted.len(), 3);
    assert_eq!(converted[0].role, "user");
    assert_eq!(converted[0].content, "Hi");
    assert_eq!(converted[1].role, "assistant");
    assert_eq!(converted[1].content, "Hello!");
    assert_eq!(converted[2].role, "user");
    assert_eq!(converted[2].content, "tool result");
}

#[test]
fn test_new_provider_stores_config() {
    let provider = AnthropicProvider::new(
        "my-key".to_string(),
        "https://custom.api.com/v1".to_string(),
    );
    assert_eq!(provider.name(), "anthropic");
    assert_eq!(provider.api_key, "my-key");
    assert_eq!(provider.base_url, "https://custom.api.com/v1");
}

#[test]
fn test_with_timeout_provider() {
    let provider = AnthropicProvider::with_timeout(
        "my-key".to_string(),
        "https://custom.api.com/v1".to_string(),
        Duration::from_secs(60),
    );
    assert_eq!(provider.name(), "anthropic");
    assert_eq!(provider.api_key, "my-key");
    assert_eq!(provider.base_url, "https://custom.api.com/v1");
}
