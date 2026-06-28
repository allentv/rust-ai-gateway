use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures::StreamExt;
use gateway_core::error::GatewayError;
use gateway_core::types::ChatRequest;
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;

use crate::AppState;

/// Handle chat completion requests (both streaming and non-streaming)
pub async fn chat_completion(
    Extension(state): Extension<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    tracing::info!(
        model = %request.model,
        messages = request.messages.len(),
        stream = request.stream,
        "Received chat completion request"
    );

    // Validate that the request has messages
    if request.messages.is_empty() {
        return Err(GatewayError::Serialization("Messages cannot be empty".to_string()).into());
    }

    if request.stream {
        handle_streaming(state, request).await
    } else {
        handle_non_streaming(state, request).await
    }
}

/// Handle non-streaming chat completion requests
async fn handle_non_streaming(
    state: Arc<AppState>,
    request: ChatRequest,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let response = state.router.route(request).await?;

    let body = json!({
        "id": response.id,
        "object": "chat.completion",
        "created": response.created_at.timestamp(),
        "model": response.model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": response.content,
            },
            "finish_reason": "stop",
        }],
        "usage": {
            "prompt_tokens": response.usage.prompt_tokens,
            "completion_tokens": response.usage.completion_tokens,
            "total_tokens": response.usage.total_tokens,
        },
    });

    Ok(Json(body).into_response())
}

/// Handle streaming chat completion requests via SSE
async fn handle_streaming(
    state: Arc<AppState>,
    request: ChatRequest,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let stream = state.router.route_stream(request).await?;

    let sse_stream = stream
        .map(|result| match result {
            Ok(chunk) => {
                let data = json!({
                    "id": chunk.id,
                    "object": "chat.completion.chunk",
                    "created": chrono::Utc::now().timestamp(),
                    "model": chunk.model,
                    "choices": [{
                        "index": 0,
                        "delta": {
                            "role": chunk.delta.role,
                            "content": chunk.delta.content,
                        },
                        "finish_reason": chunk.finish_reason,
                    }],
                });
                Ok::<_, Infallible>(Event::default().data(data.to_string()))
            }
            Err(e) => {
                tracing::error!("Stream error: {}", e);
                let data = json!({
                    "error": {
                        "message": e.to_string(),
                    }
                });
                Ok::<_, Infallible>(Event::default().data(data.to_string()))
            }
        })
        .chain(futures::stream::once(async {
            Ok::<_, Infallible>(Event::default().data("[DONE]"))
        }));

    Ok(Sse::new(sse_stream).into_response())
}
