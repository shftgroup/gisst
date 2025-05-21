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
use gisst::models::{Creator, CreatorReplayInfo, CreatorSaveInfo, CreatorStateInfo};
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
    saves: Vec<CreatorSaveInfo>,
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
                let creator_filter = format!("creator_id = \"{id}\"");
                let search = app_state.search.states();
                let state_page_num = params.state_page_num.unwrap_or(1);
                let state_limit = params.state_limit.unwrap_or(100).min(100);
                let state_results: meilisearch_sdk::search::SearchResults<CreatorStateInfo> = search
                    .search()
                    .with_hits_per_page(state_limit as usize)
                    .with_page(state_page_num as usize)
                    .with_filter(&creator_filter)
                    .with_sort(&["created_on:desc"])
                    .with_query(&params.state_contains.clone().unwrap_or_default())
                    .execute()
                    .await
                    .map_err(gisst::error::Search::from)?;
                let states: Vec<_> = state_results.hits.into_iter().map(|r| r.result).collect();
                let replay_page_num = params.replay_page_num.unwrap_or(1);
                let replay_limit = params.replay_limit.unwrap_or(100).min(100);
                let search = app_state.search.replays();
                let replay_results: meilisearch_sdk::search::SearchResults<CreatorReplayInfo> = search
                    .search()
                    .with_hits_per_page(replay_limit as usize)
                    .with_page(replay_page_num as usize)
                    .with_filter(&creator_filter)
                    .with_sort(&["created_on:desc"])
                    .with_query(&params.replay_contains.clone().unwrap_or_default())
                    .execute()
                    .await
                    .map_err(gisst::error::Search::from)?;
                let replays: Vec<_> = replay_results.hits.into_iter().map(|r| r.result).collect();
                let save_page_num = params.save_page_num.unwrap_or(1);
                let save_limit = params.save_limit.unwrap_or(100).min(100);
                let search = app_state.search.saves();
                let save_results: meilisearch_sdk::search::SearchResults<CreatorSaveInfo> = search
                    .search()
                    .with_hits_per_page(save_limit as usize)
                    .with_page(save_page_num as usize)
                    .with_filter(&creator_filter)
                    .with_sort(&["created_on:desc"])
                    .with_query(&params.save_contains.clone().unwrap_or_default())
                    .execute()
                    .await
                    .map_err(gisst::error::Search::from)?;
                let saves: Vec<_> = save_results.hits.into_iter().map(|r| r.result).collect();
                let creator_results = GetAllCreatorResult {
                    states,
                    replays,
                    saves,
                    creator,
                };
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
