use axum::{
    routing::{get, post},
    http::{StatusCode, Request, Response, Uri},
    response::{IntoResponse, Html},
    body::{boxed, Body},
    Json,
    Router,
    extract::Path,
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
use minijinja::render;



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
async fn main() -> Result<(), sqlx::Error>{

    // Parse CLI commands
    let opt = Opt::parse();

    let app: Router;

    // Setup logging and RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }

    // Configure Ip and Port
    let addr = SocketAddr::from(
        (IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
         opt.port,
        ));

    // enable console logging
    tracing_subscriber::fmt::init();

    // Run headless
    if opt.no_app {
        // Nothing implemented right now
    } else { // Run with web app
        app = Router::new()
            .route("/artifact/:artifact_id", get(get_artifact))
            .nest_service("/", ServeDir::new("../frontend-web/dist"))
    }


    tracing::debug!("listening on {}", ip_addr);

    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();

    log::info!("listening on http://{}", ip_addr);




    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://eric.kaltman@localhost/testdb")
        .await?;

    Ok(())
}


async fn get_artifact(Path(artifact_id):Path<String>) -> Html<String> {
    // Just for testing right now, in future will be a call to the database
    let core: &'static str = "v86";
    let r = render!(INDEX_TEMPLATE, content => Content { id: artifact_id, core: core.to_string() } );
    Html(r)
}


// Database structs, templates, and table generation

#[derive(Debug, Serialize)]
struct Content {
    id: i32,
    uuid: uuid::Uuid,
    title: String,
    version: String,
    path: std::path::Path,
    filename: String,
    core: String,
    core_version: String,
    image_id: uuid::Uuid,
    save_id: uuid::Uuid,
}

const CONTENT_CREATE_TABLE: &'static str = r#"
CREATE TABLE content (
    id          serial PRIMARY KEY,
    uuid        uuid,
    title       varchar(100),
    version

)
"#;

#[derive(Debug, Serialize)]
struct ContentAddition{
    id: i32,
    uuid: uuid::Uuid,
    content_id: i32,
    filename: String,
    path: std::path::Path,
}

#[derive(Debug, Serialize)]
struct Save{
    id: i32,
    uuid: uuid::Uuid,
    short_description: String,
    description: String,
    filename: String,
    path: std::path::Path,
}

#[derive(Debug, Serialize)]
struct Replay{
    id: i32,
    uuid: uuid::Uuid,
    content_id: i32,
    save_id: i32,
    filename: String,
    path: std::path::Path,
}

#[derive(Debug, Serialize)]
struct State{
    id: i32,
    uuid: uuid::Uuid,
    replay_id: i32,
    content_id: i32,
    replay_time_index: u16,
    is_checkpoint: bool,
    path: std::path::Path,
    filename: String,
    name: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct Image{
    id: i32,
    uuid: uuid::Uuid,
    filename: String,
    parent_id: i32,
}

const INDEX_TEMPLATE: &'static str = r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>GISST Player</title>
    <title>Embedulator</title>
    <script type="module" crossorigin src="/assets/index.js"></script>
    <link rel="stylesheet" href="/assets/index.css">
    {% if content.core == "ra" %}

    {% elif content.core == "v86" %}
    <script src="v86/libv86.js"></script>
    {% endif %}
</head>

<body data-artifact-id={{ content.id }}>
    <div id="ui"></div>
</body>
</html>
"#;