use super::LoggedInUserInfo;
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
use gisst::models::{Environment, Instance, InstanceWork, ObjectLink, Work};
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
    params: Query<InstanceListQueryParams>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
) -> Result<axum::response::Response, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let accept: Option<String> = parse_header(&headers, "Accept");
    let user = auth.user.as_ref().map(LoggedInUserInfo::generate_from_user);
    Ok(
        (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
            let (search_url, search_key) = app_state.search.frontend_data();
            let instance_listing = app_state.templates.get_template("instance_listing.html")?;
            Html(instance_listing.render(context!(
                base_url => BASE_URL.get(),
                search_url => search_url,
                search_key => search_key,
                user => user,
            ))?)
            .into_response()
        } else if accept
            .as_ref()
            .is_some_and(|hv| hv.contains("application/json"))
        {
            let page_num = params.page_num.unwrap_or(1);
            let limit = params.limit.unwrap_or(100).min(100);
            let search = app_state.search.instances();
            let results: meilisearch_sdk::search::SearchResults<InstanceWork> = search
                .search()
                .with_facets(meilisearch_sdk::search::Selectors::Some(&["work_platform"]))
                .with_hits_per_page(limit as usize)
                .with_page(page_num as usize)
                .with_filter(
                    &params
                        .platform
                        .as_ref()
                        .map(|p| format!("work_platform = \"{p}\""))
                        .unwrap_or_default(),
                )
                .with_query(&params.contains.clone().unwrap_or_default())
                .execute()
                .await
                .map_err(gisst::error::Search::from)?;
            let instances: Vec<_> = results.hits.into_iter().map(|r| r.result).collect();
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
    objects: Vec<ObjectLink>,
}

#[derive(Debug, serde::Deserialize)]
struct StateReplayPageQueryParams {
    creator_id: Option<uuid::Uuid>,
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
        // TODO: instance-environment-work stuff should really come from a join query
        tracing::debug!("get instance environment {params:?}");
        let work = Work::get_by_id(&mut conn, instance.work_id).await?.unwrap();
        let environment = Environment::get_by_id(&mut conn, instance.environment_id)
            .await?
            .ok_or(ServerError::RecordMissing {
                table: Table::Environment,
                uuid: instance.environment_id,
            })?;
        let objects = ObjectLink::get_all_for_instance_id(&mut conn, id).await?;

        let accept: Option<String> = parse_header(&headers, "Accept");

        let user = auth.user.as_ref().map(LoggedInUserInfo::generate_from_user);

        Ok(
            (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
                let (search_url, search_key) = app_state.search.frontend_data();
                let instance_all_listing = app_state
                    .templates
                    .get_template("instance_all_listing.html")?;
                Html(instance_all_listing.render(context!(
                    base_url => BASE_URL.get(),
                    search_url => search_url,
                    search_key => search_key,
                    creator_id => params.creator_id,
                    instance => instance,
                    work => work,
                    environment => environment,
                    objects => objects,
                    user => user,
                ))?)
                .into_response()
            } else if accept
                .as_ref()
                .is_some_and(|hv| hv.contains("application/json"))
            {
                let full_instance = FullInstance {
                    work,
                    info: instance,
                    environment,
                    objects,
                };
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
    // TODO: use transaction
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
