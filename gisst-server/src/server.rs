use crate::{
    serverconfig::ServerConfig,
    db,
    models::{
        DBModel,
        Image,
        Save,
        Replay,
        State,
        Instance,
        Environment,
        Object,
    },
    storage::{
        StorageHandler,
    },
    routes::{
        creator_router,
        environment_router,
        image_router,
        instance_router,
        object_router,
        work_router
    },
    templates::{
        TemplateHandler,
        PLAYER_TEMPLATE
    }
};
use anyhow::Result;
use axum::{
    Router,
    Server,
    extract::{
        Path,
        Json,
        Query,
    },
    routing::{
        get,
        post
    },
    response::{IntoResponse, Response, Html},
    http::{StatusCode},
    Extension,
    };

use sqlx::PgPool;
use serde_json::json;
use std::{fmt, net::{IpAddr, SocketAddr}, str::FromStr};
use std::fmt::Debug;
use minijinja::{render, Template};
use serde::{Deserialize, Deserializer, de, Serialize};
use tower_http::services::ServeDir;
use uuid::Uuid;
use std::sync::Arc;
use axum::extract::{DefaultBodyLimit, Multipart};
use axum::extract::multipart::MultipartError;
use bytes::Bytes;
use crate::models::{Creator, get_model_by_string};

#[derive(Debug, thiserror::Error)]
pub enum GISSTError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
    #[error("storage error")]
    StorageError(#[from] std::io::Error),
    #[error("record creation error")]
    RecordError(#[from] crate::models::NewRecordError),
    #[error("template error")]
    TemplateError,
    #[error("generic error")]
    Generic,
}

pub struct ServerState {
    pub pool: PgPool,
    pub storage: StorageHandler,
    pub templates: TemplateHandler,
}


impl IntoResponse for GISSTError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            GISSTError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
            GISSTError::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage error"),
            GISSTError::RecordError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "record error"),
            GISSTError::TemplateError => (StatusCode::INTERNAL_SERVER_ERROR, "template error"),
            GISSTError::Generic => (StatusCode::INTERNAL_SERVER_ERROR, "generic error"),
        };

        let body = Json(json!({"error": message}));

        (status, body).into_response()
    }
}

impl From<MultipartError> for GISSTError {
    fn from(error: MultipartError) -> Self {
        GISSTError::Generic
    }
}

pub async fn launch(config: &ServerConfig) -> Result<()> {
    // Arc is needed to allow for thread safety, see: https://docs.rs/axum/latest/axum/struct.Extension.html
    let app_state = Arc::new(ServerState{
        pool: db::new_pool(config).await?,
        storage: StorageHandler::init(config.storage.root_folder_path.to_string(), config.storage.folder_depth),
        templates: TemplateHandler::new("src/templates")?,
    });

    let app = Router::new()
        .route("/player", get(get_player))
        // .nest("/creators", creator_router())
        .nest("/environments", environment_router())
        .nest("/images", image_router())
        .nest("/instances", instance_router())
        .nest("/images", image_router())
        .nest("/objects", object_router())
        // .nest("/works", work_router())
        .nest_service("/", ServeDir::new("../frontend-web/dist"))
        .layer(Extension(app_state))
        .layer(DefaultBodyLimit::max(33554432));


    let addr = SocketAddr::new(
        IpAddr::V4(config.http.listen_address),
        config.http.listen_port,
    );

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("could not launch GISST HTTP server on port 3000");

    Ok(())
}



#[derive(Deserialize, Serialize)]
struct PlayerTemplateInfo {
    instance: Option<Instance>,
    environment: Option<Environment>,
    save: Option<Save>,
    start: PlayerStartTemplateInfo,
}

#[derive(Deserialize,Serialize)]
enum PlayerStartTemplateInfo {
    Cold,
    State(State),
    Replay(Replay),
}

#[derive(Deserialize)]
struct PlayerParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    instance_uuid: Option<Uuid>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    state_uuid: Option<Uuid>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    replay_uuid: Option<Uuid>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    save_uuid: Option<Uuid>,
}

#[derive(Deserialize)]
struct ContentUploadParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    title: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    platform_id: Option<Uuid>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    parent_id: Option<Uuid>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    version: Option<String>,
}

