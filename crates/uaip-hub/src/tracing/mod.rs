// ! Distributed Tracing Module
//!
//! Provides OpenTelemetry integration with Jaeger for distributed tracing.

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

pub mod jaeger;

/// Initialize distributed tracing with Jaeger
pub fn init_tracer(service_name: &str, jaeger_endpoint: Option<&str>) -> anyhow::Result<()> {
    // Set up OpenTelemetry trace context propagation
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Create Jaeger tracer
    let tracer = if let Some(endpoint) = jaeger_endpoint {
        jaeger::create_jaeger_tracer(service_name, endpoint)?
    } else {
        // Default to stdout exporter for development
        jaeger::create_stdout_tracer(service_name)?
    };

    // Create tracing subscriber with OpenTelemetry layer
    let telemetry_layer = OpenTelemetryLayer::new(tracer);

    // Create environment filter for log levels
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,uaip_hub=debug,tower_http=debug"));

    // Build subscriber with multiple layers
    Registry::default()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .with(telemetry_layer)
        .try_init()?;

    tracing::info!("Distributed tracing initialized with service: {}", service_name);
    Ok(())
}

/// Shutdown tracing and flush remaining spans
pub fn shutdown_tracer() {
    global::shutdown_tracer_provider();
    tracing::info!("Tracing shutdown complete");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_init() {
        // Test that tracer initialization doesn't panic
        let result = init_tracer("test-service", None);
        assert!(result.is_ok());
    }
}
