use crate::auth::AuthContext;
use crate::server::LoggedInUserInfo;
use crate::{auth, error::GISSTError, server::ServerState, utils::parse_header};
use axum::{
    extract::{Json, Path, Query},
    headers::HeaderMap,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post, put},
    Extension, Router,
};
use axum_login::RequireAuthorizationLayer;
use gisst::models::{
    Creator, DBHashable, DBModel, Environment, File, Image, Instance, Object, Replay, Save, State,
    Work,
};
use gisst::models::{CreatorReplayInfo, CreatorStateInfo, FileRecordFlatten};
use gisst::{error::ErrorTable, models::ObjectLink};
use minijinja::context;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use uuid::Uuid;

// Nested Router structs for easier reading and manipulation

pub fn creator_router() -> Router {
    Router::new().route("/:id", get(get_single_creator))
}

pub fn screenshot_router() -> Router {
    Router::new()
        .route("/create", post(create_screenshot))
        .route("/:id", get(get_single_screenshot))
}

pub fn environment_router() -> Router {
    Router::new()
        .route("/", get(get_environments))
        .route("/create", post(create_environment))
        .route(
            "/:id",
            get(get_single_environment)
                .put(edit_environment)
                .delete(delete_environment),
        )
}

pub fn image_router() -> Router {
    Router::new()
        .route("/", get(get_images))
        .route("/create", post(create_image))
        .route(
            "/:id",
            get(get_single_image).put(edit_image).delete(delete_image),
        )
}

pub fn instance_router() -> Router {
    Router::new()
        .route("/", get(get_instances))
        .route("/create", post(create_instance))
        .route(
            "/:id",
            get(get_single_instance)
                .put(edit_instance)
                .delete(delete_instance),
        )
        .route("/:id/all", get(get_all_for_instance))
}

pub fn object_router() -> Router {
    Router::new()
        .route("/", get(get_objects))
        .route("/create", post(create_object))
        .route(
            "/:id",
            get(get_single_object)
                .put(edit_object)
                .delete(delete_object),
        )
        .route("/:id/*path", get(get_subobject))
}

pub fn replay_router() -> Router {
    Router::new()
        .route("/", get(get_replays))
        .route("/create", post(create_replay))
        .route(
            "/:id",
            get(get_single_replay)
                .put(edit_replay)
                .delete(delete_replay),
        )
}
pub fn save_router() -> Router {
    Router::new()
        .route("/", get(get_saves))
        .route("/create", post(create_save))
        .route(
            "/:id",
            get(get_single_save).put(edit_save).delete(delete_save),
        )
}

pub fn state_router() -> Router {
    Router::new()
        .route("/", get(get_states))
        .route("/:id", get(get_single_state))
        .route("/create", post(create_state))
        .route_layer(RequireAuthorizationLayer::<i32, auth::User, auth::Role>::login())
        .route("/:id", put(edit_state).delete(delete_state))
        .route_layer(RequireAuthorizationLayer::<i32, auth::User, auth::Role>::login())
}

pub fn work_router() -> Router {
    Router::new()
        .route("/", get(get_works))
        .route("/create", post(create_work))
        .route(
            "/:id",
            get(get_single_work).put(edit_work).delete(delete_work),
        )
}

// CREATOR method handlers

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
    auth: auth::AuthContext,
) -> Result<axum::response::Response, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Some(creator) = Creator::get_by_id(&mut conn, id).await? {
        let creator_results = GetAllCreatorResult {
            states: Creator::get_all_state_info(&mut conn, creator.creator_id).await?,
            replays: Creator::get_all_replay_info(&mut conn, creator.creator_id).await?,
            creator,
        };

        let accept: Option<String> = parse_header(&headers, "Accept");

        let user = auth
            .current_user
            .as_ref()
            .map(LoggedInUserInfo::generate_from_user);

        Ok(
            (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
                let creator_page = app_state
                    .templates
                    .get_template("creator_all_listing.html")?;
                Html(creator_page.render(context!(
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
                Err(GISSTError::MimeTypeError)?
            })
            .into_response(),
        )
    } else {
        Err(GISSTError::RecordMissingError {
            table: ErrorTable::Creator,
            uuid: id,
        })
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct ScreenshotCreateInfo {
    #[serde_as(as = "Base64")]
    screenshot_data: Vec<u8>,
}

async fn create_screenshot(
    app_state: Extension<ServerState>,
    Json(screenshot): Json<ScreenshotCreateInfo>,
) -> Result<Json<gisst::models::Screenshot>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(
        gisst::models::Screenshot::insert(
            &mut conn,
            gisst::models::Screenshot {
                screenshot_id: Uuid::new_v4(),
                screenshot_data: screenshot.screenshot_data,
            },
        )
        .await?,
    ))
}

async fn get_single_screenshot(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<gisst::models::Screenshot>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(
        gisst::models::Screenshot::get_by_id(&mut conn, id)
            .await?
            .unwrap(),
    ))
}

