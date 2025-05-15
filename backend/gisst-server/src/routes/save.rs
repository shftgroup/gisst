use crate::auth::AuthBackend;
use crate::{auth, error::ServerError, server::ServerState};
use axum::{
    Extension, Router,
    extract::{Json, Path},
    routing::{get, post},
};
use axum_login::login_required;
use gisst::error::Table;
use gisst::models::{File, Save};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/{id}", get(get_single_save))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
        .route("/create", post(create_save))
}

async fn get_single_save(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Save>, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Save::get_by_id(&mut conn, id).await?.unwrap()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSave {
    pub instance_id: Uuid,
    pub file_id: Uuid,
    pub save_short_desc: String,
    pub save_description: String,
    pub state_derived_from: Option<Uuid>,
    pub replay_derived_from: Option<Uuid>,
    pub save_derived_from: Option<Uuid>,
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn create_save(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
    Json(save): Json<CreateSave>,
) -> Result<Json<Save>, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let creator_id = auth
        .user
        .ok_or(ServerError::AuthUserNotAuthenticated)?
        .creator_id;

    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, save.file_id).await?.is_some() {
        tracing::info!("Inserting save {save:?}");
        Ok(Json(
            Save::insert(
                &mut conn,
                Save {
                    save_id: Uuid::new_v4(),
                    instance_id: save.instance_id,
                    file_id: save.file_id,
                    save_short_desc: save.save_short_desc,
                    save_description: save.save_description,
                    creator_id,
                    state_derived_from: save.state_derived_from,
                    save_derived_from: save.save_derived_from,
                    replay_derived_from: save.replay_derived_from,
                    created_on: chrono::Utc::now(),
                },
            )
            .await?,
        ))
    } else {
        Err(ServerError::RecordMissing {
            table: Table::File,
            uuid: save.file_id,
        })?
    }
}
