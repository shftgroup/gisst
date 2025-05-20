use super::{LoggedInUserInfo, StateReplayPageQueryParams};
use crate::auth::AuthBackend;
use crate::server::BASE_URL;
use crate::{auth, error::ServerError, server::ServerState, utils::parse_header};
use axum::{
    Extension, Router,
    extract::{Json, Path, Query},
    http::header::HeaderMap,
    response::{Html, IntoResponse},
    routing::get,
};
use axum_login::login_required;
use gisst::error::Table;
use gisst::models::{Environment, Instance, InstanceWork, ObjectLink, Replay, Save, State, Work};
use minijinja::context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_instances))
        .route("/{id}", get(get_all_for_instance))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
        .route("/{id}/clone", get(clone_v86_instance))
}

#[derive(Deserialize, Debug)]
struct InstanceListQueryParams {
    page_num: Option<u32>,
    limit: Option<u32>,
    contains: Option<String>,
    platform: Option<String>,
}
#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn get_instances(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Query(params): Query<InstanceListQueryParams>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
) -> Result<axum::response::Response, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let page_num = params.page_num.unwrap_or(1);
    let limit = params.limit.unwrap_or(100).min(100);
    let accept: Option<String> = parse_header(&headers, "Accept");
    let user = auth.user.as_ref().map(LoggedInUserInfo::generate_from_user);
    let search = app_state.search.instances();
    let results: meilisearch_sdk::search::SearchResults<InstanceWork> = search
        .search()
        .with_facets(meilisearch_sdk::search::Selectors::Some(&["work_platform"]))
        .with_hits_per_page(limit as usize)
        .with_page(page_num as usize)
        .with_filter(&params.platform.as_ref().map(|p| format!("work_platform = \"{p}\"")).unwrap_or_default())
        .with_query(&params.contains.clone().unwrap_or_default())
        .execute()
        .await
        .map_err(gisst::error::Search::from)?;
    let instances: Vec<_> = results.hits.into_iter().map(|r| r.result).collect();
    Ok(
        (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
            let instance_listing = app_state.templates.get_template("instance_listing.html")?;
            Html(instance_listing.render(context!(
                base_url => BASE_URL.get(),
                has_more => instances.len() >= limit as usize,
                instances => instances,
                user => user,
                page_num => page_num,
                limit => limit,
                contains => params.contains,
                platform => params.platform,
            ))?)
            .into_response()
        } else if accept
            .as_ref()
            .is_some_and(|hv| hv.contains("application/json"))
        {
            Json(instances).into_response()
        } else {
            Err(ServerError::MimeType)?
        })
        .into_response(),
    )
}

#[derive(Debug, Serialize)]
struct FullInstance {
    info: Instance,
    work: Work,
    environment: Environment,
    states: Vec<State>,
    replays: Vec<Replay>,
    saves: Vec<Save>,
    objects: Vec<ObjectLink>,
}

#[tracing::instrument("instance-page", skip(app_state, auth), fields(userid))]
async fn get_all_for_instance(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    params: Query<StateReplayPageQueryParams>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
) -> Result<axum::response::Response, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let mut conn = app_state.pool.acquire().await?;
    if let Some(instance) = Instance::get_by_id(&mut conn, id).await? {
        // TODO: at least instance-environment stuff should really come from a join query
        tracing::debug!("get instance environment {params:?}");
        let environment = Environment::get_by_id(&mut conn, instance.environment_id)
            .await?
            .ok_or(ServerError::RecordMissing {
                table: Table::Environment,
                uuid: instance.environment_id,
            })?;
        let state_page_num = params.state_page_num.unwrap_or(0);
        let state_limit = params.state_limit.unwrap_or(100).min(100);
        let state_offset = state_page_num * state_limit;
        let states = Instance::get_all_states(
            &mut conn,
            instance.instance_id,
            params.creator_id,
            params.state_contains.clone(),
            state_offset,
            state_limit,
        )
        .await?;

        let replay_page_num = params.replay_page_num.unwrap_or(0);
        let replay_limit = params.replay_limit.unwrap_or(100).min(100);
        let replay_offset = replay_page_num * replay_limit;
        let replays = Instance::get_all_replays(
            &mut conn,
            instance.instance_id,
            params.creator_id,
            params.replay_contains.clone(),
            replay_offset,
            replay_limit,
        )
        .await?;

        let save_page_num = params.save_page_num.unwrap_or(0);
        let save_limit = params.save_limit.unwrap_or(100).min(100);
        let save_offset = save_page_num * save_limit;
        let saves = Instance::get_all_saves(
            &mut conn,
            instance.instance_id,
            params.creator_id,
            params.save_contains.clone(),
            save_offset,
            save_limit,
        )
        .await?;

        let objects = ObjectLink::get_all_for_instance_id(&mut conn, id).await?;
        let state_has_more = states.len() >= state_limit as usize;
        let save_has_more = saves.len() >= save_limit as usize;
        let replay_has_more = replays.len() >= replay_limit as usize;
        tracing::debug!(
            "{} - {state_has_more} - {} - {replay_has_more}",
            states.len(),
            replays.len()
        );
        let full_instance = FullInstance {
            work: Work::get_by_id(&mut conn, instance.work_id).await?.unwrap(),
            info: instance,
            environment,
            objects,
            states,
            saves,
            replays,
        };

        let accept: Option<String> = parse_header(&headers, "Accept");

        let user = auth.user.as_ref().map(LoggedInUserInfo::generate_from_user);

        Ok(
            (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
                let instance_all_listing = app_state
                    .templates
                    .get_template("instance_all_listing.html")?;
                Html(instance_all_listing.render(context!(
                    base_url => BASE_URL.get(),
                    state_has_more => state_has_more,
                    save_has_more => save_has_more,
                    replay_has_more => replay_has_more,
                    state_page_num => state_page_num,
                    state_limit => state_limit,
                    state_contains => params.state_contains,
                    replay_page_num => replay_page_num,
                    replay_limit => replay_limit,
                    replay_contains => params.replay_contains,
                    save_page_num => save_page_num,
                    save_limit => save_limit,
                    save_contains => params.save_contains,
                    creator_id => params.creator_id,
                    instance => full_instance,
                    user => user,
                ))?)
                .into_response()
            } else if accept
                .as_ref()
                .is_some_and(|hv| hv.contains("application/json"))
            {
                Json(full_instance).into_response()
            } else {
                Err(ServerError::MimeType)?
            })
            .into_response(),
        )
    } else {
        Err(ServerError::RecordMissing {
            table: Table::Instance,
            uuid: id,
        })
    }
}

#[derive(Deserialize, Debug)]
struct CloneParams {
    state: Option<Uuid>,
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn clone_v86_instance(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
    Query(params): Query<CloneParams>,
) -> Result<axum::response::Response, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let mut conn = app_state.pool.acquire().await?;
    let state_id = params.state.ok_or(ServerError::StateRequired)?;
    let storage_path = &app_state.root_storage_path;
    let storage_depth = app_state.folder_depth;
    let new_instance = gisst::v86clone::clone_v86_machine(
        &mut conn,
        id,
        state_id,
        storage_path,
        storage_depth,
        &app_state.indexer,
    )
    .await?;
    Ok(axum::response::Redirect::permanent(&format!("/instances/{new_instance}")).into_response())
}
