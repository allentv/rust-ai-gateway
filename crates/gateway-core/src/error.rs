/// Core error types for the gateway
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Provider error ({provider}): {message}")]
    Provider {
        provider: String,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Provider '{0}' not found")]
    ProviderNotFound(String),

    #[error("Model '{model}' not supported by provider '{provider}'")]
    ModelNotSupported { model: String, provider: String },

    #[error("Request timeout after {0}ms")]
    Timeout(u64),

    #[error("Rate limit exceeded for provider '{0}'")]
    RateLimitExceeded(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Request serialization error: {0}")]
    Serialization(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Stream closed unexpectedly")]
    StreamClosed,

    #[error("Internal error: {0}")]
    Internal(String),
}

impl GatewayError {
    pub fn provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn provider_with_source(
        provider: impl Into<String>,
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
            source: Some(source),
        }
    }
}

impl From<GatewayError> for (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    fn from(err: GatewayError) -> Self {
        let (status, error_type) = match &err {
            GatewayError::ProviderNotFound(_) => {
                (axum::http::StatusCode::NOT_FOUND, "provider_not_found")
            }
            GatewayError::ModelNotSupported { .. } => {
                (axum::http::StatusCode::BAD_REQUEST, "model_not_supported")
            }
            GatewayError::Timeout(_) => (
                axum::http::StatusCode::GATEWAY_TIMEOUT,
                "request_timeout",
            ),
            GatewayError::RateLimitExceeded(_) => (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                "rate_limit_exceeded",
            ),
            GatewayError::Authentication(_) => (
                axum::http::StatusCode::UNAUTHORIZED,
                "authentication_error",
            ),
            GatewayError::Configuration(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "configuration_error",
            ),
            GatewayError::Serialization(_) => (
                axum::http::StatusCode::BAD_REQUEST,
                "serialization_error",
            ),
            GatewayError::Network(_) => (
                axum::http::StatusCode::BAD_GATEWAY,
                "network_error",
            ),
            GatewayError::StreamClosed => (
                axum::http::StatusCode::BAD_GATEWAY,
                "stream_closed",
            ),
            GatewayError::Provider { .. } => (
                axum::http::StatusCode::BAD_GATEWAY,
                "provider_error",
            ),
            GatewayError::Internal(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
            ),
        };

        let body = serde_json::json!({
            "error": {
                "type": error_type,
                "message": err.to_string(),
            }
        });

        (status, axum::Json(body))
    }
}
