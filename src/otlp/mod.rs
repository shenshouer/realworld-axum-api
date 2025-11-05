use std::str::FromStr;

use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{ExporterBuildError, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    metrics::SdkMeterProvider,
    trace::{self, RandomIdGenerator, Sampler, SdkTracerProvider},
};
use smallvec::SmallVec;
use tonic::metadata::{MetadataMap, MetadataValue};
use tracing::warn;
use tracing_error::ErrorLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
const SERVICE_NAME: &str = "realworld-axum-api";

pub fn init_tracing(
    logger_level: Option<String>,
    endpoint: Option<String>,
    token: Option<String>,
) -> Result<Option<SdkMeterProvider>, ExporterBuildError> {
    let logger_level = logger_level.unwrap_or("info".to_owned());
    let (endpoint, token) = match (endpoint, token) {
        (Some(endpoint), Some(token)) => (endpoint, token),
        _ => {
            warn!("No endpoint or token provided, tracing will not be enabled");
            return Ok(None);
        }
    };
    let logger_provider = init_logger_provider(&endpoint, &token).unwrap();
    let tracer = init_tracer(&endpoint, &token, Some(0.5)).unwrap();

    let filter = build_env_filter(&logger_level, None);
    let otel_filter = build_env_filter(
        &logger_level,
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

    let meter_provider = init_metrics_provider(&endpoint, &token)?;
    Ok(Some(meter_provider))
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

fn get_metadata(token: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    metadata.insert(
        "authorization",
        MetadataValue::from_str(&format!("Basic {token}")).unwrap(),
    );
    metadata.insert("organization", MetadataValue::from_str("default").unwrap());
    metadata.insert("stream-name", MetadataValue::from_str("default").unwrap());
    metadata
}

fn init_tracer(
    endpoint: &str,
    token: &str,
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
        .with_metadata(get_metadata(token))
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

fn init_logger_provider(
    endpoint: &str,
    token: &str,
) -> Result<SdkLoggerProvider, ExporterBuildError> {
    let exporter = opentelemetry_otlp::LogExporterBuilder::new()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_metadata(get_metadata(token))
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

fn init_metrics_provider(
    endpoint: &str,
    token: &str,
) -> Result<SdkMeterProvider, ExporterBuildError> {
    let exporter = opentelemetry_otlp::MetricExporterBuilder::new()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_metadata(get_metadata(token))
        .build()?;
    let meter_provider = SdkMeterProvider::builder()
        .with_resource(get_resource())
        .with_periodic_exporter(exporter)
        .build();

    global::set_meter_provider(meter_provider.clone());
    Ok(meter_provider)
}