// ENVIRONMENT method handlers
#[derive(Deserialize)]
struct GetQueryParams {
    limit: Option<i64>,
}

async fn get_environments(
    app_state: Extension<ServerState>,
    Query(params): Query<GetQueryParams>,
) -> Result<Json<Vec<Environment>>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Ok(environments) = Environment::get_all(&mut conn, params.limit).await {
        Ok(environments.into())
    } else {
        Ok(Json(vec![]))
    }
}

async fn create_environment(
    app_state: Extension<ServerState>,
    Query(environment): Query<Environment>,
) -> Result<Json<Environment>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Environment::insert(&mut conn, environment).await?))
}

async fn get_single_environment(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Environment>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Environment::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_environment(
    app_state: Extension<ServerState>,
    Query(environment): Query<Environment>,
) -> Result<Json<Environment>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Environment::update(&mut conn, environment).await?))
}

async fn delete_environment(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Environment::delete_environment_image_links_by_id(&mut conn, id).await?;
    Environment::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}

// IMAGE method handlers

async fn get_images(
    app_state: Extension<ServerState>,
    Query(params): Query<GetQueryParams>,
) -> Result<Json<Vec<Image>>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Ok(images) = Image::get_all(&mut conn, params.limit).await {
        Ok(images.into())
    } else {
        Ok(Json(vec![]))
    }
}

async fn create_image(
    app_state: Extension<ServerState>,
    Query(image): Query<Image>,
) -> Result<Json<Image>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, image.file_id).await?.is_some() {
        Ok(Json(Image::insert(&mut conn, image).await?))
    } else {
        Err(GISSTError::RecordMissingError {
            table: ErrorTable::File,
            uuid: image.file_id,
        })
    }
}

async fn get_single_image(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Image>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Image::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_image(
    app_state: Extension<ServerState>,
    Query(image): Query<Image>,
) -> Result<Json<Image>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Image::update(&mut conn, image).await?))
}

async fn delete_image(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Image::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}
// INSTANCE method handlers

#[derive(Serialize)]
struct GetAllInstanceResult {
    instance: Instance,
    work: Work,
}

async fn get_instances(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Query(params): Query<GetQueryParams>,
    auth: AuthContext,
) -> Result<axum::response::Response, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let instances: Vec<Instance> = Instance::get_all(&mut conn, params.limit).await?;
    let accept: Option<String> = parse_header(&headers, "Accept");
    let user = auth
        .current_user
        .as_ref()
        .map(LoggedInUserInfo::generate_from_user);

    Ok(
        (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
            let instance_listing = app_state.templates.get_template("instance_listing.html")?;
            let mut instance_results: Vec<GetAllInstanceResult> = Vec::new();
            for inst in instances {
                instance_results.push(GetAllInstanceResult {
                    instance: Instance {
                        instance_config: inst.instance_config.clone(),
                        ..inst
                    },
                    work: Work::get_by_id(&mut conn, inst.work_id).await?.unwrap(),
                })
            }
            Html(instance_listing.render(context!(
                    instances => instance_results,
                    user => user,
            ))?)
            .into_response()
        } else if accept
            .as_ref()
            .is_some_and(|hv| hv.contains("application/json"))
        {
            Json(instances).into_response()
        } else {
            Err(GISSTError::MimeTypeError)?
        })
        .into_response(),
    )
}

