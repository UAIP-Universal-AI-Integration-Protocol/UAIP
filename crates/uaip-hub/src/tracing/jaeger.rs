//! Jaeger tracer configuration

use opentelemetry::trace::Tracer;
use opentelemetry_jaeger::JaegerPropagator;
use opentelemetry_sdk::trace::{Config, Sampler, TracerProvider};
use opentelemetry_sdk::{runtime, Resource};

/// Create a Jaeger tracer with UDP exporter
pub fn create_jaeger_tracer(
    service_name: &str,
    jaeger_endpoint: &str,
) -> anyhow::Result<impl Tracer> {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_endpoint(jaeger_endpoint)
        .with_service_name(service_name)
        .with_auto_split_batch(true)
        .install_batch(runtime::Tokio)?;

    Ok(tracer)
}

/// Create a stdout tracer for development/testing
pub fn create_stdout_tracer(service_name: &str) -> anyhow::Result<impl Tracer> {
    let exporter = opentelemetry_stdout::SpanExporter::default();

    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_config(
            Config::default()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![opentelemetry::KeyValue::new(
                    "service.name",
                    service_name.to_string(),
                )])),
        )
        .build();

    let tracer = provider.tracer(service_name.to_string());
    Ok(tracer)
}

/// Create a custom tracer with specific sampling configuration
pub fn create_custom_tracer(
    service_name: &str,
    jaeger_endpoint: &str,
    sample_ratio: f64,
) -> anyhow::Result<impl Tracer> {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_endpoint(jaeger_endpoint)
        .with_service_name(service_name)
        .with_trace_config(
            Config::default()
                .with_sampler(Sampler::TraceIdRatioBased(sample_ratio))
                .with_resource(Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                    opentelemetry::KeyValue::new("deployment.environment", "production"),
                ])),
        )
        .install_batch(runtime::Tokio)?;

    Ok(tracer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdout_tracer_creation() {
        let result = create_stdout_tracer("test-service");
        assert!(result.is_ok());
    }
}
