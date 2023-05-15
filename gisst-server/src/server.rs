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
    },
    storage,
    templates::{
        PLAYER_TEMPLATE,
    }
};
use anyhow::Result;
use axum::{
    Router,
    Server,
    body::{
        Body,
        Bytes,
    },
    extract::{
        Path,
        Json,
        Query,
    },
    routing::{get,},
    response::{IntoResponse, Response, Html},
    http::{StatusCode, Request},
    Extension,
    };

use sqlx::PgPool;
use serde_json::json;
use std::{fmt, io, net::{IpAddr, SocketAddr}, str::FromStr};
use minijinja::render;
use serde::{Deserialize, Deserializer, de, Serialize};
use tower_http::services::ServeDir;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum GISSTError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
    #[error("storage error")]
    StorageError(#[from] std::io::Error)
}


impl IntoResponse for GISSTError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            GISSTError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
            GISSTError::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage error"),
        };

        let body = Json(json!({"error": message}));

        (status, body).into_response()
    }
}

pub async fn launch(config: &ServerConfig) -> Result<()> {
    let pool = db::new_pool(config).await?;
    let storage_handler = storage::StorageHandler::init(
        config.storage.root_folder_path.to_string(),
        config.storage.folder_depth,
    );

    let app = Router::new()
        .route("/player", get(get_player))
        .route("/content/:content_id", get(get_content))
        .route("/state/:state_id", get(get_state))
        .nest_service("/", ServeDir::new("../frontend-web/dist"))
        .layer(Extension(pool));


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
    state: Option<State>,
    replay: Option<Replay>,
    save: Option<Save>,
}


#[derive(Deserialize)]
struct PlayerParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    content_uuid: Option<Uuid>,
    state_uuid: Option<Uuid>,
    replay_uuid: Option<Uuid>,
    save_uuid: Option<Uuid>,
}

async fn get_player(db: Extension<PgPool>, Query(params): Query<PlayerParams>) -> Result<Html<String>, GISSTError> {
    let mut conn = db.acquire().await?;
    let content_item:ContentItem;
    let platform_item:Platform;
    let state_item:State;
    let replay_item:Replay;
    let save_item:Save;

    // Check if the content_uuid param is present, if so query for the content item and its
    // platform information
    if !params.content_uuid.is_none(){
        if let Ok(content) = ContentItem::get_by_id(&mut conn, params.content_uuid.unwrap()).await? {
            content_item = content;

            if let Ok(platform) = Platform::get_by_id(&mut conn, content.platform_id).await? {
                platform_item = platform;
            }
        }
    }
    // Check for an entry state for the player, if so query for the state information
    if !params.state_uuid.is_none() {
        if let Ok(state) = State::get_by_id(&mut conn, params.state_uuid.unwrap()).await? {
            state_item = state;
        }
    }
    // Check for replay for the player
    if !params.replay_uuid.is_none() {
        if let Ok(replay) = Replay::get_by_id(&mut conn, params.replay_uuid.unwrap()).await? {
            replay_item = replay;
        }
    }
    // Check for save for the player
    if !params.save_uuid.is_none() {
        if let Ok(save) = Save::get_by_id(&mut conn, params.save_uuid.unwrap()).await? {
            save_item = save;
        }
    }

    Ok(Html(render!(
        PLAYER_TEMPLATE,
        player_params => PlayerTemplateInfo{
            platform: platform_item,
            replay: replay_item,
            save: save_item,
            state: state_item,
            content: content_item,
    })))
}

async fn get_item(db:Extension<PgPool>, Path((item, id)):Path<(String, Uuid)>) -> Result<Json<???>> {

}

async fn get_content(db: Extension<PgPool>, Path(content_id): Path<Uuid>) -> Result<Json<ContentItem>, GISSTError> {
    let mut conn = db.acquire().await?;
    let content = ContentItem::get_by_id(&mut conn, content_id).await?;
    Ok(Json(content.unwrap_or(ContentItem::empty())))
}

async fn get_state(db: Extension<PgPool>, Path(state_id): Path<Uuid>) -> Result<Json<State>, GISSTError> {
    let mut conn = db.acquire().await?;
    let state = State::get_by_id(&mut conn, state_id).await?;
    Ok(Json(state.unwrap_or(State::empty())))
}