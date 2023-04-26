mod db;
mod serverconfig;

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
use serde::__private::de::TagOrContentField::Content;


#[derive(Debug, thiserror::Error)]
enum GISSTError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
}

impl IntoResponse for GISSTError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            GISSTError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
        };

        let body = Json(json!({"error": message}));

        (status, body).into_response()
    }
}

async fn get_content(db: Extension<PgPool>,
                     content_id: i32) -> Result<(StatusCode, Json<ContentItem>), GISSTError> {

    Ok((StatusCode::OK, ))
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


    tracing::debug!("listening on {}", addr);

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
            assert!(fs::metadata(path.as_path()).unwrap().is_dir());
            path.pop();
        }

    }

    log::info!("listening on http://{}", addr);
    Ok(())
}

async fn get_content(db: Extension<PgPool>, content_id:Path<String>) -> Html<String> {
    let mut conn = db.acquire().await.unwrap();
    // Just getting this to work, will change unwrap to something safer later
    let id: i32 = content_id.rsplit_once("/").unwrap().1.parse::<i32>().unwrap();

    println!("{}", id);
    let content_record: ContentItem = get_content_by_id(&mut conn, id)?;
    let platform_record: Platform = get_platform_by_id(&mut conn, content_record.platform_id.unwrap())?;
    let core_record: Core = get_core_by_id(&mut conn, platform_record.core_id)?;

    // Just for testing right now, in future will be a call to the database
    let r = render!(INDEX_TEMPLATE,
        content => content_record,
        platform => platform_record,
        core => core_record,
    );
    Html(r)
}

async fn get_content_by_id(conn: &mut PgConnection, content_id: i32) -> Option<ContentItem> {
    sqlx::query_as!(
        ContentItem,
        "SELECT * FROM content WHERE content_id = $1",
        content_id
    )
        .fetch_one(conn)
        .await
}

async fn get_platform_by_id(conn: &mut PgConnection, platform_id: i32) -> Platform{
    sqlx::query_as!(
        Platform, "SELECT * FROM platform WHERE platform_id = $1",
        platform_id,
    )
        .fetch_one(conn)
        .await
}

async fn get_core_by_id(conn: &mut PgConnection, core_id: i32) -> Core {
    sqlx::query_as!(
        Core, "SELECT * FROM core WHERE core_id = $1",
        core_id,
    )
        .fetch_one(conn)
        .await?
}

// Database structs, templates, and table generation

#[derive(Debug, Serialize, Deserialize)]
struct ContentItem{
    content_id: i32,
    content_uuid: Option<uuid::Uuid>,
    content_title: Option<String>,
    content_version: Option<String>,
    content_path: Option<String>,
    content_filename: Option<String>,
    platform_id: Option<i32>,
    content_parent_id: Option<i32>,
    created_on: Option<time::PrimitiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Core{
    core_id: i32,
    core_name: String,
    core_version: String,
    core_manifest: serde_json::Value,
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
    platform_id: i32,
    core_id: i32,
    platform_framework: String,
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
    path: String,
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