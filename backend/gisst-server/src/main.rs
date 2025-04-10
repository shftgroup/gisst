mod auth;
mod db;
mod error;
mod routes;
mod selective_serve_dir;
mod server;
mod serverconfig;
mod tus;
mod utils;

use anyhow::Result;
use serverconfig::ServerConfig;
use tracing_subscriber::EnvFilter;

// Tracing dependencies
use opentelemetry::{
    KeyValue,
    global::{self},
    trace::TracerProvider as _,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use opentelemetry_semantic_conventions::{
    SCHEMA_URL,
    attribute::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, SERVICE_NAMESPACE, SERVICE_VERSION},
};
use tracing_subscriber::layer::SubscriberExt;
fn resource() -> Resource {
    Resource::builder()
        .with_schema_url(
            [
                KeyValue::new(SERVICE_NAME, "server"),
                KeyValue::new(SERVICE_NAMESPACE, "gisst"),
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
            ],
            SCHEMA_URL,
        )
        .with_service_name("server")
        .build()
}

fn init_tracer(config: &ServerConfig) -> Result<SdkTracerProvider, anyhow::Error> {
    Ok(if config.env.jaeger_endpoint.is_empty() {
        SdkTracerProvider::builder()
            .with_batch_exporter(opentelemetry_stdout::SpanExporter::default())
            .with_resource(resource())
            .build()
    } else {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(config.env.jaeger_endpoint.clone())
            .with_timeout(std::time::Duration::from_millis(200))
            .build()?;
        opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(resource())
            .build()
    })
}

#[tokio::main]
#[tracing::instrument(name = "main")]
async fn main() -> Result<()> {
    let config = serverconfig::ServerConfig::new()?;

    // Setup the tracer and logging
    let tracer_provider = init_tracer(&config)?;
    global::set_text_map_propagator(TraceContextPropagator::new());

    let filter = EnvFilter::builder()
        .with_default_directive("warn".parse()?)
        .parse(config.env.rust_log.clone())?; // Log levels taken from ../../config/default.toml
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("gisst/server")));
    tracing::subscriber::set_global_default(subscriber)?;

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let s = tracing::info_span!("initialization");
    tracing::info!("{config:?}");
    drop(s);
    server::launch(&config).await?;
    tracer_provider.shutdown()?;
    Ok(())
}
