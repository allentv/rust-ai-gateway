use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::error::GatewayError;
use crate::types::{ChatChunk, ChatRequest, ChatResponse};

/// Core provider trait that all AI providers must implement
#[async_trait]
pub trait Provider: Send + Sync {
    /// Send a chat completion request and get a complete response
    async fn complete_chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, GatewayError>;

    /// Send a chat completion request and get a streaming response
    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatChunk, GatewayError>>, GatewayError>;

    /// Provider name (e.g., "openai", "anthropic")
    fn name(&self) -> &str;

    /// Whether this provider supports streaming
    fn supports_streaming(&self) -> bool {
        true
    }

    /// List of models supported by this provider
    fn supported_models(&self) -> Vec<&str>;

    /// Check if a model is supported
    fn supports_model(&self, model: &str) -> bool {
        self.supported_models().iter().any(|m| m == &model)
    }
}