#[derive(Debug, Serialize)]
struct FullInstance {
    info: Instance,
    work: Work,
    environment: Environment,
    states: Vec<FileRecordFlatten<State>>,
    replays: Vec<FileRecordFlatten<Replay>>,
    saves: Vec<FileRecordFlatten<Save>>,
    objects: Vec<ObjectLink>,
}

async fn get_all_for_instance(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    auth: auth::AuthContext,
) -> Result<axum::response::Response, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Some(instance) = Instance::get_by_id(&mut conn, id).await? {
        // TODO: all this stuff should really come from a join query or whatever.
        let environment = Environment::get_by_id(&mut conn, instance.environment_id)
            .await?
            .ok_or(GISSTError::RecordMissingError {
                table: ErrorTable::Environment,
                uuid: instance.environment_id,
            })?;
        let states = Instance::get_all_states(&mut conn, instance.instance_id).await?;
        let mut flattened_states: Vec<FileRecordFlatten<State>> = vec![];

        for state in states.iter() {
            flattened_states.push(State::flatten_file(&mut conn, state.clone()).await?);
        }

        let replays = Instance::get_all_replays(&mut conn, id).await?;
        let mut flattened_replays: Vec<FileRecordFlatten<Replay>> = vec![];

        for replay in replays.iter() {
            flattened_replays.push(Replay::flatten_file(&mut conn, replay.clone()).await?);
        }

        let saves = Instance::get_all_saves(&mut conn, id).await?;
        let mut flattened_saves: Vec<FileRecordFlatten<Save>> = vec![];

        for save in saves.iter() {
            flattened_saves.push(Save::flatten_file(&mut conn, save.clone()).await?);
        }

        let objects = ObjectLink::get_all_for_instance_id(&mut conn, id).await?;

        let full_instance = FullInstance {
            work: Work::get_by_id(&mut conn, instance.work_id).await?.unwrap(),
            info: instance,
            environment,
            objects,
            states: flattened_states,
            replays: flattened_replays,
            saves: flattened_saves,
        };

        let accept: Option<String> = parse_header(&headers, "Accept");

        let user = auth
            .current_user
            .as_ref()
            .map(LoggedInUserInfo::generate_from_user);

        Ok(
            (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
                let instance_all_listing = app_state
                    .templates
                    .get_template("instance_all_listing.html")?;
                Html(instance_all_listing.render(context!(
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
                Err(GISSTError::MimeTypeError)?
            })
            .into_response(),
        )
    } else {
        Err(GISSTError::RecordMissingError {
            table: ErrorTable::Instance,
            uuid: id,
        })
    }
}

async fn create_instance(
    app_state: Extension<ServerState>,
    Query(instance): Query<Instance>,
) -> Result<Json<Instance>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Instance::insert(&mut conn, instance).await?))
}

async fn get_single_instance(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Instance>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Instance::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_instance(
    app_state: Extension<ServerState>,
    Query(instance): Query<Instance>,
) -> Result<Json<Instance>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Instance::update(&mut conn, instance).await?))
}

async fn delete_instance(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Instance::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}

// OBJECT method handlers

async fn get_objects(
    app_state: Extension<ServerState>,
    Query(params): Query<GetQueryParams>,
) -> Result<Json<Vec<Object>>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Ok(objects) = Object::get_all(&mut conn, params.limit).await {
        Ok(objects.into())
    } else {
        Ok(Json(vec![]))
    }
}

async fn get_single_object(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    _auth: auth::AuthContext,
) -> Result<axum::response::Response, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    let object = Object::get_by_id(&mut conn, id)
        .await?
        .ok_or(GISSTError::RecordMissingError {
            table: ErrorTable::Object,
            uuid: id,
        })?;
    let file = File::get_by_id(&mut conn, object.file_id).await?.ok_or(
        GISSTError::RecordMissingError {
            table: ErrorTable::File,
            uuid: object.file_id,
        },
    )?;

    let accept: Option<String> = parse_header(&headers, "Accept");

    Ok(
        (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
            let object_page = app_state.templates.get_template("object_listing.html")?;
            use gisst::fslist::*;
            // TODO reuse cookie instead of reloading every time
            let path = file_to_path(&app_state.root_storage_path, &file);
            let directory = if is_disk_image(&path) {
                let image = std::fs::File::open(path)?;
                recursive_listing(image)?
            } else {
                vec![]
            };
            Html(object_page.render(context!(
                object => object,
                file => file,
                directory => directory,
            ))?)
            .into_response()
        } else if accept
            .as_ref()
            .is_some_and(|hv| hv.contains("application/json"))
        {
            Json(object).into_response()
        } else {
            Err(GISSTError::MimeTypeError)?
        })
        .into_response(),
    )
}

