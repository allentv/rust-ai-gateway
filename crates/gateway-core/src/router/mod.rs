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
            warn!("Provider '{}' not found, trying fallbacks", provider_name);
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
        self.providers.values().any(|p| p.supports_model(model))
    }

    /// Get the default provider name
    pub fn default_provider_name(&self) -> &str {
        &self.default_provider
    }
}

#[cfg(test)]
mod tests;
