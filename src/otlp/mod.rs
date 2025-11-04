use std::str::FromStr;

use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{ExporterBuildError, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    trace::{self, RandomIdGenerator, Sampler, SdkTracerProvider},
};
use smallvec::SmallVec;
use tonic::metadata::{MetadataMap, MetadataValue};
use tracing_error::ErrorLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
const SERVICE_NAME: &str = "realworld-axum-api";

pub fn init_tracing(logger_level: &str, endpoint: &str) {
    let logger_provider = init_logger_provider(endpoint).unwrap();
    let tracer = init_tracer(endpoint, Some(0.5)).unwrap();

    let filter = build_env_filter(logger_level, None);
    let otel_filter = build_env_filter(
        logger_level,
        Some(if logger_level == "debug" {
            "debug"
        } else {
            "error"
        }),
    );
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .pretty();
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider).with_filter(otel_filter);
    let telemetry_layer = OpenTelemetryLayer::new(tracer);
    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .with(telemetry_layer);
    if logger_level == "debug" {
        registry.with(ErrorLayer::default()).init();
    } else {
        registry.init();
    }
}

fn get_resource() -> Resource {
    Resource::builder()
        // .with_attribute(KeyValue::new(
        //     "service.name",
        //     "opentelemetry-tracing-service",
        // ))
        .with_service_name(SERVICE_NAME)
        .build()
}

fn get_metadata() -> MetadataMap {
    let mut metadata = MetadataMap::new();
    metadata.insert(
        "authorization",
        MetadataValue::from_str("Basic cm9vdEBleGFtcGxlLmNvbTpGR2k3M21PbE55YmpJdzRT").unwrap(),
    );
    metadata.insert("organization", MetadataValue::from_str("default").unwrap());
    metadata.insert("stream-name", MetadataValue::from_str("default").unwrap());
    metadata
}

fn init_tracer(
    endpoint: &str,
    sample_ratio: Option<f64>,
) -> Result<trace::Tracer, ExporterBuildError> {
    let sample_ratio = sample_ratio.unwrap_or(1.0);
    let sampler = if sample_ratio > 0.0 && sample_ratio < 1.0 {
        Sampler::TraceIdRatioBased(sample_ratio)
    } else {
        Sampler::AlwaysOn
    };

    let exporter = opentelemetry_otlp::SpanExporterBuilder::new()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_metadata(get_metadata())
        .build()
        .unwrap();

    let tracer_provider = SdkTracerProvider::builder()
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(tracer_provider.clone());
    Ok(tracer_provider.tracer(SERVICE_NAME))
}

fn init_logger_provider(endpoint: &str) -> Result<SdkLoggerProvider, ExporterBuildError> {
    let exporter = opentelemetry_otlp::LogExporterBuilder::new()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_metadata(get_metadata())
        .build()?;
    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build();
    Ok(logger_provider)
}

fn build_env_filter(logger_level: &str, default_level: Option<&str>) -> EnvFilter {
    let level = default_level.unwrap_or(logger_level);
    let mut filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    if !matches!(logger_level, "trace" | "debug") {
        let directives: SmallVec<[&str; 5]> =
            smallvec::smallvec!["hyper", "tonic", "h2", "reqwest", "tower"];
        for directive in directives {
            filter = filter.add_directive(format!("{}=off", directive).parse().unwrap());
        }
    }
    filter
}
