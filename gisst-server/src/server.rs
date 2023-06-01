use crate::{
    serverconfig::ServerConfig,
    db,
    models::{
        ContentItem,
        Core,
        Platform,
        State,
        Save,
        Replay,
        Image,
    },
    storage::{
        StorageHandler,
    },
    templates::{
        PLAYER_TEMPLATE,
        UPLOAD_TEMPLATE,
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
    routing::{get,post},
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
use uuid::{Uuid, uuid};
use std::sync::Arc;
use axum::extract::{DefaultBodyLimit, Multipart};
use axum::extract::multipart::MultipartError;
use serde::__private::de::TagOrContentField::Content;
use bytes::Bytes;

#[derive(Debug, thiserror::Error)]
pub enum GISSTError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
    #[error("storage error")]
    StorageError(#[from] std::io::Error),
    #[error("record creation error")]
    RecordError(#[from] crate::models::NewRecordError),
    #[error("generic error")]
    Generic,
}

struct ServerState {
    pool: PgPool,
    storage: StorageHandler,
}




impl IntoResponse for GISSTError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            GISSTError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
            GISSTError::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage error"),
            GISSTError::RecordError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "record error"),
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
    });

    let app = Router::new()
        .route("/player", get(get_player))
        .route("/content/:content_id", get(get_content_json))
        .route("/content", post(content_upload))
        .route("/test_upload", get(get_upload))
        .route("/state/:state_id", get(get_state_json))
        .route("/replay/:replay_id", get(get_replay_json))
        .route("/save/:save_id", get(get_save_json))
        .route("/image/:image_id", get(get_image_json))
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


// empty_string_as_none taken from axum docs here: https://github.com/tokio-rs/axum/blob/main/examples/query-params-with-empty-strings/src/main.rs
/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where   D: Deserializer<'de>,
        T: FromStr,
        T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[derive(Deserialize, Serialize)]
struct PlayerTemplateInfo {
    content: Option<ContentItem>,
    core: Option<Core>,
    platform: Option<Platform>,
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
    content_uuid: Option<Uuid>,
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
    let mut content_item:Option<ContentItem> =  None;
    let mut platform_item:Option<Platform> = None;
    let mut state_item: Option<State> = None;
    let mut replay_item:Option<Replay>= None;
    let mut save_item:Option<Save> = None;
    let mut core_item:Option<Core> = None;

    // Check if the content_uuid param is present, if so query for the content item and its
    // platform information
    if let Some(content_uuid) = params.content_uuid {
        content_item = ContentItem::get_by_id(&mut conn, content_uuid)
            .await.map_err(|e| GISSTError::SqlError(e))?;

        platform_item = Platform::get_by_id(&mut conn, content_item.as_ref().unwrap().platform_id.unwrap())
            .await.map_err(|e| GISSTError::SqlError(e))?;

        core_item = Core::get_by_id(&mut conn, platform_item.as_ref().unwrap().core_id.unwrap())
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
            platform: platform_item,
            save: save_item,
            content: content_item,
            core: core_item,
            start: if let Some(state) = state_item {
                PlayerStartTemplateInfo::State(state)
            } else if let Some(replay) = replay_item {
                PlayerStartTemplateInfo::Replay(replay)
            } else {
                PlayerStartTemplateInfo::Cold
            }
    })))
}

async fn get_content_json(app_state: Extension<Arc<ServerState>>, Path(content_id): Path<Uuid>) -> Result<Json<ContentItem>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let content = ContentItem::get_by_id(&mut conn, content_id).await?;
    Ok(Json(content.unwrap_or_default()))
}

async fn chunk_upload(app_state: Extension<Arc<ServerState>>, mut multipart:Multipart) -> Result<(), StatusCode> {



    Ok(())
}

async fn content_upload(app_state: Extension<Arc<ServerState>>, mut multipart:Multipart) -> Result<Json<ContentItem>, GISSTError>{
    let mut content_params = ContentUploadParams{
        title: None,
        parent_id: None,
        platform_id: None,
        version: None,
    };
    let mut filename:Option<String> = None;
    let mut data: Option<Bytes> = None;
    let mut hash: Option<String> = None;

    // Iterate over form fields to collect information
    while let Some(field) = multipart.next_field().await.unwrap(){
        let name = field.name().unwrap();

        if name == "title" {
            content_params.title = field.text().await.ok();
        } else if name == "parent_id" {
            content_params.parent_id = Uuid::parse_str(&field.text().await?).ok();
        } else if name == "version" {
            content_params.version = field.text().await.ok();
        } else if name == "platform_id" {
            content_params.parent_id = Uuid::parse_str(&field.text().await?).ok();
        } else if name == "content" {
            filename = Some(field.file_name().unwrap().to_string());
            data = Some(field.bytes().await.unwrap());
            hash = Some(StorageHandler::get_md5_hash(&data.clone().unwrap()));
        }
    }

    let new_uuid = Uuid::new_v4();
    let mut conn = app_state.pool.acquire().await.map_err(|e| GISSTError::SqlError(e))?;

    // Check if the content is already in the database, if not add the file to storage
    if hash.is_some() &&
        filename.is_some() &&
        data.is_some() &&
        ContentItem::get_by_hash(&mut conn, &hash.as_ref().unwrap()).await?.is_none(){

        // Get file pathing information to add to the database
        if let Ok(file_info) = app_state.storage
            .write_file_to_uuid_folder(new_uuid, &filename.unwrap(), &data.unwrap()).await {

            if let Ok(content_item) = ContentItem::insert(&mut conn, ContentItem{
                content_id: new_uuid,
                content_hash: hash,
                content_title: content_params.title,
                content_version: content_params.version,
                content_parent_id: content_params.parent_id,
                content_source_filename: Some(file_info.source_filename),
                content_dest_filename: Some(file_info.dest_filename.to_string()),
                platform_id: content_params.platform_id,
                created_on: None,
            }).await {
                return Ok(Json(content_item));
            } else {
                // If anything went wrong with adding the SQL record, delete the uploaded file
                app_state.storage.delete_file_with_uuid(new_uuid, &file_info.dest_filename).await?;
            }
        }
    }

    Err(GISSTError::Generic)
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

async fn get_upload(app_state: Extension<Arc<ServerState>>) -> Result<Html<String>, GISSTError> {
    Ok(Html(render!(UPLOAD_TEMPLATE)))
}
