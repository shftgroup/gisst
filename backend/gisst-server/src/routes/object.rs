use crate::auth::AuthBackend;
use crate::server::BASE_URL;
use crate::{auth, error::ServerError, server::ServerState, utils::parse_header};
use axum::{
    Extension, Router,
    extract::{Json, Path},
    http::header::HeaderMap,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use axum_login::login_required;
use gisst::models::{File, Object};
use gisst::{error::Table, inc_metric};
use minijinja::context;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/{id}", get(get_single_object))
        .route("/{id}/{*path}", get(get_subobject))
        .route("/create", post(create_object))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn get_single_object(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
) -> Result<axum::response::Response, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let mut conn = app_state.pool.acquire().await?;

    let object = Object::get_by_id(&mut conn, id)
        .await?
        .ok_or(ServerError::RecordMissing {
            table: Table::Object,
            uuid: id,
        })?;
    let file =
        File::get_by_id(&mut conn, object.file_id)
            .await?
            .ok_or(ServerError::RecordMissing {
                table: Table::File,
                uuid: object.file_id,
            })?;

    let accept: Option<String> = parse_header(&headers, "Accept");

    Ok(
        (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
            use gisst::fslist::{file_to_path, is_disk_image, recursive_listing};
            let object_page = app_state.templates.get_template("object_listing.html")?;
            // TODO reuse cookie instead of reloading every time
            let path = file_to_path(&app_state.root_storage_path, &file);
            let directory = if is_disk_image(&path) {
                inc_metric!(conn, fslist_recursive_listing, 1, path = path.to_str());
                let image = std::fs::File::open(path)?;
                recursive_listing(image)?
            } else {
                vec![]
            };
            Html(object_page.render(context!(
                base_url => BASE_URL.get(),
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
            Err(ServerError::MimeType)?
        })
        .into_response(),
    )
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn get_subobject(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path((id, subpath)): Path<(Uuid, String)>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
) -> Result<axum::response::Response, ServerError> {
    use gisst::fslist::{file_to_path, get_file_at_path, is_disk_image};
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );

    let mut conn = app_state.pool.acquire().await?;

    let object = Object::get_by_id(&mut conn, id)
        .await?
        .ok_or(ServerError::RecordMissing {
            table: Table::Object,
            uuid: id,
        })?;
    let file =
        File::get_by_id(&mut conn, object.file_id)
            .await?
            .ok_or(ServerError::RecordMissing {
                table: Table::File,
                uuid: object.file_id,
            })?;
    let path = file_to_path(&app_state.root_storage_path, &file);
    let (mime, data) = {
        let subpath = subpath.clone();
        let is_disk = is_disk_image(&path);
        if is_disk {
            inc_metric!(
                conn,
                fslist_get_file_at_path,
                1,
                path = path.to_str(),
                subpath = &subpath
            );
        }
        tokio::task::spawn_blocking(move || {
            if is_disk {
                get_file_at_path(std::fs::File::open(path)?, std::path::Path::new(&subpath))
                    .map_err(ServerError::from)
            } else {
                Err(ServerError::Subobject(format!("{id}:{subpath}")))
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
                    .ok_or(ServerError::Subobject(
                        "can't download empty thing".to_string()
                    ))?
                    .to_string_lossy()
            ),
        ),
    ];
    Ok((headers, data).into_response())
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateObject {
    pub file_id: Uuid,
    pub object_description: Option<String>,
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn create_object(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<auth::AuthBackend>,
    Json(object): Json<CreateObject>,
) -> Result<Json<Object>, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let creator_id = auth
        .user
        .ok_or(ServerError::AuthUserNotAuthenticated)?
        .creator_id;

    let mut conn = app_state.pool.acquire().await?;

    if let Some(file) = File::get_by_id(&mut conn, object.file_id).await? {
        tracing::info!("Inserting object {object:?}");
        Ok(Json(
            Object::insert(
                &mut conn,
                Object {
                    object_id: Uuid::new_v4(),
                    file_id: object.file_id,
                    object_description: object
                        .object_description
                        .or_else(|| Some(file.file_filename.clone())),
                    //creator_id,
                    created_on: chrono::Utc::now(),
                },
            )
            .await?,
        ))
    } else {
        Err(ServerError::RecordMissing {
            table: Table::File,
            uuid: object.file_id,
        })?
    }
}
