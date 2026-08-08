use crate::{
    error::ServerError,
    server::ServerState,
    task::{Task, TaskState},
};
use axum::{
    Extension, Router,
    extract::{Json, Path, Query},
    http::{HeaderMap, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/{id}/{update}", post(task_update))
        .route("/{id}", get(get_single_task))
        .route("/claim", post(claim_task))
        .route("/", get(list_tasks))
}

#[derive(Deserialize, Debug)]
struct TaskListQueryParams {
    state: Option<TaskState>,
    task_type: Option<String>,
}

#[tracing::instrument(skip(app_state))]
async fn list_tasks(
    app_state: Extension<ServerState>,
    params: Query<TaskListQueryParams>,
) -> Result<axum::response::Response, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(
        Json(Task::get_tasks(&mut conn, params.state, params.task_type.as_deref()).await?)
            .into_response(),
    )
}

#[tracing::instrument(skip(app_state))]
async fn get_single_task(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<axum::response::Response, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Task::get_by_id(&mut conn, id).await?.unwrap()).into_response())
}

#[derive(Deserialize, Debug)]
struct TaskClaimParams {
    task_type: Option<String>,
}

fn authenticate_worker(
    headers: &HeaderMap,
    app_state: &ServerState,
) -> Result<String, ServerError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|hv| hv.to_str().ok())
        .ok_or(ServerError::PermissionDenied)?;
    // We expect Authorization: Bearer worker-XXXXYYYY...
    let (authtype, authval) = auth_header
        .split_once(' ')
        .ok_or(ServerError::PermissionDenied)?;
    if authtype != "Bearer"
        || !authval.starts_with("worker-")
        || !app_state
            .task_worker_keys
            .iter()
            .any(|k| k.as_str() == authval)
    {
        return Err(ServerError::PermissionDenied);
    }
    Ok(authval.to_string())
}

#[tracing::instrument(skip(app_state))]
async fn claim_task(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    params: Json<TaskClaimParams>,
) -> Result<axum::response::Response, ServerError> {
    let claimant = authenticate_worker(&headers, &app_state)?;
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(
        Task::claim_available(&mut conn, params.task_type.as_deref(), &claimant)
            .await?
            .ok_or(ServerError::NoTaskReady)?,
    )
    .into_response())
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum TaskUpdate {
    Status,
    Error,
    Complete,
}

#[derive(Deserialize, Debug)]
struct TaskUpdateParams {
    status: serde_json::Value,
}

#[tracing::instrument(skip(app_state))]
async fn task_update(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path((id, update)): Path<(Uuid, TaskUpdate)>,
    params: Json<TaskUpdateParams>,
) -> Result<axum::response::Response, ServerError> {
    let claimant = authenticate_worker(&headers, &app_state)?;
    let mut conn = app_state.pool.acquire().await?;
    let task = Task::get_by_id(&mut conn, id)
        .await?
        .ok_or(ServerError::FileNotFound)?;
    if task.task_claimant.as_ref() != Some(&claimant) {
        return Err(ServerError::PermissionDenied);
    }
    let status = params.status.clone();
    match update {
        TaskUpdate::Status => Task::update_status(&mut conn, id, status).await?,
        TaskUpdate::Error => Task::error(&mut conn, id, status).await?,
        TaskUpdate::Complete => Task::complete(&mut conn, id, status).await?,
    }
    Ok(Json(serde_json::json!({})).into_response())
}
