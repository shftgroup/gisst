mod db;
mod serverconfig;
mod models;
mod server;
mod storage;

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
use crate::models::ContentItem;

use anyhow::Result;
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    let config = serverconfig::ServerConfig::new()?;
    debug!(?config);
    server::launch(&config).await?;
    Ok(())
}

    // Setup basic directory structure
    let root_application_path: &str = "~/Library/Application Support/Gisst";
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



// Database structs, templates, and table generation




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