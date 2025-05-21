use super::LoggedInUserInfo;
use crate::server::BASE_URL;
use crate::{auth, error::ServerError, server::ServerState, utils::parse_header};
use axum::{
    Extension, Router,
    extract::{Json, Path},
    http::header::HeaderMap,
    response::{Html, IntoResponse},
    routing::get,
};
use gisst::error::Table;
use gisst::models::Creator;
use minijinja::context;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new().route("/{id}", get(get_single_creator))
}

async fn get_single_creator(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
) -> Result<axum::response::Response, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Some(creator) = Creator::get_by_id(&mut conn, id).await? {
        let accept: Option<String> = parse_header(&headers, "Accept");

        let user = auth.user.as_ref().map(LoggedInUserInfo::generate_from_user);

        Ok(
            (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
                let (search_url, search_key) = app_state.search.frontend_data();
                let creator_page = app_state
                    .templates
                    .get_template("creator_all_listing.html")?;
                Html(creator_page.render(context!(
                    base_url => BASE_URL.get(),
                    search_url => search_url,
                    search_key => search_key,
                    creator => creator,
                    user => user,
                ))?)
                .into_response()
            } else if accept
                .as_ref()
                .is_some_and(|hv| hv.contains("application/json"))
            {
                Json(creator).into_response()
            } else {
                Err(ServerError::MimeType)?
            })
            .into_response(),
        )
    } else {
        Err(ServerError::RecordMissing {
            table: Table::Creator,
            uuid: id,
        })
    }
}
