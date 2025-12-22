//! Rate limiting middleware to prevent abuse
//!
//! Implements token bucket algorithm for rate limiting

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::warn;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window duration
    pub window_duration: Duration,
    /// Burst size (allows temporary spikes)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_duration: Duration::from_secs(60),
            burst_size: 20,
        }
    }
}

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            last_refill: Instant::now(),
            max_tokens,
            refill_rate,
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimitLayer {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    config: RateLimitConfig,
}

impl RateLimitLayer {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if request is allowed for given key (e.g., IP address)
    pub async fn check_rate_limit(&self, key: &str) -> bool {
        let mut buckets = self.buckets.write().await;

        let bucket = buckets.entry(key.to_string()).or_insert_with(|| {
            let refill_rate =
                self.config.max_requests as f64 / self.config.window_duration.as_secs_f64();
            TokenBucket::new(self.config.burst_size as f64, refill_rate)
        });

        bucket.try_consume(1.0)
    }

    /// Clean up old buckets (should be called periodically)
    pub async fn cleanup_old_buckets(&self) {
        let mut buckets = self.buckets.write().await;
        buckets.retain(|_, bucket| {
            let elapsed = bucket.last_refill.elapsed();
            elapsed < self.config.window_duration * 2
        });
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
    limiter: RateLimitLayer,
) -> Response {
    // Extract client identifier (IP address or user ID)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Check rate limit
    if !limiter.check_rate_limit(&client_ip).await {
        warn!(
            client_ip = %client_ip,
            "Rate limit exceeded"
        );

        // Record metrics
        crate::metrics::Metrics::record_http_request(
            request.method().as_str(),
            request.uri().path(),
            429,
            0.0,
        );

        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        )
            .into_response();
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10.0, 1.0); // 10 tokens, 1 per second

        // Should allow 10 requests
        for _ in 0..10 {
            assert!(bucket.try_consume(1.0));
        }

        // 11th request should fail
        assert!(!bucket.try_consume(1.0));

        // Wait and try again
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(bucket.try_consume(1.0));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            max_requests: 5,
            window_duration: Duration::from_secs(1),
            burst_size: 5,
        };

        let limiter = RateLimitLayer::new(config);

        // Should allow 5 requests
        for i in 0..5 {
            assert!(
                limiter.check_rate_limit("test_ip").await,
                "Request {} should be allowed",
                i
            );
        }

        // 6th request should be denied
        assert!(!limiter.check_rate_limit("test_ip").await);

        // Wait and try again
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(limiter.check_rate_limit("test_ip").await);
    }

    #[tokio::test]
    async fn test_cleanup_old_buckets() {
        let config = RateLimitConfig::default();
        let limiter = RateLimitLayer::new(config);

        // Create some buckets
        limiter.check_rate_limit("ip1").await;
        limiter.check_rate_limit("ip2").await;

        // Cleanup should not remove recent buckets
        limiter.cleanup_old_buckets().await;
        assert_eq!(limiter.buckets.read().await.len(), 2);
    }
}
