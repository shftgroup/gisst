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
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
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
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = init_tracer().unwrap();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(telemetry).with(fmt::Layer::default());
    tracing::subscriber::set_global_default(subscriber).unwrap();
    
    // let default_tracing_directive = config
    //     .env
    //     .default_directive
    //     .clone()
    //     .parse()
    //     .expect("default tracing directive has to be valid");
    // let filter = EnvFilter::builder()
    //     .with_default_directive(default_tracing_directive)
    //     .parse(config.env.rust_log.clone())?;
    // tracing_subscriber::fmt()
    //     .with_target(false)
    //     .with_env_filter(filter)
    //     .pretty()
    //     .init();

    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let path = std::env::current_dir()?;
    let config = serverconfig::ServerConfig::new()?;

    let default_tracing_directive = config
        .env
        .default_directive
        .clone()
        .parse()
        .expect("default tracing directive has to be valid");
    let _filter = EnvFilter::builder()
        .with_default_directive(default_tracing_directive)
        .parse(config.env.rust_log.clone())?;
    
    tracing::info!("The current directory is {}", path.display());
    tracing::debug!("{:?}", &config);
    tracing::error!("WHoopsie daisy its good");
    server::launch(&config).await?;

    global::shutdown_tracer_provider();
    Ok(())
}
