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
use minijinja::context;
use uuid::Uuid;

pub fn router() -> Router { Router::new().route("/", get(get_dashboard))}

async fn get_dashboard(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
) -> Result<axum::response::Response, ServerError> {
    let user = auth.user.as_ref().map(LoggedInUserInfo::generate_from_user);
    let (search_url, search_key) = app_state.search.frontend_data();
    let dashboard= app_state.templates.get_template("dashboard.html")?;
    Ok(
        Html(
            dashboard.render(context! {
                base_url => BASE_URL.get(),
                search_url => search_url,
                search_key => search_key,
                user => user,
            })
        ).into_response()
    )
}