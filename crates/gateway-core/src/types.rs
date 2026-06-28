use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A message in a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Role of a message participant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// Request for chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Response from chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub content: String,
    pub usage: TokenUsage,
    pub model: String,
    pub provider: String,
    pub created_at: DateTime<Utc>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// A single chunk in a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub id: String,
    pub delta: Delta,
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
}

/// Delta content in a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Unique request identifier
#[derive(Debug, Clone)]
pub struct RequestId(String);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_new_calculates_total() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_token_usage_new_zeros() {
        let usage = TokenUsage::new(0, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn test_request_id_new_produces_valid_uuid() {
        let id = RequestId::new();
        let uuid_str = id.as_str();
        // UUID v4 format: 8-4-4-4-12 hex digits
        assert_eq!(uuid_str.len(), 36);
        assert_eq!(uuid_str.chars().filter(|c| *c == '-').count(), 4);
        // Verify it parses as a valid UUID
        assert!(uuid::Uuid::parse_str(uuid_str).is_ok());
    }

    #[test]
    fn test_request_id_as_str() {
        let id = RequestId::new();
        let s = id.as_str();
        assert!(!s.is_empty());
        assert_eq!(s.len(), 36);
    }

    #[test]
    fn test_request_id_display() {
        let id = RequestId::new();
        let displayed = format!("{id}");
        assert_eq!(displayed, id.as_str());
    }

    #[test]
    fn test_request_id_default() {
        let id = RequestId::default();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn test_request_id_unique() {
        let id1 = RequestId::new();
        let id2 = RequestId::new();
        assert_ne!(id1.as_str(), id2.as_str());
    }

    #[test]
    fn test_message_serialization_roundtrip() {
        let msg = Message {
            role: Role::User,
            content: "Hello, world!".to_string(),
            name: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_message_serialization_with_name() {
        let msg = Message {
            role: Role::Assistant,
            content: "Hi there!".to_string(),
            name: Some("assistant".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"name\":\"assistant\""));
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_message_role_serialization() {
        let roles = vec![
            (Role::System, "\"system\""),
            (Role::User, "\"user\""),
            (Role::Assistant, "\"assistant\""),
            (Role::Tool, "\"tool\""),
        ];
        for (role, expected_json) in roles {
            let msg = Message {
                role,
                content: "test".to_string(),
                name: None,
            };
            let json = serde_json::to_string(&msg).unwrap();
            assert!(
                json.contains(expected_json),
                "Expected {expected_json} in {json}"
            );
        }
    }

    #[test]
    fn test_chat_request_serialization_roundtrip() {
        let req = ChatRequest {
            messages: vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
                name: None,
            }],
            model: "gpt-4".to_string(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            stream: false,
            provider: Some("openai".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.model, deserialized.model);
        assert_eq!(req.max_tokens, deserialized.max_tokens);
        assert_eq!(req.temperature, deserialized.temperature);
        assert!(!deserialized.stream);
    }

    #[test]
    fn test_chat_request_defaults() {
        let json = r#"{
            "messages": [{"role": "user", "content": "Hi"}],
            "model": "gpt-4"
        }"#;
        let req: ChatRequest = serde_json::from_str(json).unwrap();
        assert!(!req.stream);
        assert!(req.max_tokens.is_none());
        assert!(req.temperature.is_none());
        assert!(req.provider.is_none());
    }

    #[test]
    fn test_chat_response_serialization_roundtrip() {
        let resp = ChatResponse {
            id: "test-id".to_string(),
            content: "Hello!".to_string(),
            usage: TokenUsage::new(10, 20),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ChatResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp.id, deserialized.id);
        assert_eq!(resp.content, deserialized.content);
        assert_eq!(resp.model, deserialized.model);
    }

    #[test]
    fn test_chat_chunk_serialization_roundtrip() {
        let chunk = ChatChunk {
            id: "chunk-1".to_string(),
            delta: Delta {
                role: Some("assistant".to_string()),
                content: Some("Hello".to_string()),
            },
            finish_reason: None,
            usage: Some(TokenUsage::new(5, 10)),
        };
        let json = serde_json::to_string(&chunk).unwrap();
        let deserialized: ChatChunk = serde_json::from_str(&json).unwrap();
        assert_eq!(chunk.id, deserialized.id);
        assert_eq!(chunk.delta.content, deserialized.delta.content);
        assert!(deserialized.finish_reason.is_none());
    }

    #[test]
    fn test_chat_chunk_minimal() {
        let chunk = ChatChunk {
            id: "chunk-2".to_string(),
            delta: Delta {
                role: None,
                content: Some(" world".to_string()),
            },
            finish_reason: Some("stop".to_string()),
            usage: None,
        };
        let json = serde_json::to_string(&chunk).unwrap();
        // role should be skipped when None
        assert!(!json.contains("\"role\""));
        // usage should be skipped when None
        assert!(!json.contains("\"usage\""));
        assert!(json.contains("\"finish_reason\":\"stop\""));
    }
}
