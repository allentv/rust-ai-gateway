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
            GatewayError::Timeout(_) => {
                (axum::http::StatusCode::GATEWAY_TIMEOUT, "request_timeout")
            }
            GatewayError::RateLimitExceeded(_) => (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                "rate_limit_exceeded",
            ),
            GatewayError::Authentication(_) => {
                (axum::http::StatusCode::UNAUTHORIZED, "authentication_error")
            }
            GatewayError::Configuration(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "configuration_error",
            ),
            GatewayError::Serialization(_) => {
                (axum::http::StatusCode::BAD_REQUEST, "serialization_error")
            }
            GatewayError::Network(_) => (axum::http::StatusCode::BAD_GATEWAY, "network_error"),
            GatewayError::StreamClosed => (axum::http::StatusCode::BAD_GATEWAY, "stream_closed"),
            GatewayError::Provider { .. } => {
                (axum::http::StatusCode::BAD_GATEWAY, "provider_error")
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_provider_constructor() {
        let err = GatewayError::provider("openai", "something went wrong");
        match &err {
            GatewayError::Provider {
                provider,
                message,
                source,
            } => {
                assert_eq!(provider, "openai");
                assert_eq!(message, "something went wrong");
                assert!(source.is_none());
            }
            _ => panic!("Expected Provider variant"),
        }
        assert!(err.to_string().contains("openai"));
        assert!(err.to_string().contains("something went wrong"));
    }

    #[test]
    fn test_provider_with_source_constructor() {
        let source = std::io::Error::new(std::io::ErrorKind::Other, "io error");
        let err = GatewayError::provider_with_source("anthropic", "api failed", Box::new(source));
        match &err {
            GatewayError::Provider {
                provider,
                message,
                source,
            } => {
                assert_eq!(provider, "anthropic");
                assert_eq!(message, "api failed");
                assert!(source.is_some());
            }
            _ => panic!("Expected Provider variant"),
        }
    }

    #[test]
    fn test_error_to_http_provider_not_found() {
        let err = GatewayError::ProviderNotFound("ollama".to_string());
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_to_http_model_not_supported() {
        let err = GatewayError::ModelNotSupported {
            model: "gpt-5".to_string(),
            provider: "openai".to_string(),
        };
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_to_http_timeout() {
        let err = GatewayError::Timeout(5000);
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
    }

    #[test]
    fn test_error_to_http_rate_limit_exceeded() {
        let err = GatewayError::RateLimitExceeded("openai".to_string());
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_error_to_http_authentication() {
        let err = GatewayError::Authentication("invalid key".to_string());
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_error_to_http_configuration() {
        let err = GatewayError::Configuration("bad config".to_string());
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_to_http_serialization() {
        let err = GatewayError::Serialization("invalid json".to_string());
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_error_to_http_network() {
        let reqwest_err = reqwest::Client::new()
            .get("http://127.0.0.1:1")
            .send()
            .await
            .unwrap_err();
        let err = GatewayError::Network(reqwest_err);
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_error_to_http_stream_closed() {
        let err = GatewayError::StreamClosed;
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_error_to_http_provider() {
        let err = GatewayError::provider("openai", "upstream error");
        let (status, body) = err.into();
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        // Verify the JSON body structure
        let error_obj = body.get("error").unwrap();
        assert_eq!(error_obj.get("type").unwrap(), "provider_error");
        assert!(error_obj
            .get("message")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("openai"));
    }

    #[test]
    fn test_error_to_http_internal() {
        let err = GatewayError::Internal("oops".to_string());
        let (status, _) = err.into();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_display_messages() {
        let cases: Vec<(GatewayError, &str)> = vec![
            (
                GatewayError::ProviderNotFound("test".to_string()),
                "not found",
            ),
            (GatewayError::Timeout(1000), "1000ms"),
            (
                GatewayError::RateLimitExceeded("test".to_string()),
                "Rate limit",
            ),
            (
                GatewayError::Authentication("bad key".to_string()),
                "bad key",
            ),
            (GatewayError::StreamClosed, "Stream closed"),
            (GatewayError::Internal("oops".to_string()), "oops"),
        ];
        for (err, expected_substr) in cases {
            let msg = err.to_string();
            assert!(
                msg.contains(expected_substr),
                "Expected '{expected_substr}' in '{msg}'"
            );
        }
    }
}
