use crate::{error::GISSTError, server::ServerState, utils::parse_header};
use axum::{
    extract::{Json, Path, Query},
    headers::HeaderMap,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use gisst::models::{
    DBHashable, DBModel, Environment, File, Image, Instance, Object, Replay, Save, State, Work,
};
use gisst::models::{FileRecordFlatten, NewRecordError};
use minijinja::render;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use sqlx::PgConnection;
use uuid::{uuid, Uuid};

// Nested Router structs for easier reading and manipulation
// pub fn creator_router() -> Router {
//     Router::new()
//         .route("/", get(get_creators))
//         .route("/create", post(create_creator))
//         .route("/:id", get(get_single_creator)
//             .put(edit_creator)
//             .delete(delete_creator))
// }

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
        .route("/create", post(create_state))
        .route(
            "/:id",
            get(get_single_state).put(edit_state).delete(delete_state),
        )
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
// #[derive(Deserialize)]
// struct CreatorsGetQueryParams {
//     limit: Option<i64>,
// }

// async fn get_creators(
//     app_state: Extension<Arc<ServerState>>,
//     Query(params): Query<CreatorsGetQueryParams>,
// ) -> Result<Json<Vec<Creator>>, GISSTError> {
//     let mut conn = app_state.pool.acquire().await?;
//     if let Ok(creators) = Creator::get_all(&mut conn, params.limit).await {
//         Ok(creators.into())
//     } else {
//         Ok(Json(vec![]))
//     }
// }

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
struct Screenshot {
    screenshot_id: Uuid,
    #[serde_as(as = "Base64")]
    screenshot_data: Vec<u8>,
}

impl Screenshot {
    async fn insert(conn: &mut PgConnection, model: Self) -> Result<Self, NewRecordError> {
        sqlx::query_as!(
            Screenshot,
            r#"INSERT INTO screenshot (screenshot_id, screenshot_data) VALUES ($1, $2)
            RETURNING screenshot_id, screenshot_data
            "#,
            model.screenshot_id,
            model.screenshot_data
        )
        .fetch_one(conn)
        .await
        .map_err(|_| NewRecordError::Screenshot)
    }
    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Screenshot,
            r#" SELECT screenshot_id, screenshot_data FROM screenshot
            WHERE screenshot_id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }
}

async fn create_screenshot(
    app_state: Extension<ServerState>,
    Json(screenshot): Json<Screenshot>,
) -> Result<Json<Screenshot>, GISSTError> {
    dbg!(&screenshot.screenshot_id);
    let mut conn = app_state.pool.acquire().await?;
    if screenshot.screenshot_id != uuid!("00000000-0000-0000-0000-000000000000") {
        Ok(Json(Screenshot::insert(&mut conn, screenshot).await?))
    } else {
        Ok(Json(
            Screenshot::insert(
                &mut conn,
                Screenshot {
                    screenshot_id: Uuid::new_v4(),
                    screenshot_data: screenshot.screenshot_data,
                },
            )
            .await?,
        ))
    }
}

async fn get_single_screenshot(
    app_state: Extension<ServerState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Screenshot>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Screenshot::get_by_id(&mut conn, id).await?.unwrap()))
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
    Ok(Json(
        Environment::update(&mut conn, environment)
            .await
            .map_err(GISSTError::RecordUpdateError)?,
    ))
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
        Err(GISSTError::RecordCreateError(
            gisst::models::NewRecordError::Image,
        ))
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
    Ok(Json(
        Image::update(&mut conn, image)
            .await
            .map_err(GISSTError::RecordUpdateError)?,
    ))
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

async fn get_instances(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Query(params): Query<GetQueryParams>,
) -> Result<axum::response::Response, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let instances: Vec<Instance> = Instance::get_all(&mut conn, params.limit).await?;
    let accept: Option<String> = parse_header(&headers, "Accept");

    Ok(
        (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
            Html(render!(
                app_state.templates.get_template("instance_listing")?,
                instances => instances
            ))
            .into_response()
        } else if accept
            .as_ref()
            .is_some_and(|hv| hv.contains("application/json"))
        {
            Json(instances).into_response()
        } else {
            Err(GISSTError::Generic)?
        })
        .into_response(),
    )
}

#[derive(Debug, Serialize)]
struct FullInstance {
    info: Instance,
    states: Vec<FileRecordFlatten<State>>,
    replays: Vec<FileRecordFlatten<Replay>>,
    saves: Vec<FileRecordFlatten<Save>>,
}

async fn get_all_for_instance(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<axum::response::Response, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Some(instance) = Instance::get_by_id(&mut conn, id).await? {
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

        let full_instance = FullInstance {
            info: instance,
            states: flattened_states,
            replays: flattened_replays,
            saves: flattened_saves,
        };

        let accept: Option<String> = parse_header(&headers, "Accept");

        Ok(
            (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
                Html(render!(
                    app_state.templates.get_template("instance_all_listing")?,
                    instance => dbg!(full_instance),
                ))
                .into_response()
            } else if accept
                .as_ref()
                .is_some_and(|hv| hv.contains("application/json"))
            {
                Json(full_instance).into_response()
            } else {
                Err(GISSTError::Generic)?
            })
            .into_response(),
        )
    } else {
        Err(GISSTError::Generic)
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
    Path(id): Path<Uuid>,
) -> Result<Json<Object>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Object::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn create_object(
    app_state: Extension<ServerState>,
    Query(object): Query<Object>,
) -> Result<Json<Object>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, object.file_id).await?.is_some() {
        Ok(Json(Object::insert(&mut conn, object).await?))
    } else {
        Err(GISSTError::RecordCreateError(
            gisst::models::NewRecordError::Object,
        ))
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

async fn create_replay(
    app_state: Extension<ServerState>,
    Json(replay): Json<Replay>,
) -> Result<Json<Replay>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, replay.file_id).await?.is_some() {
        if replay.replay_id != uuid!("00000000-0000-0000-0000-000000000000") {
            Ok(Json(Replay::insert(&mut conn, replay).await?))
        } else {
            Ok(Json(
                Replay::insert(
                    &mut conn,
                    Replay {
                        replay_id: Uuid::new_v4(),
                        ..replay
                    },
                )
                .await?,
            ))
        }
    } else {
        Err(GISSTError::RecordCreateError(
            gisst::models::NewRecordError::Replay,
        ))
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
        Err(GISSTError::RecordCreateError(
            gisst::models::NewRecordError::Save,
        ))
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

async fn create_state(
    app_state: Extension<ServerState>,
    Json(state): Json<State>,
) -> Result<Json<State>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, state.file_id).await?.is_some() {
        if state.state_id != uuid!("00000000-0000-0000-0000-000000000000") {
            Ok(Json(State::insert(&mut conn, state).await?))
        } else {
            Ok(Json(
                State::insert(
                    &mut conn,
                    State {
                        state_id: Uuid::new_v4(),
                        ..state
                    },
                )
                .await?,
            ))
        }
    } else {
        Err(GISSTError::RecordCreateError(
            gisst::models::NewRecordError::State,
        ))
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
