use crate::auth::AuthBackend;
use crate::{error::ServerError, server::ServerState};
use axum::{
    Extension, Router,
    extract::{Json, Path},
    routing::{get, post},
};
use axum_login::login_required;
use gisst::error::Table;
use gisst::models::{File, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/{id}", get(get_single_state))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
        .route("/create", post(create_state))
}
async fn get_single_state(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<State>, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(State::get_by_id(&mut conn, id).await?.unwrap()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateState {
    pub instance_id: Uuid,
    pub is_checkpoint: bool,
    pub file_id: Uuid,
    pub state_name: String,
    pub state_description: String,
    pub screenshot_id: Uuid,
    pub replay_id: Option<Uuid>,
    pub state_replay_index: Option<i32>,
    pub state_derived_from: Option<Uuid>,
    pub save_derived_from: Option<Uuid>,
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn create_state(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<crate::auth::AuthBackend>,
    Json(state): Json<CreateState>,
) -> Result<Json<State>, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, state.file_id).await?.is_some() {
        tracing::info!("Inserting state {state:?}");
        Ok(Json(
            State::insert(
                &mut conn,
                State {
                    state_id: Uuid::new_v4(),
                    instance_id: state.instance_id,
                    is_checkpoint: state.is_checkpoint,
                    file_id: state.file_id,
                    state_name: state.state_name,
                    state_description: state.state_description,
                    screenshot_id: state.screenshot_id,
                    replay_id: state.replay_id,
                    creator_id: auth
                        .user
                        .ok_or(ServerError::AuthUserNotAuthenticated)?
                        .creator_id,
                    state_replay_index: state.state_replay_index,
                    state_derived_from: state.state_derived_from,
                    created_on: chrono::Utc::now(),
                    save_derived_from: state.save_derived_from,
                },
                &app_state.indexer,
            )
            .await?,
        ))
    } else {
        Err(ServerError::RecordMissing {
            table: Table::File,
            uuid: state.file_id,
        })?
    }
}
