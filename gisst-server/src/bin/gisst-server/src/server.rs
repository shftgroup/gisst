use gisstlib::{
    models::{self, DBModel, Environment, Instance, Replay, Save, State},
    storage::StorageHandler,
    GISSTError,
};

use crate::{
    db,
    routes::{
        // creator_router,
        environment_router,
        image_router,
        instance_router,
        object_router,
        work_router,
    },
    serverconfig::ServerConfig,
    templates::{TemplateHandler, PLAYER_TEMPLATE},
};
use anyhow::Result;
use axum::{
    extract::{Path, Query},
    response::Html,
    routing::get,
    Extension, Router, Server,
};

use axum::extract::DefaultBodyLimit;
use minijinja::render;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tower_http::services::ServeDir;
use uuid::Uuid;

pub struct ServerState {
    pub pool: PgPool,
    pub storage: StorageHandler,
    pub templates: TemplateHandler,
}

pub async fn launch(config: &ServerConfig) -> Result<()> {
    // Arc is needed to allow for thread safety, see: https://docs.rs/axum/latest/axum/struct.Extension.html
    let app_state = Arc::new(ServerState {
        pool: db::new_pool(config).await?,
        storage: StorageHandler::init(
            config.storage.root_folder_path.to_string(),
            config.storage.folder_depth,
        ),
        templates: TemplateHandler::new("src/bin/gisst-server/src/templates")?,
    });

    let app = Router::new()
        .route("/play/:instance_id", get(get_player))
        // .nest("/creators", creator_router())
        .nest("/environments", environment_router())
        .nest("/instances", instance_router())
        .nest("/images", image_router())
        .nest("/objects", object_router())
        .nest("/works", work_router())
        .nest_service("/", ServeDir::new("../frontend-web/dist"))
        .nest_service("/storage", ServeDir::new("storage"))
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
    instance: Instance,
    environment: Environment,
    save: Option<Save>,
    start: PlayerStartTemplateInfo,
    manifest: Vec<models::ObjectLink>,
}

#[derive(Deserialize, Serialize)]
enum PlayerStartTemplateInfo {
    Cold,
    State(State),
    Replay(Replay),
}

#[derive(Deserialize)]
struct PlayerParams {
    state: Option<Uuid>,
    replay: Option<Uuid>,
}

async fn get_player(
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
    Query(params): Query<PlayerParams>,
) -> Result<Html<String>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let instance = Instance::get_by_id(&mut conn, id)
        .await?
        .ok_or(GISSTError::Generic)?;
    let environment = Environment::get_by_id(&mut conn, instance.environment_id)
        .await?
        .ok_or(GISSTError::Generic)?;
    let start = match (params.state, params.replay) {
        (Some(id), None) => PlayerStartTemplateInfo::State(
            State::get_by_id(&mut conn, id)
                .await?
                .ok_or(GISSTError::Generic)?,
        ),
        (None, Some(id)) => PlayerStartTemplateInfo::Replay(
            Replay::get_by_id(&mut conn, id)
                .await?
                .ok_or(GISSTError::Generic)?,
        ),
        (None, None) => PlayerStartTemplateInfo::Cold,
        (_, _) => return Err(GISSTError::Generic),
    };
    let manifest =
        models::ObjectLink::get_all_for_instance_id(&mut conn, instance.instance_id).await?;
    Ok(Html(render!(
        PLAYER_TEMPLATE,
        player_params => PlayerTemplateInfo {
            environment,
            instance,
            save: None,
            start,
            manifest
        }
    )))
}

// #[derive(Serialize)]
// struct ModelInfo {
//     name: String,
//     fields: Vec<ModelField>
// }

// #[derive(Serialize)]
// struct ModelField { name: String, field_type: String }
//
// async fn get_upload_form(app_state: Extension<Arc<ServerState>>, Path(model): Path<String>) -> Result<Html<String>, GISSTError> {
//
//     let mut model_name = model.clone();
//     make_ascii_title_case(&mut model_name);
//
//     let model_schema = get_model_by_string(&model_name).fields();
//     println!("{:?}", model_schema);
//
//     Ok(Html(render!(
//         app_state.templates.get_template("debug_upload")?,
//         model => ModelInfo{
//             name: model,
//             fields: convert_model_field_vec_to_form_fields(model_schema)
//         }
//     )))
// }

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
