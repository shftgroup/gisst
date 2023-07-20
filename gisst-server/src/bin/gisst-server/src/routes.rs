use crate::server::ServerState;
use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::{
    extract::{Json, Path, Query},
    routing::{get, post},
    Extension, Router,
};
use bytes::Bytes;
use gisstlib::{
    models::{DBHashable, DBModel, Environment, Image, Instance, Object, Work},
    storage::StorageHandler,
    GISSTError,
};
use serde::{Deserialize};
use std::sync::Arc;
use uuid::Uuid;


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
    app_state: Extension<Arc<ServerState>>,
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
    app_state: Extension<Arc<ServerState>>,
    Query(environment): Query<Environment>,
) -> Result<Json<Environment>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Environment::insert(&mut conn, environment).await?))
}



async fn get_single_environment(
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Environment>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Environment::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_environment(
    app_state: Extension<Arc<ServerState>>,
    Query(environment): Query<Environment>,
) -> Result<Json<Environment>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(
        Environment::update(&mut conn, environment)
            .await
            .map_err(GISSTError::RecordUpdateError)?,
    ))
}

async fn delete_record<T: DBModel>(
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    T::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}

async fn delete_file_record<T:DBModel + DBHashable>(
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {

    Ok(StatusCode::OK)
}

async fn delete_environment(
    app_state: Extension<Arc<ServerState>>,
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
    app_state: Extension<Arc<ServerState>>,
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
    app_state: Extension<Arc<ServerState>>,
    mut multipart: Multipart,
) -> Result<Json<Image>, GISSTError> {
    let mut filename: Option<String> = None;
    let mut data: Option<Bytes> = None;
    let mut hash: Option<String> = None;
    let mut image_description: Option<String> = None;
    let mut image_config_json: Option<sqlx::types::JsonValue> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "image_file" => {
                filename = Some(field.file_name().unwrap().to_string());
                data = Some(field.bytes().await.unwrap());
                hash = Some(StorageHandler::get_md5_hash(&data.clone().unwrap()))
            }
            "image_description" => image_description = Some(field.text().await.unwrap()),
            "image_config_json" => {
                image_config_json = Some(serde_json::Value::from(&*field.bytes().await.unwrap()))
            }
            _ => (),
        }
    }

    let mut conn = app_state.pool.acquire().await?;

    if hash.is_some()
        && Image::get_by_hash(&mut conn, hash.as_ref().unwrap())
            .await?
            .is_none()
    {
        let new_uuid = Uuid::new_v4();
        if let Ok(file_info) = app_state
            .storage
            .write_file_to_uuid_folder(new_uuid, &filename.unwrap(), &data.unwrap())
            .await
        {
            if let Ok(image) = Image::insert(
                &mut conn,
                Image {
                    image_id: new_uuid,
                    image_hash: hash.unwrap(),
                    image_description,
                    image_filename: file_info.source_filename,
                    image_source_path: file_info.source_path,
                    image_dest_path: file_info.dest_path,
                    created_on: None,
                    image_config: image_config_json,
                },
            )
            .await
            {
                return Ok(Json(image));
            } else {
                app_state
                    .storage
                    .delete_file_with_uuid(new_uuid, &file_info.dest_filename)
                    .await?;
            }
        }
    }
    Err(GISSTError::Generic)
}

async fn get_single_image(
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Image>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Image::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_image(
    app_state: Extension<Arc<ServerState>>,
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
    app_state: Extension<Arc<ServerState>>,
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
    app_state: Extension<Arc<ServerState>>,
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
    app_state: Extension<Arc<ServerState>>,
    Query(instance): Query<Instance>,
) -> Result<Json<Instance>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Instance::insert(&mut conn, instance).await?))
}

async fn get_single_instance(
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Instance>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Instance::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_instance(
    app_state: Extension<Arc<ServerState>>,
    Query(instance): Query<Instance>,
) -> Result<Json<Instance>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Instance::update(&mut conn, instance).await?))
}

async fn delete_instance(
    app_state: Extension<Arc<ServerState>>,
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
    app_state: Extension<Arc<ServerState>>,
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
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Object>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Object::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn edit_object(
    app_state: Extension<Arc<ServerState>>,
    Query(object): Query<Object>,
) -> Result<Json<Object>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Object::update(&mut conn, object).await?))
}

async fn delete_object(
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Object::delete_by_id(&mut conn, id).await?;
    Object::delete_object_instance_links_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}

async fn create_object(
    app_state: Extension<Arc<ServerState>>,
    mut multipart: Multipart,
) -> Result<Json<Object>, GISSTError> {
    let mut filename: Option<String> = None;
    let mut data: Option<Bytes> = None;
    let mut hash: Option<String> = None;
    let mut object_description: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "object_file" => {
                filename = Some(field.file_name().unwrap().to_string());
                data = Some(field.bytes().await.unwrap());
                hash = Some(StorageHandler::get_md5_hash(&data.clone().unwrap()))
            }
            "object_description" => object_description = Some(field.text().await.unwrap()),
            _ => (),
        }
    }

    let mut conn = app_state.pool.acquire().await?;

    if hash.is_some()
        && Object::get_by_hash(&mut conn, hash.as_ref().unwrap())
            .await?
            .is_none()
    {
        let new_uuid = Uuid::new_v4();
        if let Ok(file_info) = app_state
            .storage
            .write_file_to_uuid_folder(new_uuid, &filename.unwrap(), &data.unwrap())
            .await
        {
            if let Ok(object) = Object::insert(
                &mut conn,
                Object {
                    object_id: new_uuid,
                    object_hash: hash.unwrap(),
                    object_description,
                    object_filename: file_info.source_filename.to_string(),
                    object_source_path: file_info.source_filename.to_string(),
                    object_dest_path: file_info.dest_path,
                    created_on: None,
                },
            )
            .await
            {
                return Ok(Json(object));
            } else {
                app_state
                    .storage
                    .delete_file_with_uuid(new_uuid, &file_info.dest_filename)
                    .await?;
            }
        }
    }
    Err(GISSTError::Generic)
}

// WORK method handlers
#[derive(Deserialize)]
struct WorksGetQueryParams {
    limit: Option<i64>,
}
//
async fn get_works(
    app_state: Extension<Arc<ServerState>>,
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
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Work>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Work::get_by_id(&mut conn, id).await?.unwrap()))
}

async fn create_work(
    app_state: Extension<Arc<ServerState>>,
    Query(work): Query<Work>,
) -> Result<Json<Work>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Work::insert(&mut conn, work).await?))
}

async fn edit_work(
    app_state: Extension<Arc<ServerState>>,
    Query(work): Query<Work>,
) -> Result<Json<Work>, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(Work::update(&mut conn, work).await?))
}

async fn delete_work(
    app_state: Extension<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    Work::delete_by_id(&mut conn, id).await?;
    Ok(StatusCode::OK)
}
