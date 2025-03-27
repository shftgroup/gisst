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
use tracing_subscriber::{fmt,EnvFilter};

// Tracing dependencies
use opentelemetry::{global, runtime, KeyValue};
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::{Resource, trace};
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;

fn init_tracer() -> Result<trace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            // Specify which port we want to export to (Jaeger)
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://127.0.0.1:4317/"), // Uses grpc
        )
        .with_trace_config(
            // Specify the name of our server in Jaeger
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "gisst-server"
            )])),
        )
        .install_batch(runtime::Tokio)
}

#[tokio::main]
#[tracing::instrument(name="main")] 
async fn main() -> Result<()> {

    let config = serverconfig::ServerConfig::new()?;

    // Setup the tracer and logging
    global::set_text_map_propagator(TraceContextPropagator::new());
    let filter = EnvFilter::builder()
        .with_default_directive("warn".parse()?)
        .parse(config.env.rust_log.clone())?; // Log levels taken from ../../config/default.toml
    let tracer = init_tracer().unwrap();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default()
        .with(filter)
        .with(telemetry)
        .with(fmt::Layer::default());
    tracing::subscriber::set_global_default(subscriber).unwrap();

    
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();
    
    let path = std::env::current_dir()?;
    tracing::info!("The current directory is {}", path.display());
    tracing::debug!("{:?}", &config);
    server::launch(&config).await?;

    global::shutdown_tracer_provider();
    Ok(())
}
