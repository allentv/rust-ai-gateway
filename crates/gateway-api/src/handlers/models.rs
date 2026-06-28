use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::AppState;

/// List all available models across providers (OpenAI-compatible /v1/models endpoint)
pub async fn list_models(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let models = state.router.available_models_with_providers();

    let data: Vec<Value> = models
        .iter()
        .map(|(model, provider)| {
            json!({
                "id": model,
                "object": "model",
                "created": 0,
                "owned_by": provider,
            })
        })
        .collect();

    Ok(Json(json!({
        "object": "list",
        "data": data,
    })))
}
