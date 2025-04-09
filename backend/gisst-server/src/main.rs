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
use tracing_subscriber::{EnvFilter, fmt};

// Tracing dependencies
use opentelemetry::{
    global::{self},
    trace::TracerProvider as _,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider};
use tracing_subscriber::layer::SubscriberExt;

fn init_tracer(config: &ServerConfig) -> Result<SdkTracerProvider, Box<dyn std::error::Error>> {
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(config.env.jaeger_endpoint.clone())
        .with_timeout(std::time::Duration::from_millis(200))
        .build()?;
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_simple_exporter(otlp_exporter)
        .build();
    Ok(tracer_provider)
}

#[tokio::main]
#[tracing::instrument(name = "main")]
async fn main() -> Result<()> {
    let config = serverconfig::ServerConfig::new()?;

    // Setup the tracer and logging
    global::set_text_map_propagator(TraceContextPropagator::new());
    let filter = EnvFilter::builder()
        .with_default_directive("warn".parse()?)
        .parse(config.env.rust_log.clone())?; // Log levels taken from ../../config/default.toml
    let subscriber = tracing_subscriber::Registry::default().with(filter);
    if config.env.jaeger_endpoint.is_empty() {
        let subscriber = subscriber.with(fmt::Layer::default());
        tracing::subscriber::set_global_default(subscriber).unwrap();
    } else {
        let tracer = init_tracer(&config).unwrap();
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer.tracer("gisst-server"));
        let subscriber = subscriber.with(telemetry).with(fmt::Layer::default());
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let path = std::env::current_dir()?;
    tracing::info!("The current directory is {}", path.display());
    tracing::debug!("{:?}", &config);
    server::launch(&config).await?;
    Ok(())
}
