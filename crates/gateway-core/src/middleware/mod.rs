use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use governor::clock::Clock;
use governor::{Quota, RateLimiter};
use moka::future::Cache as MokaCache;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::GatewayError;
use crate::types::{ChatRequest, ChatResponse, TokenUsage};

// ===== Rate Limiter =====

/// Per-provider rate limiter using the governor crate
pub struct ProviderRateLimiter {
    limiters: HashMap<String, governor::DefaultDirectRateLimiter>,
}

impl ProviderRateLimiter {
    /// Create rate limiters from provider configurations
    pub fn from_config(
        providers: &HashMap<String, gateway_config::schema::ProviderConfig>,
    ) -> Self {
        let mut limiters = HashMap::new();

        for (name, config) in providers {
            if let Some(rate_limit) = &config.rate_limit {
                if let Some(rpm) = rate_limit.requests_per_minute {
                    if let Some(non_zero) = NonZeroU32::new(rpm) {
                        let quota = Quota::per_minute(non_zero);
                        let limiter = RateLimiter::direct(quota);
                        limiters.insert(name.clone(), limiter);
                        info!(
                            "Rate limiter initialized for provider '{}': {} req/min",
                            name, rpm
                        );
                    }
                }
            }
        }

        Self { limiters }
    }

    /// Check if a request is allowed for the given provider
    pub async fn check_rate_limit(&self, provider: &str) -> Result<(), GatewayError> {
        if let Some(limiter) = self.limiters.get(provider) {
            match limiter.check() {
                Ok(()) => {
                    debug!("Rate limit check passed for provider '{}'", provider);
                    Ok(())
                }
                Err(not_until) => {
                    warn!("Rate limit exceeded for provider '{}'", provider);
                    let clock = governor::clock::QuantaClock::default();
                    Err(GatewayError::RateLimitExceeded(format!(
                        "Rate limit exceeded for provider '{}'. Retry after {:?}",
                        provider,
                        not_until.wait_time_from(clock.now())
                    )))
                }
            }
        } else {
            Ok(()) // No rate limiter configured for this provider
        }
    }
}

// ===== Cache =====

/// Cache for chat responses using moka
pub struct ChatCache {
    cache: MokaCache<String, ChatResponse>,
    enabled: bool,
}

impl ChatCache {
    /// Create a new cache from configuration
    pub fn new(config: &gateway_config::schema::CacheConfig) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(config.max_size)
            .time_to_live(Duration::from_secs(config.ttl_seconds))
            .build();

        info!(
            "Chat cache initialized: enabled={}, ttl={}s, max_size={}",
            config.enabled, config.ttl_seconds, config.max_size
        );

        Self {
            cache,
            enabled: config.enabled,
        }
    }

    /// Generate a cache key from a chat request
    fn cache_key(request: &ChatRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.model.hash(&mut hasher);

        // Serialize messages to bytes for hashing since Message doesn't derive Hash
        if let Ok(msg_json) = serde_json::to_vec(&request.messages) {
            msg_json.hash(&mut hasher);
        }

        request.temperature.map(|t| t.to_bits()).hash(&mut hasher);
        request.max_tokens.hash(&mut hasher);

        format!("chat:{:x}", hasher.finish())
    }

    /// Get a cached response if available
    pub async fn get(&self, request: &ChatRequest) -> Option<ChatResponse> {
        if !self.enabled {
            return None;
        }
        let key = Self::cache_key(request);
        let result = self.cache.get(&key).await;
        if result.is_some() {
            debug!("Cache hit for key: {}", key);
        }
        result
    }

    /// Store a response in the cache
    pub async fn put(&self, request: &ChatRequest, response: &ChatResponse) {
        if !self.enabled {
            return;
        }
        let key = Self::cache_key(request);
        debug!("Cache put for key: {}", key);
        self.cache.insert(key, response.clone()).await;
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        self.cache.invalidate_all();
        info!("Chat cache cleared");
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            enabled: self.enabled,
            entry_count: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub enabled: bool,
    pub entry_count: u64,
    pub weighted_size: u64,
}

// ===== Auth Middleware =====

/// API key authentication middleware
pub struct AuthMiddleware {
    api_keys: Vec<String>,
    required: bool,
}

impl AuthMiddleware {
    /// Create a new auth middleware with the given API keys
    pub fn new(api_keys: Vec<String>, required: bool) -> Self {
        info!(
            "Auth middleware initialized: required={}, keys_configured={}",
            required,
            api_keys.len()
        );
        Self { api_keys, required }
    }

    /// Validate an API key
    pub fn validate_api_key(&self, key: Option<&str>) -> Result<(), GatewayError> {
        match key {
            Some(key) => {
                if self.api_keys.is_empty() || self.api_keys.contains(&key.to_string()) {
                    debug!("API key validated successfully");
                    Ok(())
                } else {
                    warn!("Invalid API key provided");
                    Err(GatewayError::Authentication("Invalid API key".to_string()))
                }
            }
            None => {
                if self.required {
                    warn!("API key required but not provided");
                    Err(GatewayError::Authentication("API key required".to_string()))
                } else {
                    Ok(())
                }
            }
        }
    }
}

// ===== Cost Meter =====

/// Tracks token usage and request counts for metering
pub struct CostMeter {
    total_requests: AtomicU64,
    total_prompt_tokens: AtomicU64,
    total_completion_tokens: AtomicU64,
    provider_requests: RwLock<HashMap<String, u64>>,
    enabled: bool,
}

/// Cost metering statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeteringStats {
    pub total_requests: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    pub provider_requests: HashMap<String, u64>,
}

impl CostMeter {
    /// Create a new cost meter from configuration
    pub fn new(config: &gateway_config::schema::MeteringConfig) -> Self {
        info!(
            "Cost meter initialized: enabled={}, metrics={:?}",
            config.enabled, config.metrics
        );
        Self {
            total_requests: AtomicU64::new(0),
            total_prompt_tokens: AtomicU64::new(0),
            total_completion_tokens: AtomicU64::new(0),
            provider_requests: RwLock::new(HashMap::new()),
            enabled: config.enabled,
        }
    }

    /// Record a completed request
    pub async fn record_request(&self, provider: &str, usage: &TokenUsage) {
        if !self.enabled {
            return;
        }

        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_prompt_tokens
            .fetch_add(usage.prompt_tokens as u64, Ordering::Relaxed);
        self.total_completion_tokens
            .fetch_add(usage.completion_tokens as u64, Ordering::Relaxed);

        let mut provider_requests = self.provider_requests.write().await;
        *provider_requests.entry(provider.to_string()).or_insert(0) += 1;

        debug!(
            "Recorded request for provider '{}': {} tokens",
            provider, usage.total_tokens
        );
    }

    /// Get current metering statistics
    pub async fn stats(&self) -> MeteringStats {
        let provider_requests = self.provider_requests.read().await.clone();
        let prompt = self.total_prompt_tokens.load(Ordering::Relaxed);
        let completion = self.total_completion_tokens.load(Ordering::Relaxed);

        MeteringStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_prompt_tokens: prompt,
            total_completion_tokens: completion,
            total_tokens: prompt + completion,
            provider_requests,
        }
    }

    /// Check if metering is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
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
}
