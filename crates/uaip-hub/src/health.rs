//! Advanced health check with dependency verification
//!
//! Provides comprehensive health status for the UAIP Hub and its dependencies

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::time::Instant;

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

/// Health checker service
pub struct HealthChecker {
    start_time: Instant,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    /// Perform complete health check
    pub async fn check_health(&self) -> HealthCheckResponse {
        let mut dependencies = Vec::new();

        // Check PostgreSQL
        dependencies.push(self.check_postgres().await);

        // Check Redis
        dependencies.push(self.check_redis().await);

        // Check NATS
        dependencies.push(self.check_nats().await);

        // Determine overall status
        let overall_status = self.determine_overall_status(&dependencies);

        HealthCheckResponse {
            status: overall_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            dependencies,
        }
    }

    /// Check PostgreSQL health
    async fn check_postgres(&self) -> DependencyHealth {
        let start = Instant::now();

        // TODO: Actual database connection check
        // For now, simulate the check
        let (status, message) = if std::env::var("DATABASE_URL").is_ok() {
            (HealthStatus::Healthy, None)
        } else {
            (
                HealthStatus::Unhealthy,
                Some("DATABASE_URL not configured".to_string()),
            )
        };

        DependencyHealth {
            name: "PostgreSQL".to_string(),
            status,
            response_time_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message,
        }
    }

    /// Check Redis health
    async fn check_redis(&self) -> DependencyHealth {
        let start = Instant::now();

        // TODO: Actual Redis connection check
        let (status, message) = if std::env::var("REDIS_URL").is_ok() {
            (HealthStatus::Healthy, None)
        } else {
            (
                HealthStatus::Degraded,
                Some("REDIS_URL not configured - caching disabled".to_string()),
            )
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

        // TODO: Actual NATS connection check
        let (status, message) = if std::env::var("NATS_URL").is_ok() {
            (HealthStatus::Healthy, None)
        } else {
            (
                HealthStatus::Unhealthy,
                Some("NATS_URL not configured".to_string()),
            )
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
        assert!(health.uptime_seconds >= 0);
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
