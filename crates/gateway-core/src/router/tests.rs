use super::*;
use gateway_config::schema::{RoutingConfig, ServerConfig};

fn test_config() -> GatewayConfig {
    let mut providers = HashMap::new();
    providers.insert(
        "openai".to_string(),
        ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
            rate_limit: None,
        },
    );
    providers.insert(
        "anthropic".to_string(),
        ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            models: vec!["claude-3-opus-20240229".to_string()],
            rate_limit: None,
        },
    );

    GatewayConfig {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
        },
        providers,
        routing: RoutingConfig {
            default_provider: "openai".to_string(),
            fallback_providers: vec!["anthropic".to_string()],
        },
        cache: Default::default(),
        telemetry: Default::default(),
        metering: Default::default(),
    }
}

#[test]
fn test_router_creation() {
    let config = test_config();
    let router = Router::new(&config);
    assert!(router.is_ok());
    let router = router.unwrap();
    assert_eq!(router.default_provider_name(), "openai");
}

#[test]
fn test_available_providers() {
    let config = test_config();
    let router = Router::new(&config).unwrap();
    let providers = router.available_providers();
    assert!(providers.contains(&"openai"));
    assert!(providers.contains(&"anthropic"));
}

#[test]
fn test_available_models() {
    let config = test_config();
    let router = Router::new(&config).unwrap();
    let models = router.available_models();
    assert!(models.contains(&"gpt-4".to_string()));
    assert!(models.contains(&"gpt-3.5-turbo".to_string()));
    assert!(models.contains(&"claude-3-opus-20240229".to_string()));
}

#[test]
fn test_is_model_supported() {
    let config = test_config();
    let router = Router::new(&config).unwrap();
    assert!(router.is_model_supported("gpt-4"));
    assert!(router.is_model_supported("claude-3-opus-20240229"));
    assert!(!router.is_model_supported("nonexistent-model"));
}

#[test]
fn test_invalid_default_provider() {
    let mut config = test_config();
    config.routing.default_provider = "nonexistent".to_string();
    let router = Router::new(&config);
    assert!(router.is_err());
}

#[test]
fn test_get_provider() {
    let config = test_config();
    let router = Router::new(&config).unwrap();
    assert!(router.get_provider("openai").is_some());
    assert!(router.get_provider("nonexistent").is_none());
}

#[tokio::test]
async fn test_resolve_provider_known_model_default() {
    let config = test_config();
    let router = Router::new(&config).unwrap();
    let request = ChatRequest {
        messages: vec![crate::types::Message {
            role: crate::types::Role::User,
            content: "Hello".to_string(),
            name: None,
        }],
        model: "gpt-4".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        provider: None,
    };
    // No provider specified, default is "openai" which supports gpt-4
    let result = router.route(request).await;
    // The request will fail because we're using a fake API key,
    // but the router should attempt to use the openai provider (not return ProviderNotFound)
    assert!(!matches!(result, Err(GatewayError::ProviderNotFound(_))));
}

#[tokio::test]
async fn test_resolve_provider_explicit_provider() {
    let config = test_config();
    let router = Router::new(&config).unwrap();
    let request = ChatRequest {
        messages: vec![crate::types::Message {
            role: crate::types::Role::User,
            content: "Hello".to_string(),
            name: None,
        }],
        model: "claude-3-opus-20240229".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        provider: Some("anthropic".to_string()),
    };
    // Explicit provider "anthropic" with a model it supports
    let result = router.route(request).await;
    // Should route to anthropic (will fail due to fake API key, but not ProviderNotFound)
    assert!(!matches!(result, Err(GatewayError::ProviderNotFound(_))));
}

#[tokio::test]
async fn test_resolve_provider_fallback_for_unsupported_model() {
    let config = test_config();
    let router = Router::new(&config).unwrap();
    let request = ChatRequest {
        messages: vec![crate::types::Message {
            role: crate::types::Role::User,
            content: "Hello".to_string(),
            name: None,
        }],
        model: "claude-3-opus-20240229".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        provider: None,
    };
    // Default provider is "openai" which doesn't support claude-3-opus
    // Should fall back to "anthropic"
    let result = router.route(request).await;
    assert!(!matches!(result, Err(GatewayError::ProviderNotFound(_))));
}

#[tokio::test]
async fn test_resolve_provider_unknown_model_no_fallback() {
    let config = test_config();
    let router = Router::new(&config).unwrap();
    let request = ChatRequest {
        messages: vec![crate::types::Message {
            role: crate::types::Role::User,
            content: "Hello".to_string(),
            name: None,
        }],
        model: "nonexistent-model".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        provider: None,
    };
    // No provider supports "nonexistent-model"
    let result = router.route(request).await;
    match result {
        Err(GatewayError::ModelNotSupported { model, .. }) => {
            assert_eq!(model, "nonexistent-model");
        }
        other => panic!("Expected ModelNotSupported error, got: {:?}", other),
    }
}
