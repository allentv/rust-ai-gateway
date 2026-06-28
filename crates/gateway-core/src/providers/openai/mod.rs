use std::time::Duration;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::GatewayError;
use crate::providers::traits::Provider;
use crate::types::{ChatChunk, ChatRequest, ChatResponse, Delta, Message, Role, TokenUsage};

const OPENAI_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4-turbo",
    "gpt-4",
    "gpt-3.5-turbo",
];

/// OpenAI API request format
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// OpenAI API response format
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    id: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
    model: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI streaming response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAiStreamChunk {
    id: String,
    choices: Vec<OpenAiStreamChoice>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    role: Option<String>,
    content: Option<String>,
}

/// OpenAI provider implementation
pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

/// Default request timeout for OpenAI API calls (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

impl OpenAiProvider {
    /// Create a new OpenAI provider with default timeout
    pub fn new(api_key: String, base_url: String) -> Self {
        Self::with_timeout(api_key, base_url, DEFAULT_TIMEOUT)
    }

    /// Create a new OpenAI provider with a custom request timeout
    pub fn with_timeout(api_key: String, base_url: String, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build reqwest client");
        Self {
            client,
            api_key,
            base_url,
        }
    }

    fn convert_messages(messages: &[Message]) -> Vec<OpenAiMessage> {
        messages
            .iter()
            .map(|m| OpenAiMessage {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: m.content.clone(),
                name: m.name.clone(),
            })
            .collect()
    }

    fn map_role(role: Option<String>) -> Option<String> {
        role.filter(|r| !r.is_empty())
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    #[instrument(skip(self, request), fields(provider = "openai"))]
    async fn complete_chat(&self, request: ChatRequest) -> Result<ChatResponse, GatewayError> {
        if !self.supports_model(&request.model) {
            return Err(GatewayError::ModelNotSupported {
                model: request.model.clone(),
                provider: self.name().to_string(),
            });
        }

        let openai_request = OpenAiRequest {
            model: request.model.clone(),
            messages: Self::convert_messages(&request.messages),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: false,
        };

        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| GatewayError::provider("openai", e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(GatewayError::provider(
                "openai",
                format!("HTTP {status}: {body}"),
            ));
        }

        let openai_response: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| GatewayError::provider("openai", e.to_string()))?;

        let content = openai_response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        let usage = openai_response
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            })
            .unwrap_or_else(|| TokenUsage::new(0, 0));

        Ok(ChatResponse {
            id: openai_response.id,
            content,
            usage,
            model: openai_response.model,
            provider: self.name().to_string(),
            created_at: chrono::Utc::now(),
        })
    }

    #[instrument(skip(self, request), fields(provider = "openai"))]
    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatChunk, GatewayError>>, GatewayError> {
        if !self.supports_model(&request.model) {
            return Err(GatewayError::ModelNotSupported {
                model: request.model.clone(),
                provider: self.name().to_string(),
            });
        }

        let openai_request = OpenAiRequest {
            model: request.model.clone(),
            messages: Self::convert_messages(&request.messages),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: true,
        };

        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| GatewayError::provider("openai", e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(GatewayError::provider(
                "openai",
                format!("HTTP {status}: {body}"),
            ));
        }

        let stream = response
            .bytes_stream()
            .flat_map(|result| {
                match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        // SSE format: "data: {...}\n\n" or "data: [DONE]\n\n"
                        let mut chunks = Vec::new();
                        for line in text.lines() {
                            let line = line.trim();
                            if line.is_empty() || line.starts_with(':') {
                                continue;
                            }
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    continue;
                                }
                                match serde_json::from_str::<OpenAiStreamChunk>(data) {
                                    Ok(chunk) => {
                                        for choice in chunk.choices {
                                            chunks.push(Ok(ChatChunk {
                                                id: chunk.id.clone(),
                                                model: chunk.model.clone(),
                                                delta: Delta {
                                                    role: Self::map_role(choice.delta.role),
                                                    content: choice.delta.content,
                                                },
                                                finish_reason: choice.finish_reason,
                                                usage: None,
                                            }));
                                        }
                                    }
                                    Err(_) => continue,
                                }
                            }
                        }
                        futures::stream::iter(chunks).boxed()
                    }
                    Err(e) => futures::stream::once(async move {
                        Err(GatewayError::provider("openai", e.to_string()))
                    })
                    .boxed(),
                }
            })
            .boxed();

        Ok(stream)
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn supported_models(&self) -> Vec<&str> {
        OPENAI_MODELS.to_vec()
    }
}

#[cfg(test)]
mod tests;
