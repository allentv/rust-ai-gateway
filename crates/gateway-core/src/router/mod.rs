use std::collections::HashMap;

use tracing::{info, warn};

use crate::error::GatewayError;
use crate::providers::{
    AnthropicProvider, CustomProvider, GoogleProvider, OpenAiProvider, Provider,
};
use crate::types::{ChatRequest, ChatResponse};
use gateway_config::schema::{GatewayConfig, ProviderConfig};

/// Router that manages provider selection and request routing
pub struct Router {
    providers: HashMap<String, Box<dyn Provider>>,
    default_provider: String,
    fallback_providers: Vec<String>,
}

impl Router {
    /// Create a new Router from gateway configuration
    pub fn new(config: &GatewayConfig) -> Result<Self, GatewayError> {
        let mut providers: HashMap<String, Box<dyn Provider>> = HashMap::new();

        for (name, provider_config) in &config.providers {
            let provider = Self::create_provider(name, provider_config)?;
            providers.insert(name.clone(), provider);
            info!("Initialized provider: {}", name);
        }

        // Validate that the default provider exists
        if !providers.contains_key(&config.routing.default_provider) {
            return Err(GatewayError::Configuration(format!(
                "Default provider '{}' not found in configuration",
                config.routing.default_provider
            )));
        }

        // Validate fallback providers exist
        for fallback in &config.routing.fallback_providers {
            if !providers.contains_key(fallback) {
                return Err(GatewayError::Configuration(format!(
                    "Fallback provider '{}' not found in configuration",
                    fallback
                )));
            }
        }

        info!(
            "Router initialized with default provider: {}, fallbacks: {:?}",
            config.routing.default_provider, config.routing.fallback_providers
        );

        Ok(Self {
            providers,
            default_provider: config.routing.default_provider.clone(),
            fallback_providers: config.routing.fallback_providers.clone(),
        })
    }

    /// Create a provider instance from configuration
    fn create_provider(
        name: &str,
        config: &ProviderConfig,
    ) -> Result<Box<dyn Provider>, GatewayError> {
        let provider: Box<dyn Provider> = match name {
            "openai" => Box::new(OpenAiProvider::new(
                config.api_key.clone(),
                config.base_url.clone(),
            )),
            "anthropic" => Box::new(AnthropicProvider::new(
                config.api_key.clone(),
                config.base_url.clone(),
            )),
            "google" => Box::new(GoogleProvider::new(
                config.api_key.clone(),
                config.base_url.clone(),
            )),
            _ => {
                // Treat unknown providers as custom OpenAI-compatible providers
                Box::new(CustomProvider::new(
                    name.to_string(),
                    config.api_key.clone(),
                    config.base_url.clone(),
                    config.models.clone(),
                ))
            }
        };
        Ok(provider)
    }

    /// Route a chat completion request to the appropriate provider
    pub async fn route(&self, request: ChatRequest) -> Result<ChatResponse, GatewayError> {
        // Determine which provider to use
        let provider_name = request
            .provider
            .as_deref()
            .unwrap_or(&self.default_provider);

        // Try the primary provider first
        if let Some(provider) = self.providers.get(provider_name) {
            if provider.supports_model(&request.model) {
                return provider.complete_chat(request).await;
            } else {
                warn!(
                    "Model '{}' not supported by provider '{}', trying fallbacks",
                    request.model, provider_name
                );
            }
        } else {
            warn!(
                "Provider '{}' not found, trying fallbacks",
                provider_name
            );
        }

        // Try fallback providers
        for fallback_name in &self.fallback_providers {
            if let Some(provider) = self.providers.get(fallback_name) {
                if provider.supports_model(&request.model) {
                    info!(
                        "Routing to fallback provider '{}' for model '{}'",
                        fallback_name, request.model
                    );
                    return provider.complete_chat(request).await;
                }
            }
        }

        Err(GatewayError::ModelNotSupported {
            model: request.model,
            provider: provider_name.to_string(),
        })
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn Provider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// List all available provider names
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// List all available models across all providers
    pub fn available_models(&self) -> Vec<String> {
        let mut models = Vec::new();
        for provider in self.providers.values() {
            for model in provider.supported_models() {
                if !models.contains(&model.to_string()) {
                    models.push(model.to_string());
                }
            }
        }
        models
    }

    /// Check if a model is supported by any provider
    pub fn is_model_supported(&self, model: &str) -> bool {
        self.providers
            .values()
            .any(|p| p.supports_model(model))
    }

    /// Get the default provider name
    pub fn default_provider_name(&self) -> &str {
        &self.default_provider
    }
}

#[cfg(test)]
mod tests {
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
                models: vec!["claude-3-opus".to_string()],
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
        assert!(models.contains(&"claude-3-opus".to_string()));
    }

    #[test]
    fn test_is_model_supported() {
        let config = test_config();
        let router = Router::new(&config).unwrap();
        assert!(router.is_model_supported("gpt-4"));
        assert!(router.is_model_supported("claude-3-opus"));
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
}
