use crate::error::GISSTError;
use gisst::models::{DBModel, Environment, Instance, Save};
use std::collections::HashMap;
use tower::ServiceBuilder;

use crate::{
    db,
    routes::{
        // creator_router,
        environment_router,
        image_router,
        instance_router,
        object_router,
        replay_router,
        save_router,
        state_router,
        work_router,
    },
    serverconfig::ServerConfig,
    templates::{TemplateHandler, PLAYER_TEMPLATE},
    tus::{tus_creation, tus_head, tus_patch},
};
use anyhow::Result;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, Query},
    response::Html,
    routing::method_routing::{get, patch, post},
    Extension, Router, Server,
};

use axum::extract::DefaultBodyLimit;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use gisst::storage::{PendingUpload, StorageHandler};
use minijinja::render;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use tower_http::{cors::CorsLayer, services::ServeDir};
use uuid::Uuid;

#[derive(Clone)]
pub struct ServerState {
    pub pool: PgPool,
    pub root_storage_path: String,
    pub temp_storage_path: String,
    pub folder_depth: u8,
    pub default_chunk_size: usize,
    pub pending_uploads: Arc<RwLock<HashMap<Uuid, PendingUpload>>>,
    pub templates: TemplateHandler,
}

pub async fn launch(config: &ServerConfig) -> Result<()> {
    StorageHandler::init_storage(
        &config.storage.root_folder_path,
        &config.storage.temp_folder_path,
    )?;
    let app_state = ServerState {
        pool: db::new_pool(config).await?,
        root_storage_path: config.storage.root_folder_path.clone(),
        temp_storage_path: config.storage.temp_folder_path.clone(),
        folder_depth: config.storage.folder_depth,
        default_chunk_size: config.storage.chunk_size,
        pending_uploads: Default::default(),
        templates: TemplateHandler::new("gisst-server/src/templates")?,
    };

    let app = Router::new()
        .route("/play/:instance_id", get(get_player))
        .route("/resources/:id", patch(tus_patch).head(tus_head))
        .route("/resources", post(tus_creation))
        .route("/debug/tus_test", get(get_upload_form))
        // .nest("/creators", creator_router())
        .nest("/environments", environment_router())
        .nest("/instances", instance_router())
        .nest("/images", image_router())
        .nest("/replays", replay_router())
        .nest("/saves", save_router())
        .nest("/states", state_router())
        .nest("/objects", object_router())
        .nest("/works", work_router())
        .nest_service("/", ServeDir::new("../frontend-web/dist"))
        .nest_service(
            "/storage",
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin(tower_http::cors::AllowOrigin::any())
                        .expose_headers([
                            axum::http::header::ACCEPT_RANGES,
                            axum::http::header::CONTENT_LENGTH,
                            axum::http::header::RANGE,
                            axum::http::header::CONTENT_RANGE,
                        ])
                        .allow_methods([axum::http::Method::GET]),
                )
                .layer(HandleErrorLayer::new(handle_error))
                // This map_err is needed to get the types to work out after handleerror and before servedir.
                .map_err(|e| panic!("{:?}", e))
                .service(ServeDir::new("storage")),
        )
        .layer(Extension(app_state))
        .layer(DefaultBodyLimit::max(33554432));

    let addr = SocketAddr::new(
        IpAddr::V4(config.http.listen_address),
        config.http.listen_port.clone(),
    );

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("could not launch GISST HTTP server on port 3000");

    Ok(())
}
async fn handle_error(error: axum::BoxError) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", error),
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectLink {
    pub object_id: Uuid,
    pub object_role: gisst::models::ObjectRole,
    pub file_hash: String,
    pub file_filename: String,
    pub file_source_path: String,
    pub file_dest_path: String,
}
impl ObjectLink {
    pub async fn get_all_for_instance_id(
        conn: &mut sqlx::PgConnection,
        id: Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT object_id, instanceObject.object_role as "object_role:_", file.file_hash as file_hash, file.file_filename as file_filename, file.file_source_path as file_source_path, file.file_dest_path as file_dest_path
            FROM object
            JOIN instanceObject USING(object_id)
            JOIN instance USING(instance_id)
            JOIN file USING(file_id)
            WHERE instance_id = $1
            "#,
            id
        )
        .fetch_all(conn)
        .await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ReplayLink {
    pub replay_id: Uuid,
    pub instance_id: Uuid,
    pub creator_id: Uuid,
    pub replay_forked_from: Option<Uuid>,
    pub created_on: Option<time::OffsetDateTime>,
    pub file_hash: String,
    pub file_filename: String,
    pub file_source_path: String,
    pub file_dest_path: String,
}
impl ReplayLink {
    pub async fn get_by_id(conn: &mut sqlx::PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT replay_id,
            instance_id,
            creator_id,
            replay_forked_from,
            replay.created_on,
            file.file_hash as file_hash,
            file.file_filename as file_filename,
            file.file_source_path as file_source_path,
            file.file_dest_path as file_dest_path
            FROM replay
            JOIN file USING(file_id)
            WHERE replay_id = $1
            "#,
            id,
        )
        .fetch_optional(conn)
        .await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StateLink {
    pub state_id: Uuid,
    pub instance_id: Uuid,
    pub is_checkpoint: bool,
    pub state_name: String,
    pub state_description: String,
    pub screenshot_id: Option<Uuid>,
    pub replay_id: Option<Uuid>,
    pub creator_id: Option<Uuid>,
    pub state_replay_index: Option<i32>,
    pub state_derived_from: Option<Uuid>,
    pub created_on: Option<time::OffsetDateTime>,
    pub file_hash: String,
    pub file_filename: String,
    pub file_source_path: String,
    pub file_dest_path: String,
}
impl StateLink {
    pub async fn get_by_id(conn: &mut sqlx::PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT
            state_id,
            instance_id,
            is_checkpoint,
            state_name,
            state_description,
            screenshot_id,
            replay_id,
            creator_id,
            state_replay_index,
            state_derived_from,
            state.created_on,
            file.file_hash as file_hash,
            file.file_filename as file_filename,
            file.file_source_path as file_source_path,
            file.file_dest_path as file_dest_path
            FROM state
            JOIN file USING(file_id)
            WHERE state_id = $1
            "#,
            id,
        )
        .fetch_optional(conn)
        .await
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct PlayerTemplateInfo {
    instance: Instance,
    environment: Environment,
    save: Option<Save>,
    start: PlayerStartTemplateInfo,
    manifest: Vec<ObjectLink>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
enum PlayerStartTemplateInfo {
    Cold,
    State(StateLink),
    Replay(ReplayLink),
}

#[derive(Deserialize)]
struct PlayerParams {
    state: Option<Uuid>,
    replay: Option<Uuid>,
}

async fn get_player(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(params): Query<PlayerParams>,
) -> Result<axum::response::Response, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let instance = Instance::get_by_id(&mut conn, id)
        .await?
        .ok_or(GISSTError::Generic)?;
    let environment = Environment::get_by_id(&mut conn, instance.environment_id)
        .await?
        .ok_or(GISSTError::Generic)?;
    let start = match dbg!((params.state, params.replay)) {
        (Some(id), None) => PlayerStartTemplateInfo::State(
            StateLink::get_by_id(&mut conn, id)
                .await?
                .ok_or(GISSTError::Generic)?,
        ),
        (None, Some(id)) => PlayerStartTemplateInfo::Replay(
            ReplayLink::get_by_id(&mut conn, id)
                .await?
                .ok_or(GISSTError::Generic)?,
        ),
        (None, None) => PlayerStartTemplateInfo::Cold,
        (_, _) => return Err(GISSTError::Generic),
    };
    let manifest =
        dbg!(ObjectLink::get_all_for_instance_id(&mut conn, instance.instance_id).await?);
    let accept = dbg!(headers
        .get("Accept")
        .map(|hv| hv.to_str())
        .and_then(|hv| hv.ok()));
    Ok((
        [("Access-Control-Allow-Origin", "*")],
        if accept.is_none() || accept.is_some_and(|hv| hv.contains("text/html")) {
            Html(render!(
                PLAYER_TEMPLATE,
                player_params => PlayerTemplateInfo {
                    environment,
                    instance,
                    save: None,
                    start,
                    manifest
                }
            ))
            .into_response()
        } else if accept.is_some_and(|hv| hv.contains("application/json")) {
            println!("send json response");
            axum::Json(PlayerTemplateInfo {
                environment,
                instance,
                save: None,
                start,
                manifest,
            })
            .into_response()
        } else {
            println!("send error response");
            Err(GISSTError::Generic)?
        },
    )
        .into_response())
}

// #[derive(Serialize)]
// struct ModelInfo {
//     name: String,
//     fields: Vec<ModelField>
// }

// #[derive(Serialize)]
// struct ModelField { name: String, field_type: String }
//
async fn get_upload_form(app_state: Extension<ServerState>) -> Result<Html<String>, GISSTError> {
    Ok(Html(render!(app_state
        .templates
        .get_template("debug_upload")?,)))
}

// Utility Functions

// fn convert_model_field_vec_to_form_fields(fields:Vec<(String,String)>) -> Vec<ModelField> {
//     fields.iter().map(|(field, ft)| match ft.as_str() {
//         "OffsetDateTime" => ModelField { name: field.to_string(), field_type: "datetime-local".to_string() },
//         "i32" => ModelField { name: field.to_string(), field_type: "number".to_string() },
//         "Uuid" => ModelField { name: field.to_string(), field_type: "text".to_string() },
//         "String" => ModelField { name: field.to_string(), field_type: "text".to_string() },
//         "Json" => ModelField { name: field.to_string(), field_type: "file".to_string() },
//          _ => ModelField { name: field.to_string(), field_type: ft.to_string() }
//     }).collect()
// }

// fn make_ascii_title_case(s: &mut str) {
//     if let Some(r) = s.get_mut(0..1) {
//         r.make_ascii_uppercase();
//     }
// }