async fn get_player(app_state: Extension<Arc<ServerState>>, Query(params): Query<PlayerParams>) -> Result<Html<String>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let mut instance_item:Option<Instance> =  None;
    let mut environment_item:Option<Environment> = None;
    let mut state_item: Option<State> = None;
    let mut replay_item:Option<Replay>= None;
    let mut save_item:Option<Save> = None;

    // Check if the content_uuid param is present, if so query for the content item and its
    // platform information
    if let Some(instance_uuid) = params.instance_uuid {
        instance_item = Instance::get_by_id(&mut conn, instance_uuid)
            .await.map_err(|e| GISSTError::SqlError(e))?;

        environment_item = Environment::get_by_id(&mut conn, instance_item.as_ref().unwrap().environment_id)
            .await.map_err(|e| GISSTError::SqlError(e))?;
    }

    // Check for an entry state for the player, if so query for the state information
    if !params.state_uuid.is_none() {
        if let Ok(state) = State::get_by_id(&mut conn, params.state_uuid.unwrap()).await?.ok_or("Cannot get state from database"){
            state_item = Some(state);
        }
    }
    // Check for replay for the player
    if !params.replay_uuid.is_none() {
        if let Ok(replay) = Replay::get_by_id(&mut conn, params.replay_uuid.unwrap()).await?.ok_or("Cannot get replay from database"){
            replay_item = Some(replay);
        }
    }
    // Check for save for the player
    if !params.save_uuid.is_none() {
        if let Ok(save) = Save::get_by_id(&mut conn, params.save_uuid.unwrap()).await?.ok_or("Cannot get save from database"){
            save_item = Some(save);
        }
    }

    Ok(Html(render!(
        PLAYER_TEMPLATE,
        player_params => PlayerTemplateInfo{
            environment: environment_item,
            save: save_item,
            instance: instance_item,
            start: if let Some(state) = state_item {
                PlayerStartTemplateInfo::State(state)
            } else if let Some(replay) = replay_item {
                PlayerStartTemplateInfo::Replay(replay)
            } else {
                PlayerStartTemplateInfo::Cold
            }
    })))
}

async fn get_instance_json(app_state: Extension<Arc<ServerState>>, Path(content_id): Path<Uuid>) -> Result<Json<Instance>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let content = Instance::get_by_id(&mut conn, content_id).await?;
    Ok(Json(content.unwrap_or_default()))
}

async fn get_state_json(app_state: Extension<Arc<ServerState>>, Path(state_id): Path<Uuid>) -> Result<Json<State>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let state = State::get_by_id(&mut conn, state_id).await?;
    Ok(Json(state.unwrap_or_default()))
}

async fn get_replay_json(app_state: Extension<Arc<ServerState>>, Path(replay_id): Path<Uuid>) -> Result<Json<Replay>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let replay = Replay::get_by_id(&mut conn, replay_id).await?;
    Ok(Json(replay.unwrap_or_default()))
}

async fn get_save_json(app_state: Extension<Arc<ServerState>>, Path(save_id): Path<Uuid>) -> Result<Json<Save>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let save = Save::get_by_id(&mut conn, save_id).await?;
    Ok(Json(save.unwrap_or_default()))
}
async fn get_image_json(app_state: Extension<Arc<ServerState>>, Path(image_id): Path<Uuid>) -> Result<Json<Image>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let image = Image::get_by_id(&mut conn, image_id).await?;
    Ok(Json(image.unwrap_or_default()))
}

#[derive(Serialize)]
struct ModelInfo {
    name: String,
    fields: Vec<ModelField>
}

#[derive(Serialize)]
struct ModelField { name: String, field_type: String }

async fn get_upload_form(app_state: Extension<Arc<ServerState>>, Path(model): Path<String>) -> Result<Html<String>, GISSTError> {

    let mut model_name = model.clone();
    make_ascii_title_case(&mut model_name);

    let model_schema = get_model_by_string(&model_name).fields();
    println!("{:?}", model_schema);

    Ok(Html(render!(
        app_state.templates.get_template("debug_upload")?,
        model => ModelInfo{
            name: model,
            fields: convert_model_field_vec_to_form_fields(model_schema)
        }
    )))
}

async fn get_model_view(app_state: Extension<Arc<ServerState>>, Path(model): Path<String>) -> Result<Html<String>, GISSTError> {


}

// Utility Functions

fn convert_model_field_vec_to_form_fields(fields:Vec<(String,String)>) -> Vec<ModelField> {
    fields.iter().map(|(field, ft)| match ft.as_str() {
        "OffsetDateTime" => ModelField { name: field.to_string(), field_type: "datetime-local".to_string() },
        "i32" => ModelField { name: field.to_string(), field_type: "number".to_string() },
        "Uuid" => ModelField { name: field.to_string(), field_type: "text".to_string() },
        "String" => ModelField { name: field.to_string(), field_type: "text".to_string() },
        "Json" => ModelField { name: field.to_string(), field_type: "file".to_string() },
         _ => ModelField { name: field.to_string(), field_type: ft.to_string() }
    }).collect()
}

fn make_ascii_title_case(s: &mut str) {
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
}

