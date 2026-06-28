use async_trait::async_trait;
use futures::stream::BoxStream;
use tracing::instrument;

use crate::error::GatewayError;
use crate::types::{ChatChunk, ChatRequest, ChatResponse};
use crate::providers::traits::Provider;

const GOOGLE_MODELS: &[&str] = &[
    "gemini-2.0-flash",
    "gemini-2.0-pro",
    "gemini-1.5-flash",
    "gemini-1.5-pro",
];

/// Google AI (Gemini) provider
#[allow(dead_code)]
pub struct GoogleProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl GoogleProvider {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    #[instrument(skip(self), fields(model = %request.model))]
    async fn complete_chat(
        &self,
        _request: ChatRequest,
    ) -> Result<ChatResponse, GatewayError> {
        Err(GatewayError::Internal(
            "Google provider not yet implemented".to_string(),
        ))
    }

    #[instrument(skip(self), fields(model = %request.model))]
    async fn stream_chat(
        &self,
        _request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatChunk, GatewayError>>, GatewayError> {
        Err(GatewayError::Internal(
            "Google provider not yet implemented".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "google"
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supported_models(&self) -> Vec<&str> {
        GOOGLE_MODELS.to_vec()
    }
}
