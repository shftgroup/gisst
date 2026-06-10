use crate::{
    auth::{self, AuthBackend},
    error::ServerError,
    server::ServerState,
};
use axum::{
    Extension, Router,
    extract::{Json, Path},
    routing::{get, post},
};
use axum_login::login_required;
use gisst::{
    error::Table,
    models::{File, Video},
};
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/{id}", get(get_single_video))
        .route("/create", post(create_video))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
}

#[derive(Debug, Deserialize)]
struct VideoCreateInfo {
    file_id: Uuid, // File
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn create_video(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
    Json(video): Json<VideoCreateInfo>,
) -> Result<Json<Video>, ServerError> {
    let user_id = auth.user.as_ref().map(|u| u.creator_id);
    tracing::Span::current().record("userid", user_id.map(|u| u.to_string()));
    let mut conn = app_state.pool.acquire().await?;
    if File::get_by_id(&mut conn, video.file_id).await?.is_some() {
        tracing::info!("Inserting video {video:?}");
        Ok(Json(
            Video::insert(
                &mut conn,
                Video {
                    video_id: Uuid::new_v4(),
                    file_id: video.file_id,
                    creator_id: user_id.ok_or(ServerError::AuthUserNotAuthenticated)?,
                    created_on: chrono::Utc::now(),
                },
            )
            .await?,
        ))
    } else {
        Err(ServerError::RecordMissing {
            table: Table::File,
            uuid: video.file_id,
        })
    }
}

async fn get_single_video(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Video>, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Video::get_by_id(&mut conn, id).await?.unwrap()))
}
