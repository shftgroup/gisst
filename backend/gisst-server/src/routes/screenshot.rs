use crate::auth::AuthBackend;
use crate::{auth, error::ServerError, server::ServerState};
use axum::{
    Extension, Router,
    extract::{Json, Path},
    routing::{get, post},
};
use axum_login::login_required;
use serde::Deserialize;
use serde_with::{base64::Base64, serde_as};
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/{id}", get(get_single_screenshot))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
        .route("/create", post(create_screenshot))
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct ScreenshotCreateInfo {
    #[serde_as(as = "Base64")]
    screenshot_data: Vec<u8>,
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn create_screenshot(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
    Json(screenshot): Json<ScreenshotCreateInfo>,
) -> Result<Json<gisst::models::Screenshot>, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(
        gisst::models::Screenshot::insert(
            &mut conn,
            gisst::models::Screenshot {
                screenshot_id: Uuid::new_v4(),
                screenshot_data: screenshot.screenshot_data,
            },
        )
        .await?,
    ))
}

async fn get_single_screenshot(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<gisst::models::Screenshot>, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(
        gisst::models::Screenshot::get_by_id(&mut conn, id)
            .await?
            .unwrap(),
    ))
}