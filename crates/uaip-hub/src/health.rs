//! Advanced health check with dependency verification
//!
//! Provides comprehensive health status for the UAIP Hub and its dependencies

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual dependency health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub name: String,
    pub status: HealthStatus,
    pub response_time_ms: Option<f64>,
    pub message: Option<String>,
}

/// Complete health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: String,
    pub dependencies: Vec<DependencyHealth>,
}

/// Cached health check result
#[derive(Debug, Clone)]
struct CachedHealth {
    result: HealthCheckResponse,
    cached_at: Instant,
}

/// Health checker service with caching and circuit breaker
pub struct HealthChecker {
    start_time: Instant,
    db_pool: Option<sqlx::PgPool>,
    redis_client: Option<redis::Client>,
    nats_client: Option<async_nats::Client>,
    cache: Arc<Mutex<Option<CachedHealth>>>,
    cache_ttl: Duration,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            db_pool: None,
            redis_client: None,
            nats_client: None,
            cache: Arc::new(Mutex::new(None)),
            cache_ttl: Duration::from_secs(5), // 5 second cache TTL
        }
    }

    pub fn with_db(mut self, pool: sqlx::PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    pub fn with_redis(mut self, client: redis::Client) -> Self {
        self.redis_client = Some(client);
        self
    }

    pub fn with_nats(mut self, client: async_nats::Client) -> Self {
        self.nats_client = Some(client);
        self
    }

    /// Perform complete health check with caching
    pub async fn check_health(&self) -> HealthCheckResponse {
        // Check if we have a valid cached result
        if let Ok(cache_guard) = self.cache.lock() {
            if let Some(cached) = cache_guard.as_ref() {
                if cached.cached_at.elapsed() < self.cache_ttl {
                    tracing::debug!("Returning cached health check result");
                    return cached.result.clone();
                }
            }
        }

        // Perform actual health checks
        let mut dependencies = Vec::new();

        // Check PostgreSQL
        dependencies.push(self.check_postgres().await);

        // Check Redis
        dependencies.push(self.check_redis().await);

        // Check NATS
        dependencies.push(self.check_nats().await);

        // Determine overall status
        let overall_status = self.determine_overall_status(&dependencies);

        let result = HealthCheckResponse {
            status: overall_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            dependencies,
        };

        // Cache the result
        if let Ok(mut cache_guard) = self.cache.lock() {
            *cache_guard = Some(CachedHealth {
                result: result.clone(),
                cached_at: Instant::now(),
            });
        }

        result
    }

    /// Check PostgreSQL health with timeout
    async fn check_postgres(&self) -> DependencyHealth {
        let start = Instant::now();

        let (status, message) = match &self.db_pool {
            Some(pool) => {
                // Try to execute a simple query with timeout (circuit breaker pattern)
                let timeout_duration = Duration::from_secs(5);
                match tokio::time::timeout(timeout_duration, sqlx::query("SELECT 1").execute(pool))
                    .await
                {
                    Ok(Ok(_)) => {
                        // Also check pool statistics
                        let pool_size = pool.size();
                        let idle_connections = pool.num_idle();

                        (
                            HealthStatus::Healthy,
                            Some(format!(
                                "Pool size: {}, Idle: {}",
                                pool_size, idle_connections
                            )),
                        )
                    }
                    Ok(Err(e)) => (
                        HealthStatus::Unhealthy,
                        Some(format!("Database query failed: {}", e)),
                    ),
                    Err(_) => (
                        HealthStatus::Unhealthy,
                        Some(format!(
                            "Database query timeout (>{}s)",
                            timeout_duration.as_secs()
                        )),
                    ),
                }
            }
            None => (
                HealthStatus::Degraded,
                Some("PostgreSQL connection not configured".to_string()),
            ),
        };

        DependencyHealth {
            name: "PostgreSQL".to_string(),
            status,
            response_time_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message,
        }
    }

    /// Check Redis health with timeout
    async fn check_redis(&self) -> DependencyHealth {
        let start = Instant::now();

        let (status, message) = match &self.redis_client {
            Some(client) => {
                let timeout_duration = Duration::from_secs(3);

                // Try to get a connection and execute PING with timeout
                let check_future = async {
                    let mut conn = client.get_async_connection().await?;
                    redis::cmd("PING").query_async::<_, String>(&mut conn).await
                };

                match tokio::time::timeout(timeout_duration, check_future).await {
                    Ok(Ok(response)) => {
                        if response == "PONG" {
                            (HealthStatus::Healthy, Some("PONG received".to_string()))
                        } else {
                            (
                                HealthStatus::Degraded,
                                Some(format!("Unexpected response: {}", response)),
                            )
                        }
                    }
                    Ok(Err(e)) => (
                        HealthStatus::Unhealthy,
                        Some(format!("Redis check failed: {}", e)),
                    ),
                    Err(_) => (
                        HealthStatus::Unhealthy,
                        Some(format!(
                            "Redis check timeout (>{}s)",
                            timeout_duration.as_secs()
                        )),
                    ),
                }
            }
            None => (
                HealthStatus::Degraded,
                Some("Redis connection not configured - caching disabled".to_string()),
            ),
        };

        DependencyHealth {
            name: "Redis".to_string(),
            status,
            response_time_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message,
        }
    }

    /// Check NATS health
    async fn check_nats(&self) -> DependencyHealth {
        let start = Instant::now();

        let (status, message) = match &self.nats_client {
            Some(client) => {
                // Check connection state
                if client.connection_state() == async_nats::connection::State::Connected {
                    // Get server info for additional details
                    let server_info = client.server_info();
                    (
                        HealthStatus::Healthy,
                        Some(format!(
                            "Connected to NATS server version {}",
                            server_info.version
                        )),
                    )
                } else {
                    (
                        HealthStatus::Unhealthy,
                        Some(format!(
                            "NATS connection state: {:?}",
                            client.connection_state()
                        )),
                    )
                }
            }
            None => (
                HealthStatus::Degraded,
                Some("NATS connection not configured - messaging disabled".to_string()),
            ),
        };

        DependencyHealth {
            name: "NATS".to_string(),
            status,
            response_time_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message,
        }
    }

    /// Determine overall status from dependencies
    fn determine_overall_status(&self, dependencies: &[DependencyHealth]) -> HealthStatus {
        let has_unhealthy = dependencies
            .iter()
            .any(|d| d.status == HealthStatus::Unhealthy);
        let has_degraded = dependencies
            .iter()
            .any(|d| d.status == HealthStatus::Degraded);

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check handler
pub async fn health_check_handler(
    checker: &HealthChecker,
) -> (StatusCode, Json<HealthCheckResponse>) {
    let health = checker.check_health().await;

    let status_code = match health.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(health))
}

/// Liveness probe - simple check that service is running
pub async fn liveness_probe() -> StatusCode {
    StatusCode::OK
}

/// Readiness probe - check if service is ready to accept traffic
pub async fn readiness_probe(checker: &HealthChecker) -> StatusCode {
    let health = checker.check_health().await;

    match health.status {
        HealthStatus::Healthy | HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_checker() {
        let checker = HealthChecker::new();
        let health = checker.check_health().await;

        assert!(!health.version.is_empty());
        assert!(!health.timestamp.is_empty());
        assert!(!health.dependencies.is_empty());
    }

    #[tokio::test]
    async fn test_determine_overall_status() {
        let checker = HealthChecker::new();

        // All healthy
        let deps = vec![DependencyHealth {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            response_time_ms: Some(1.0),
            message: None,
        }];
        assert_eq!(
            checker.determine_overall_status(&deps),
            HealthStatus::Healthy
        );

        // One degraded
        let deps = vec![
            DependencyHealth {
                name: "test1".to_string(),
                status: HealthStatus::Healthy,
                response_time_ms: Some(1.0),
                message: None,
            },
            DependencyHealth {
                name: "test2".to_string(),
                status: HealthStatus::Degraded,
                response_time_ms: Some(1.0),
                message: Some("degraded".to_string()),
            },
        ];
        assert_eq!(
            checker.determine_overall_status(&deps),
            HealthStatus::Degraded
        );

        // One unhealthy
        let deps = vec![
            DependencyHealth {
                name: "test1".to_string(),
                status: HealthStatus::Healthy,
                response_time_ms: Some(1.0),
                message: None,
            },
            DependencyHealth {
                name: "test2".to_string(),
                status: HealthStatus::Unhealthy,
                response_time_ms: Some(1.0),
                message: Some("unhealthy".to_string()),
            },
        ];
        assert_eq!(
            checker.determine_overall_status(&deps),
            HealthStatus::Unhealthy
        );
    }

    #[tokio::test]
    async fn test_liveness_probe() {
        let status = liveness_probe().await;
        assert_eq!(status, StatusCode::OK);
    }
}
