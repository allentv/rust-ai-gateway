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
