use crate::server::ServerState;
use axum::http::StatusCode;
use axum::{
    extract::{Json, Path, Query},
    routing::{get, post},
    Extension, Router,
};
use gisstlib::{
    models::{DBModel, Environment, Image, Instance, Object, Work},
    GISSTError,
};
use serde::{Deserialize};
use uuid::Uuid;
use gisstlib::models::File;


// Nested Router structs for easier reading and manipulation
// pub fn creator_router() -> Router {
//     Router::new()
//         .route("/", get(get_creators))
//         .route("/create", post(create_creator))
//         .route("/:id", get(get_single_creator)
//             .put(edit_creator)
//             .delete(delete_creator))
// }

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

// ENVIRONMENT method handlers
#[derive(Deserialize)]
struct EnvironmentsGetQueryParams {
    limit: Option<i64>,
}

async fn get_environments(
    app_state: Extension<ServerState>,
    Query(params): Query<EnvironmentsGetQueryParams>,
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
#[derive(Deserialize)]
struct ImagesGetQueryParams {
    limit: Option<i64>,
}

async fn get_images(
    app_state: Extension<ServerState>,
    Query(params): Query<ImagesGetQueryParams>,
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
        Err(GISSTError::RecordCreateError(gisstlib::models::NewRecordError::Image))
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
#[derive(Deserialize)]
struct InstancesGetQueryParams {
    limit: Option<i64>,
}

async fn get_instances(
    app_state: Extension<ServerState>,
    Query(params): Query<InstancesGetQueryParams>,
) -> Result<Json<Vec<Instance>>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    if let Ok(instances) = Instance::get_all(&mut conn, params.limit).await {
        Ok(instances.into())
    } else {
        Ok(Json(vec![]))
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
#[derive(Deserialize)]
struct ObjectsGetQueryParams {
    limit: Option<i64>,
}

async fn get_objects(
    app_state: Extension<ServerState>,
    Query(params): Query<ObjectsGetQueryParams>,
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

async fn create_object(
    app_state: Extension<ServerState>,
    Query(object): Query<Object>,
) -> Result<Json<Object>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;

    if File::get_by_id(&mut conn, object.file_id).await?.is_some() {
        Ok(Json(Object::insert(&mut conn, object).await?))
    } else {
        Err(GISSTError::RecordCreateError(gisstlib::models::NewRecordError::Object))
    }
}

// WORK method handlers
#[derive(Deserialize)]
struct WorksGetQueryParams {
    limit: Option<i64>,
}
//
async fn get_works(
    app_state: Extension<ServerState>,
    Query(params): Query<WorksGetQueryParams>,
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
