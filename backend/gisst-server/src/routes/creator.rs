use super::{LoggedInUserInfo, StateReplayPageQueryParams};
use crate::server::BASE_URL;
use crate::{auth, error::ServerError, server::ServerState, utils::parse_header};
use axum::{
    Extension, Router,
    extract::{Json, Path, Query},
    http::header::HeaderMap,
    response::{Html, IntoResponse},
    routing::get,
};
use gisst::error::Table;
use gisst::models::{Creator, CreatorReplayInfo, CreatorStateInfo};
use minijinja::context;
use serde::Serialize;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new().route("/{id}", get(get_single_creator))
}

#[derive(Serialize)]
struct GetAllCreatorResult {
    states: Vec<CreatorStateInfo>,
    replays: Vec<CreatorReplayInfo>,
    creator: Creator,
}

async fn get_single_creator(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    params: Query<StateReplayPageQueryParams>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
) -> Result<axum::response::Response, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Some(creator) = Creator::get_by_id(&mut conn, id).await? {
        let state_page_num = params.state_page_num.unwrap_or(0);
        let state_limit = params.state_limit.unwrap_or(100).min(100);
        let state_offset = state_page_num * state_limit;
        let replay_page_num = params.replay_page_num.unwrap_or(0);
        let replay_limit = params.replay_limit.unwrap_or(100).min(100);
        let replay_offset = replay_page_num * replay_limit;
        let creator_results = GetAllCreatorResult {
            states: Creator::get_all_state_info(
                &mut conn,
                creator.creator_id,
                params.state_contains.clone(),
                state_offset,
                state_limit,
            )
            .await?,
            replays: Creator::get_all_replay_info(
                &mut conn,
                creator.creator_id,
                params.replay_contains.clone(),
                replay_offset,
                replay_limit,
            )
            .await?,
            creator,
        };
        let state_has_more = creator_results.states.len() >= state_limit as usize;
        let replay_has_more = creator_results.replays.len() >= replay_limit as usize;

        let accept: Option<String> = parse_header(&headers, "Accept");

        let user = auth.user.as_ref().map(LoggedInUserInfo::generate_from_user);

        Ok(
            (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
                let creator_page = app_state
                    .templates
                    .get_template("creator_all_listing.html")?;
                Html(creator_page.render(context!(
                    base_url => BASE_URL.get(),
                    state_has_more => state_has_more,
                    replay_has_more => replay_has_more,
                    state_page_num => state_page_num,
                    state_limit => state_limit,
                    state_contains => params.state_contains,
                    replay_page_num => replay_page_num,
                    replay_limit => replay_limit,
                    replay_contains => params.replay_contains,
                    creator => creator_results,
                    user => user,
                ))?)
                .into_response()
            } else if accept
                .as_ref()
                .is_some_and(|hv| hv.contains("application/json"))
            {
                Json(creator_results).into_response()
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
