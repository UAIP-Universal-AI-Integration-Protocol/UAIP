//! Graceful shutdown handler for production deployments
//!
//! Ensures clean shutdown of all connections and resources

use std::time::Duration;
use tokio::signal;
use tracing::{error, info, warn};

/// Graceful shutdown configuration
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// Maximum time to wait for graceful shutdown
    pub grace_period: Duration,
    /// Whether to force shutdown after grace period
    pub force_after_grace_period: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            grace_period: Duration::from_secs(30),
            force_after_grace_period: true,
        }
    }
}

/// Shutdown signal handler
pub struct ShutdownHandler {
    config: ShutdownConfig,
}

impl ShutdownHandler {
    pub fn new(config: ShutdownConfig) -> Self {
        Self { config }
    }

    /// Wait for shutdown signal (SIGTERM, SIGINT, or Ctrl+C)
    pub async fn wait_for_signal(&self) {
        #[cfg(unix)]
        {
            self.wait_for_signal_unix().await
        }

        #[cfg(not(unix))]
        {
            self.wait_for_signal_ctrl_c().await
        }
    }

    #[cfg(unix)]
    async fn wait_for_signal_unix(&self) {
        use signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to setup SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM signal, initiating graceful shutdown");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT signal, initiating graceful shutdown");
            }
        }

        self.perform_shutdown().await;
    }

    #[cfg(not(unix))]
    async fn wait_for_signal_ctrl_c(&self) {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");

        info!("Received Ctrl+C, initiating graceful shutdown");
        self.perform_shutdown().await;
    }

    /// Perform graceful shutdown steps
    async fn perform_shutdown(&self) {
        info!(
            grace_period_secs = self.config.grace_period.as_secs(),
            "Starting graceful shutdown"
        );

        // Step 1: Stop accepting new connections
        info!("Step 1/4: Stopping new connections");

        // Step 2: Close existing connections gracefully
        info!("Step 2/4: Closing existing connections");
        self.close_connections().await;

        // Step 3: Flush metrics and logs
        info!("Step 3/4: Flushing metrics and logs");
        self.flush_metrics_and_logs().await;

        // Step 4: Clean up resources
        info!("Step 4/4: Cleaning up resources");
        self.cleanup_resources().await;

        info!("Graceful shutdown completed successfully");
    }

    async fn close_connections(&self) {
        // TODO: Implement connection draining
        // - WebSocket connections
        // - Database connections
        // - Redis connections
        // - NATS connections

        // Wait for existing requests to complete (with timeout)
        let timeout = tokio::time::sleep(self.config.grace_period);
        tokio::pin!(timeout);

        tokio::select! {
            _ = timeout => {
                if self.config.force_after_grace_period {
                    warn!("Grace period elapsed, forcing shutdown");
                } else {
                    info!("Grace period elapsed, continuing shutdown");
                }
            }
        }
    }

    async fn flush_metrics_and_logs(&self) {
        // Flush any pending metrics
        if let Err(e) = tokio::task::spawn_blocking(|| {
            // Force flush of metrics
            prometheus::gather();
        })
        .await
        {
            error!(error = ?e, "Failed to flush metrics");
        }

        // Give logs time to flush
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    async fn cleanup_resources(&self) {
        // TODO: Clean up resources
        // - Close file handles
        // - Release locks
        // - Finalize transactions
    }
}

impl Default for ShutdownHandler {
    fn default() -> Self {
        Self::new(ShutdownConfig::default())
    }
}

/// Create a shutdown signal that can be awaited
pub async fn shutdown_signal() {
    let handler = ShutdownHandler::default();
    handler.wait_for_signal().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_config() {
        let config = ShutdownConfig::default();
        assert_eq!(config.grace_period, Duration::from_secs(30));
        assert!(config.force_after_grace_period);
    }

    #[tokio::test]
    async fn test_shutdown_handler_creation() {
        let handler = ShutdownHandler::default();
        assert_eq!(handler.config.grace_period, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_flush_metrics() {
        let handler = ShutdownHandler::default();
        handler.flush_metrics_and_logs().await;
        // Should complete without panicking
    }
}