async fn get_subobject(
    app_state: Extension<ServerState>,
    _headers: HeaderMap,
    Path((id, subpath)): Path<(Uuid, String)>,
    _auth: auth::AuthContext,
) -> Result<axum::response::Response, GISSTError> {
    use gisst::fslist::*;

    let mut conn = app_state.pool.acquire().await?;

    let object = Object::get_by_id(&mut conn, id)
        .await?
        .ok_or(GISSTError::RecordMissingError {
            table: ErrorTable::Object,
            uuid: id,
        })?;
    let file = File::get_by_id(&mut conn, object.file_id).await?.ok_or(
        GISSTError::RecordMissingError {
            table: ErrorTable::File,
            uuid: object.file_id,
        },
    )?;
    let path = file_to_path(&app_state.root_storage_path, &file);
    let (mime, data) = {
        let subpath = subpath.clone();
        tokio::task::spawn_blocking(move || {
            if is_disk_image(&path) {
                get_file_at_path(std::fs::File::open(path)?, std::path::Path::new(&subpath))
                    .map_err(GISSTError::FSListError)
            } else {
                Err(GISSTError::SubobjectError(format!("{id}:{subpath}")))
            }
        })
        .await??
    };
    let headers = [
        (axum::http::header::CONTENT_TYPE, mime),
        (
            axum::http::header::CONTENT_DISPOSITION,
            format!(
                "attachment; filename=\"{}\"",
                std::path::Path::new(&subpath)
                    .file_name()
                    .ok_or(GISSTError::SubobjectError(
                        "can't download empty thing".to_string()
                    ))?
                    .to_string_lossy()
            ),
        ),
    ];
    Ok((headers, data).into_response())
}

async fn create_object(
    app_state: Extension<ServerState>,
    Query(object): Query<Object>,
) -> Result<Json<Object>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, object.file_id).await?.is_some() {
        Ok(Json(Object::insert(&mut conn, object).await?))
    } else {
        Err(GISSTError::RecordMissingError {
            table: ErrorTable::File,
            uuid: object.file_id,
        })
    }
}

async fn edit_object(
    app_state: Extension<ServerState>,
    Query(object): Query<Object>,
) -> Result<Json<Object>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Object::update(&mut conn, object).await?))
}

async fn delete_object(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Object::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}
// Replay method handlers

async fn get_replays(
    app_state: Extension<ServerState>,
    Query(params): Query<GetQueryParams>,
) -> Result<Json<Vec<Replay>>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Ok(replays) = Replay::get_all(&mut conn, params.limit).await {
        Ok(replays.into())
    } else {
        Ok(Json(vec![]))
    }
}

async fn get_single_replay(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Replay>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Replay::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_replay(
    app_state: Extension<ServerState>,
    Query(replay): Query<Replay>,
) -> Result<Json<Replay>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Replay::update(&mut conn, replay).await?))
}

async fn delete_replay(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Replay::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReplay {
    pub replay_name: String,
    pub replay_description: String,
    pub instance_id: Uuid,
    pub replay_forked_from: Option<Uuid>,
    pub file_id: Uuid,
}

async fn create_replay(
    app_state: Extension<ServerState>,
    auth: AuthContext,
    Json(replay): Json<CreateReplay>,
) -> Result<Json<Replay>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, replay.file_id).await?.is_some() {
        Ok(Json(
            Replay::insert(
                &mut conn,
                Replay {
                    replay_id: Uuid::new_v4(),
                    replay_name: replay.replay_name,
                    replay_description: replay.replay_description,
                    instance_id: replay.instance_id,
                    creator_id: auth
                        .current_user
                        .ok_or(GISSTError::AuthUserNotAuthenticatedError)?
                        .creator_id,
                    replay_forked_from: replay.replay_forked_from,
                    file_id: replay.file_id,
                    created_on: chrono::Utc::now(),
                },
            )
            .await?,
        ))
    } else {
        Err(GISSTError::RecordMissingError {
            table: ErrorTable::File,
            uuid: replay.file_id,
        })?
    }
}
// Save method handlers

