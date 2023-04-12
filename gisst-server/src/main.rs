use axum::{
    routing::{get, post},
    http::{StatusCode, Request, Response, Uri},
    response::IntoResponse,
    body::{boxed, Body},
    Json,
    Router,
};
use serde::{Deserialize,Serialize};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use axum::body::BoxBody;
use sqlx::postgres::PgPoolOptions;
use tower::{ServiceExt, ServiceBuilder};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use clap::Parser;
use log;



// Setup command line interface, taken from: https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/
#[derive(Parser, Debug)]
#[clap(name = "gisst-server", about = "Basic GISST Server Implementation")]
struct Opt {
    /// set log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,
    /// set server address
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr:String,

    /// set server port
    #[clap(short = 'p', long = "port", default_value = "3000")]
    port: u16,

    /// do not serve frontend application
    #[arg(short,long)]
    no_app: bool,
}


#[tokio::main]
async fn main() {

    // Parse CLI commands
    let opt = Opt::parse();

    // Setup logging and RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    // enable console logging
    tracing_subscriber::fmt::init();

    // Run
    if opt.no_app {

    } else{
        tokio::join!(
            serve(using_serve_dir(), opt.addr, opt.port),
        );
    }




    //let pool = PgPoolOptions::new()
    //    .max_connections(5)
    //    .connect("postgres://postgres:password@localhost/test")
    //    .await?;
}

fn using_serve_dir() -> Router {
    Router::new().nest_service("/", ServeDir::new("../frontend-web/dist"))
}

async fn serve(app: Router, ip_addr:String, port:u16){
    let addr = SocketAddr::from(
        (IpAddr::from_str(ip_addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        port,
    ));
    tracing::debug!("listening on {}", ip_addr);

    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();

    log::info!("listening on http://{}", ip_addr);
}