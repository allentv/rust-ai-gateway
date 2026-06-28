use axum::http::StatusCode;
use axum::Json;
use gateway_core::error::GatewayError;
use gateway_core::types::{ChatRequest, TokenUsage};
use serde_json::{json, Value};

pub async fn chat_completion(
    Json(request): Json<ChatRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!(
        model = %request.model,
        messages = request.messages.len(),
        stream = request.stream,
        "Received chat completion request"
    );

    // Validate that the request has messages
    if request.messages.is_empty() {
        return Err(
            GatewayError::Serialization("Messages cannot be empty".to_string()).into(),
        );
    }

    // TODO: Route to the appropriate provider once router is integrated
    // For now, return a placeholder response
    let usage = TokenUsage::new(10, 20);
    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let created = chrono::Utc::now();

    let body = json!({
        "id": id,
        "object": "chat.completion",
        "created": created.timestamp(),
        "model": request.model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": format!(
                    "Echo: Received {} messages for model '{}'",
                    request.messages.len(),
                    request.model
                ),
            },
            "finish_reason": "stop",
        }],
        "usage": {
            "prompt_tokens": usage.prompt_tokens,
            "completion_tokens": usage.completion_tokens,
            "total_tokens": usage.total_tokens,
        },
    });

    Ok(Json(body))
}