async fn get_saves(
    app_state: Extension<ServerState>,
    Query(params): Query<GetQueryParams>,
) -> Result<Json<Vec<Save>>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Ok(saves) = Save::get_all(&mut conn, params.limit).await {
        Ok(saves.into())
    } else {
        Ok(Json(vec![]))
    }
}

async fn get_single_save(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Save>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Save::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_save(
    app_state: Extension<ServerState>,
    Query(save): Query<Save>,
) -> Result<Json<Save>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Save::update(&mut conn, save).await?))
}

async fn delete_save(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Save::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}

async fn create_save(
    app_state: Extension<ServerState>,
    Query(save): Query<Save>,
) -> Result<Json<Save>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, save.file_id).await?.is_some() {
        Ok(Json(Save::insert(&mut conn, save).await?))
    } else {
        Err(GISSTError::RecordMissingError {
            table: ErrorTable::File,
            uuid: save.file_id,
        })?
    }
}
// State method handlers

async fn get_states(
    app_state: Extension<ServerState>,
    Query(params): Query<GetQueryParams>,
) -> Result<Json<Vec<State>>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Ok(states) = State::get_all(&mut conn, params.limit).await {
        Ok(states.into())
    } else {
        Ok(Json(vec![]))
    }
}

async fn get_single_state(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<State>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(State::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_state(
    app_state: Extension<ServerState>,
    Query(state): Query<State>,
) -> Result<Json<State>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(State::update(&mut conn, state).await?))
}

async fn delete_state(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    State::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateState {
    pub instance_id: Uuid,
    pub is_checkpoint: bool,
    pub file_id: Uuid,
    pub state_name: String,
    pub state_description: String,
    pub screenshot_id: Uuid,
    pub replay_id: Option<Uuid>,
    pub state_replay_index: Option<i32>,
    pub state_derived_from: Option<Uuid>,
}

#[axum::debug_handler]
async fn create_state(
    app_state: Extension<ServerState>,
    auth: AuthContext,
    Json(state): Json<CreateState>,
) -> Result<Json<State>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, state.file_id).await?.is_some() {
        Ok(Json(
            State::insert(
                &mut conn,
                State {
                    state_id: Uuid::new_v4(),
                    instance_id: state.instance_id,
                    is_checkpoint: state.is_checkpoint,
                    file_id: state.file_id,
                    state_name: state.state_name,
                    state_description: state.state_description,
                    screenshot_id: state.screenshot_id,
                    replay_id: state.replay_id,
                    creator_id: auth
                        .current_user
                        .ok_or(GISSTError::AuthUserNotAuthenticatedError)?
                        .creator_id,
                    state_replay_index: state.state_replay_index,
                    state_derived_from: state.state_derived_from,
                    created_on: chrono::Utc::now(),
                },
            )
            .await?,
        ))
    } else {
        Err(GISSTError::RecordMissingError {
            table: ErrorTable::File,
            uuid: state.file_id,
        })?
    }
}

// WORK method handlers
async fn get_works(
    app_state: Extension<ServerState>,
    Query(params): Query<GetQueryParams>,
) -> Result<Json<Vec<Work>>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Ok(works) = Work::get_all(&mut conn, params.limit).await {
        Ok(works.into())
    } else {
        Ok(Json(vec![]))
    }
}

async fn get_single_work(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Work>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Work::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn create_work(
    app_state: Extension<ServerState>,
    Query(work): Query<Work>,
) -> Result<Json<Work>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Work::insert(&mut conn, work).await?))
}

async fn edit_work(
    app_state: Extension<ServerState>,
    Query(work): Query<Work>,
) -> Result<Json<Work>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Work::update(&mut conn, work).await?))
}

async fn delete_work(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Work::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}
