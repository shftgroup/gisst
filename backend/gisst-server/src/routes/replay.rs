use crate::auth::{self,AuthBackend,User};
use crate::{error::ServerError, server::ServerState};
use axum::{
    Extension, Router,
    extract::{Json, Path},
    routing::{get, post},
    response::NoContent
};
use axum_login::login_required;
use gisst::error::Table;
use gisst::models::{File, Replay};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{HideShowParams,LoggedInUserInfo};

pub fn router() -> Router {
    Router::new()
        .route("/{id}", get(get_single_replay))
        .route("/{id}/hide", post(hideshow_replay))
        .route("/create", post(create_replay))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
}
async fn get_single_replay(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Replay>, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Replay::get_by_id(&mut conn, id).await?.unwrap()))
}



#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn hideshow_replay(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<crate::auth::AuthBackend>,
    Path(id): Path<Uuid>,
    Json(hidden): Json<HideShowParams>,
) -> Result<NoContent, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let user = auth.user.as_ref().map(LoggedInUserInfo::generate_from_user).ok_or(ServerError::AuthUserNotAuthenticated)?;
    let mut conn = app_state.pool.acquire().await?;
    let replay = Replay::get_by_id(conn.as_mut(), id).await?.ok_or(ServerError::RecordMissing{table:Table::Replay,uuid:id})?;
    if user.creator_id == replay.creator_id || user.role <= User::ROLE_ADMIN {
        Replay::set_hidden(conn.as_mut(), id, hidden.state, &app_state.indexer).await?;
        Ok(NoContent)
    } else {
        Err(ServerError::PermissionDenied)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReplay {
    pub replay_name: String,
    pub replay_description: String,
    pub instance_id: Uuid,
    pub replay_forked_from: Option<Uuid>,
    pub file_id: Uuid,
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn create_replay(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
    Json(replay): Json<CreateReplay>,
) -> Result<Json<Replay>, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let mut conn = app_state.pool.acquire().await?;
    if File::get_by_id(&mut conn, replay.file_id).await?.is_some() {
        Ok(Json(
            Replay::insert(
                &mut conn,
                Replay {
                    replay_id: Uuid::new_v4(),
                    replay_name: replay.replay_name,
                    replay_description: replay.replay_description,
                    instance_id: replay.instance_id,
                    creator_id: auth
                        .user
                        .ok_or(ServerError::AuthUserNotAuthenticated)?
                        .creator_id,
                    replay_forked_from: replay.replay_forked_from,
                    file_id: replay.file_id,
                    created_on: chrono::Utc::now(),
                    hidden: false
                },
                &app_state.indexer,
            )
            .await?,
        ))
    } else {
        Err(ServerError::RecordMissing {
            table: Table::File,
            uuid: replay.file_id,
        })?
    }
}
