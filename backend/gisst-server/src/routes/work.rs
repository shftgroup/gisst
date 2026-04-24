use crate::utils::uuid_or_empty_string;
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
use gisst::models::Work;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/{id}", get(get_single_work))
        .route("/create", post(create_or_derive_work))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
}

async fn get_single_work(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Work>, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Work::get_by_id(&mut conn, id).await?.unwrap()))
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, serde::Deserialize)]
struct CreateWork {
    #[serde(deserialize_with = "uuid_or_empty_string")]
    work_id: Option<Uuid>,
    work_name: String,
    work_platform: String,
    work_version: String,
    #[serde(deserialize_with = "uuid_or_empty_string")]
    work_derived_from: Option<Uuid>,
}
#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn create_or_derive_work(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
    Json(work): Json<CreateWork>,
) -> Result<Json<Work>, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let creator_id = auth
        .user
        .ok_or(ServerError::AuthUserNotAuthenticated)?
        .creator_id;

    let mut conn = app_state.pool.acquire().await?;

    tracing::info!("Inserting work {work:?}");
    if let Some(id) = work.work_id
        && let Some(existing) = Work::get_by_id(&mut conn, id).await?
    {
        // if this work differs from existing, derive a new work
        if work.work_name == existing.work_name
            && work.work_version == existing.work_version
            && work.work_platform == existing.work_platform
        {
            Ok(Json(existing))
        } else {
            Ok(Json(
                Work::insert(
                    &mut conn,
                    Work {
                        work_id: Uuid::new_v4(),
                        work_name: work.work_name.clone(),
                        work_version: work.work_version.clone(),
                        work_platform: work.work_platform.clone(),
                        work_derived_from: Some(existing.work_id),
                        creator_id:Some(creator_id),
                        created_on: chrono::Utc::now(),
                    },
                )
                .await?,
            ))
        }
    } else if let Some(existing) = Work::get_by_metadata(
        &mut conn,
        &work.work_name,
        &work.work_version,
        &work.work_platform,
    )
    .await?
    {
        // we had no ID to start, so use the existing work
        Ok(Json(existing))
    } else {
        // insert a new work
        Ok(Json(
            Work::insert(
                &mut conn,
                Work {
                    work_id: Uuid::new_v4(),
                    work_name: work.work_name.clone(),
                    work_version: work.work_version.clone(),
                    work_platform: work.work_platform.clone(),
                    work_derived_from: work.work_derived_from,
                        creator_id:Some(creator_id),
                    created_on: chrono::Utc::now(),
                },
            )
            .await?,
        ))
    }
}
