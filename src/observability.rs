use opentelemetry::KeyValue;
use opentelemetry_otlp::{TonicExporterBuilder, WithExportConfig};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tokio::time::Duration;
use tonic::metadata::MetadataMap;
use tracing_subscriber::{prelude::*, EnvFilter};

use crate::settings::SETTINGS;

fn new_exporter() -> TonicExporterBuilder {
    let mut map = MetadataMap::with_capacity(1);
    if let Some(auth) = &SETTINGS.otlp_auth {
        map.insert("authorization", auth.parse().unwrap());
    }
    opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(SETTINGS.otlp_collector.clone().unwrap())
        .with_timeout(Duration::from_secs(3))
        .with_metadata(map.clone())
}

fn init_otlp_tracing() {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(new_exporter())
        .with_trace_config(
            sdktrace::config()
                .with_resource(Resource::new(vec![KeyValue::new(SERVICE_NAME, "rumsim")])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to initialize tracer.");

    let layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

fn init_stdout_tracing() {
    let layer = tracing_subscriber::fmt::layer();
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

pub fn init_tracing() {
    if SETTINGS.otlp_collector != None {
        init_otlp_tracing();
    } else {
        init_stdout_tracing();
    };
}
