use crate::{
    serverconfig::ServerConfig,
    db,
    models::{ContentItem},
};
use anyhow::Result;
use axum::{Router, Server, routing::{get, post}, response::{IntoResponse}, http::{StatusCode}, Extension, ServiceExt};

use sqlx::PgPool;
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tower_http::services::ServeDir;

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

pub async fn launch(config: &ServerConfig) -> Result<()> {
    let pool = db::new_pool(config).await?;
    let app = Router::new()
        .route("/content/:content_id", get(get_content))
        .nest_service("/", ServeDir::new("../frontend-web/dist"))
        .layer(Extension(pool));

    let addr = SocketAddr::new(
        IpAddr::V4(config.http.listen_address),
        config.http.listen.port,
    );

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("could not launch GISST HTTP server on port 3000");

    Ok(())

}



async fn get_content(db: Extension<PgPool>, Path(content_id): Path<i32>) -> Json<ContentItem> {
    let mut conn = db.acquire().await?;
    let content = ContentItem::get_by_id(&mut conn, content_id).await?;
    match content {
        Some(x) => Json(x),
        None => Json(ContentItem::empty())
    }
}
