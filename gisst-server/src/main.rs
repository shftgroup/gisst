use axum::{
    routing::{get, post},
    http::{StatusCode, Request, Response, Uri},
    response::{IntoResponse, Html},
    body::{boxed, Body},
    Json,
    Router,
    extract::Path,
    Extension,
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
use minijinja::{render, context};
use sqlx::{PgConnection, PgPool, Postgres};
use std::fs::{self, DirBuilder};


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

    // connect to PostgreSQL
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://eric.kaltman@localhost/testdb")
        .await?;

    // Run headless
    if opt.no_app {
        // Nothing implemented right now
    } else { // Run with web app
        app = Router::new()
            .route("/content/:content_id", get(get_content))
            .nest_service("/", ServeDir::new("../frontend-web/dist"))
            .layer(Extension(pool))
    }


    tracing::debug!("listening on {}", ip_addr);

    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();


    // Setup basic directory structure
    let root_application_path = "~/Library/Application Support/Gisst";
    if !fs::metadata(root_application_path).unwrap().is_dir() {
        DirBuilder::new()
            .recursive(true)
            .create(root_application_path).unwrap();

        let mut path = std::path::PathBuf::new();
        path.push(root_application_path);

        let mut folders = vec![
            "state".to_string(),
            "save".to_string(),
            "replay".to_string(),
            "content".to_string()].into_iter();

        for folder_path in folders {
            path.push(folder_path);
            DirBuilder::new()
                .recursive(false)
                .create(path.as_path())
                .unwrap();
            path.pop();
        }

    }
    assert!(fs::metadata(path).unwrap().is_dir());

    log::info!("listening on http://{}", ip_addr);
    Ok(())
}

async fn get_content(db: Extension<PgPool>, content_id:Path<String>) -> Html<String> {
    let mut conn = db.acquire().await?;
    // Just getting this to work, will change unwrap to something safer later
    let id: i32 = content_id.rsplit_once("/").unwrap().1.parse::<i32>().unwrap();

    let content_record: Content = get_content_by_id(&mut conn, id);
    let platform_record: Platform = get_platform_by_id(&mut conn, content_record.platform_id);
    let core_record: Core = get_core_by_id(&mut conn, platform_record.core_id);

    // Just for testing right now, in future will be a call to the database
    let r = render!(INDEX_TEMPLATE, context! {
        content => content_record,
        platform => platform_record,
        core => core_record,
    });
    Html(r)
}

async fn get_content_by_id(conn: &mut PgConnection, content_id: i32) -> Content {
    sqlx::query_as!(
        Content, r"SELECT * FROM content WHERE id = ?",
        content_id
    )
        .fetch_one(conn)
        .await?
}

async fn get_platform_by_id(conn: &mut PgConnection, platform_id: i32) -> Platform{
    sqlx::query_as!(
        Platform, r"SELECT * FROM platform WHERE id = ?",
        platform_id,
    )
        .fetch_one(conn)
        .await?
}

async fn get_core_by_id(conn: &mut PgConnection, core_id: i32) -> Core {
    sqlx::query_as!(
        Core, r"SELECT * FROM core WHERE id = ?",
        core_id,
    )
        .fetch_one(conn)
        .await?
}

// Database structs, templates, and table generation

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    id: i32,
    uuid: uuid::Uuid,
    title: String,
    version: String,
    path: String,
    filename: String,
    platform_id: i32,
    parent_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Core{
    id: i32,
    name: String,
    version: String,
    manifest: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ContentAddition{
    id: i32,
    uuid: uuid::Uuid,
    content_id: i32,
    filename: String,
    path: String,
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
struct Platform{
    id: i32,
    core_id: i32,
    framework: String,
}

#[derive(Debug, Serialize)]
struct State{
    id: i32,
    uuid: uuid::Uuid,
    screenshot: Vec<u8>,
    replay_id: i32,
    content_id: i32,
    replay_time_index: u16,
    is_checkpoint: bool,
    path: std::path::Path,
    filename: String,
    name: String,
    description: String,
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
    {% if content.framework == "ra" %}

    {% elif content.framework == "v86" %}
    <script src="v86/libv86.js"></script>
    {% endif %}
</head>

<body data-artifact-id={{ content.id }}>
    <div id="ui"></div>
</body>
</html>
"#;