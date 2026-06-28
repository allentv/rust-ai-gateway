use async_trait::async_trait;
use futures::stream::BoxStream;
use tracing::instrument;

use crate::error::GatewayError;
use crate::providers::traits::Provider;
use crate::types::{ChatChunk, ChatRequest, ChatResponse};

/// Custom OpenAI-compatible provider
#[allow(dead_code)]
pub struct CustomProvider {
    name: String,
    api_key: String,
    base_url: String,
    models: Vec<String>,
    client: reqwest::Client,
}

impl CustomProvider {
    pub fn new(name: String, api_key: String, base_url: String, models: Vec<String>) -> Self {
        Self {
            name,
            api_key,
            base_url,
            models,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Provider for CustomProvider {
    #[instrument(skip(self), fields(model = %request.model))]
    async fn complete_chat(&self, request: ChatRequest) -> Result<ChatResponse, GatewayError> {
        let _ = request;
        Err(GatewayError::Internal(format!(
            "Custom provider '{}' not yet implemented",
            self.name
        )))
    }

    #[instrument(skip(self), fields(model = %request.model))]
    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatChunk, GatewayError>>, GatewayError> {
        let _ = request;
        Err(GatewayError::Internal(format!(
            "Custom provider '{}' not yet implemented",
            self.name
        )))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supported_models(&self) -> Vec<&str> {
        self.models.iter().map(|s| s.as_str()).collect()
    }
}
