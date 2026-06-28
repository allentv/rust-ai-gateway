use super::*;
use crate::types::{ChatRequest, Message, Role};

fn create_provider() -> OpenAiProvider {
    OpenAiProvider::new(
        "test-api-key".to_string(),
        "https://api.openai.com/v1".to_string(),
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
    assert_eq!(provider.name(), "openai");
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
    assert!(models.contains(&"gpt-4o"));
    assert!(models.contains(&"gpt-4o-mini"));
    assert!(models.contains(&"gpt-4-turbo"));
    assert!(models.contains(&"gpt-4"));
    assert!(models.contains(&"gpt-3.5-turbo"));
    assert_eq!(models.len(), 5);
}

#[test]
fn test_supports_model_known() {
    let provider = create_provider();
    assert!(provider.supports_model("gpt-4o"));
    assert!(provider.supports_model("gpt-4"));
    assert!(provider.supports_model("gpt-3.5-turbo"));
}

#[test]
fn test_supports_model_unknown() {
    let provider = create_provider();
    assert!(!provider.supports_model("claude-3-opus"));
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
            assert_eq!(p, "openai");
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

    assert!(result.is_err());
    match result.unwrap_err() {
        GatewayError::ModelNotSupported { model, provider: p } => {
            assert_eq!(model, "unsupported-model");
            assert_eq!(p, "openai");
        }
        other => panic!("Expected ModelNotSupported, got: {:?}", other),
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

    let converted = OpenAiProvider::convert_messages(&messages);
    assert_eq!(converted.len(), 4);
    assert_eq!(converted[0].role, "system");
    assert_eq!(converted[1].role, "user");
    assert_eq!(converted[1].name, Some("user1".to_string()));
    assert_eq!(converted[2].role, "assistant");
    assert_eq!(converted[3].role, "tool");
}

#[test]
fn test_map_role() {
    assert_eq!(
        OpenAiProvider::map_role(Some("assistant".to_string())),
        Some("assistant".to_string())
    );
    assert_eq!(OpenAiProvider::map_role(None), None);
    assert_eq!(OpenAiProvider::map_role(Some("".to_string())), None);
}

#[test]
fn test_new_provider_stores_config() {
    let provider = OpenAiProvider::new(
        "my-key".to_string(),
        "https://custom.api.com/v1".to_string(),
    );
    assert_eq!(provider.name(), "openai");
    assert_eq!(provider.api_key, "my-key");
    assert_eq!(provider.base_url, "https://custom.api.com/v1");
}
