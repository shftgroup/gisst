use crate::{error::ServerError, server::ServerState};
use axum::{
    Extension, Router,
    extract::{Json, Path},
    routing::get,
};
use gisst::models::Work;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new().route("/{id}", get(get_single_work))
}

async fn get_single_work(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Work>, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Work::get_by_id(&mut conn, id).await?.unwrap()))
}
