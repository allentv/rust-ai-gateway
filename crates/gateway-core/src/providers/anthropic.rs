use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::GatewayError;
use crate::types::{ChatChunk, ChatRequest, ChatResponse, Delta, Message, Role, TokenUsage};
use crate::providers::traits::Provider;

const ANTHROPIC_MODELS: &[&str] = &[
    "claude-sonnet-4-20250514",
    "claude-3-5-sonnet-20241022",
    "claude-3-5-haiku-20241022",
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
];

/// Anthropic API request format
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    system: Option<String>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response format
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    content: Vec<AnthropicContent>,
    model: String,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic streaming response events
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicStreamEvent {
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        delta: AnthropicDelta,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "message_start")]
    MessageStart {
        message: AnthropicStreamMessage,
    },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: usize },
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamMessage {
    id: String,
}

/// Anthropic provider implementation
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, base_url: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            api_key,
            base_url,
        }
    }

    fn convert_messages(messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system = None;
        let mut api_messages = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    system = Some(msg.content.clone());
                }
                Role::User => {
                    api_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
                Role::Assistant => {
                    api_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: msg.content.clone(),
                    });
                }
                Role::Tool => {
                    // Anthropic doesn't support tool messages in the same way
                    api_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
            }
        }

        (system, api_messages)
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    #[instrument(skip(self, request), fields(provider = "anthropic"))]
    async fn complete_chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, GatewayError> {
        if !self.supports_model(&request.model) {
            return Err(GatewayError::ModelNotSupported {
                model: request.model.clone(),
                provider: self.name().to_string(),
            });
        }

        let (system, messages) = Self::convert_messages(&request.messages);

        let anthropic_request = AnthropicRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            system,
            stream: false,
        };

        let url = format!("{}/messages", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| GatewayError::provider("anthropic", e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(GatewayError::provider(
                "anthropic",
                format!("HTTP {status}: {body}"),
            ));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| GatewayError::provider("anthropic", e.to_string()))?;

        let content = anthropic_response
            .content
            .first()
            .and_then(|c| c.text.clone())
            .unwrap_or_default();

        let usage = anthropic_response
            .usage
            .map(|u| TokenUsage::new(u.input_tokens, u.output_tokens))
            .unwrap_or_else(|| TokenUsage::new(0, 0));

        Ok(ChatResponse {
            id: anthropic_response.id,
            content,
            usage,
            model: anthropic_response.model,
            provider: self.name().to_string(),
            created_at: chrono::Utc::now(),
        })
    }

    #[instrument(skip(self, request), fields(provider = "anthropic"))]
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

        let (system, messages) = Self::convert_messages(&request.messages);

        let anthropic_request = AnthropicRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            system,
            stream: true,
        };

        let url = format!("{}/messages", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| GatewayError::provider("anthropic", e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(GatewayError::provider(
                "anthropic",
                format!("HTTP {status}: {body}"),
            ));
        }

        let mut message_id = String::new();

        let stream = response
            .bytes_stream()
            .flat_map(move |result| {
                let id = message_id.clone();
                let chunks = match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        let mut chunks = Vec::new();
                        for line in text.lines() {
                            let line = line.trim();
                            if line.is_empty() || !line.starts_with("data: ") {
                                continue;
                            }
                            let data = &line[6..];
                            match serde_json::from_str::<AnthropicStreamEvent>(data) {
                                Ok(event) => match event {
                                    AnthropicStreamEvent::ContentBlockDelta { delta } => {
                                        if let Some(text) = delta.text {
                                            chunks.push(Ok(ChatChunk {
                                                id: id.clone(),
                                                delta: Delta {
                                                    role: None,
                                                    content: Some(text),
                                                },
                                                finish_reason: None,
                                                usage: None,
                                            }));
                                        }
                                    }
                                    AnthropicStreamEvent::MessageStop => {
                                        chunks.push(Ok(ChatChunk {
                                            id: id.clone(),
                                            delta: Delta {
                                                role: None,
                                                content: None,
                                            },
                                            finish_reason: Some("stop".to_string()),
                                            usage: None,
                                        }));
                                    }
                                    _ => {}
                                },
                                Err(_) => continue,
                            }
                        }
                        futures::stream::iter(chunks).boxed()
                    }
                    Err(e) => {
                        futures::stream::once(async move {
                            Err(GatewayError::provider("anthropic", e.to_string()))
                        })
                        .boxed()
                    }
                };
                futures::future::ready(chunks)
            })
            .boxed();

        Ok(stream)
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn supported_models(&self) -> Vec<&str> {
        ANTHROPIC_MODELS.to_vec()
    }
}
