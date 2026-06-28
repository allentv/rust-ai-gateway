use super::*;
use gateway_config::schema::{
    CacheConfig, MeteringConfig, MetricType, ProviderConfig, RateLimitConfig,
};

#[tokio::test]
async fn test_rate_limiter_creation() {
    let mut providers = HashMap::new();
    providers.insert(
        "openai".to_string(),
        ProviderConfig {
            api_key: "test".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            models: vec![],
            rate_limit: Some(RateLimitConfig {
                requests_per_minute: Some(60),
                tokens_per_minute: None,
            }),
        },
    );

    let limiter = ProviderRateLimiter::from_config(&providers);
    assert!(limiter.check_rate_limit("openai").await.is_ok());
    assert!(limiter.check_rate_limit("unknown").await.is_ok());
}

#[tokio::test]
async fn test_cache_operations() {
    let config = CacheConfig {
        enabled: true,
        ttl_seconds: 60,
        max_size: 100,
    };
    let cache = ChatCache::new(&config);

    let request = ChatRequest {
        messages: vec![],
        model: "gpt-4".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        provider: None,
    };

    // Cache miss
    assert!(cache.get(&request).await.is_none());

    let response = ChatResponse {
        id: "test".to_string(),
        content: "hello".to_string(),
        usage: TokenUsage::new(10, 20),
        model: "gpt-4".to_string(),
        provider: "openai".to_string(),
        created_at: chrono::Utc::now(),
    };

    cache.put(&request, &response).await;

    // Cache hit
    let cached = cache.get(&request).await;
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().content, "hello");
}

#[test]
fn test_auth_middleware() {
    let middleware = AuthMiddleware::new(vec!["key1".to_string()], true);

    // Valid key
    assert!(middleware.validate_api_key(Some("key1")).is_ok());

    // Invalid key
    assert!(middleware.validate_api_key(Some("wrong")).is_err());

    // Missing key when required
    assert!(middleware.validate_api_key(None).is_err());

    // Missing key when not required
    let middleware2 = AuthMiddleware::new(vec![], false);
    assert!(middleware2.validate_api_key(None).is_ok());
}

#[tokio::test]
async fn test_cost_meter() {
    let config = MeteringConfig {
        enabled: true,
        metrics: vec![MetricType::RequestCount, MetricType::TokenUsage],
    };
    let meter = CostMeter::new(&config);

    let usage = TokenUsage::new(100, 50);
    meter.record_request("openai", &usage).await;

    let stats = meter.stats().await;
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.total_prompt_tokens, 100);
    assert_eq!(stats.total_completion_tokens, 50);
    assert_eq!(stats.total_tokens, 150);
    assert_eq!(stats.provider_requests.get("openai"), Some(&1));
}
