use crate::{
    server::ServerState,
    utils::{check_header, get_metadata, parse_header},
};

use crate::error::GISSTError;
use bytes::Bytes;
use gisst::models::File as GFile;
use uuid::Uuid;

use axum::{
    extract::Path,
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};

use axum_macros::debug_handler;
use gisst::storage::{FileInformation, PendingUpload, StorageHandler};

pub async fn tus_head(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Response, GISSTError> {
    let tus_resumable: Option<String> = parse_header(&headers, "Tus-Resumable");
    if tus_resumable.is_none() || tus_resumable.unwrap() != "1.0.0" {
        return Ok(([("Tus-Version", "1.0.0")], StatusCode::PRECONDITION_FAILED).into_response());
    }

    let mut conn = app_state.pool.acquire().await?;

    if let Some(file) = GFile::get_by_id(&mut conn, id).await? {
        return Ok(([
            ("Tus-Resumable", "1.0.0"),
            ("Upload-Offset", &file.file_size.to_string()),
            ("Cache-Control", "no-store"),
        ])
        .into_response());
    }

    if let Some(pu) = app_state.pending_uploads.read().unwrap().get(&id) {
        Ok(([
            ("Tus-Resumable", "1.0.0"),
            ("Upload-Offset", &pu.offset.to_string()),
            ("Upload-Length", &pu.length.to_string()),
            ("Cache-Control", "no-store"),
        ])
        .into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            format!("Unable to locate pending upload with id {}", id),
        )
            .into_response())
    }
}

#[debug_handler]
pub async fn tus_patch(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    body: Bytes,
) -> Result<axum::response::Response, GISSTError> {
    //Check for correct headers
    let check_content_type = |val: &str| val == "application/offset+octet-stream";
    if !check_header(&headers, "Content-Type", check_content_type) {
        return Ok((StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unknown content-type.").into_response());
    }

    let offset: Option<usize> = parse_header(&headers, "Upload-Offset");
    if offset.is_none() {
        return Ok((StatusCode::UNSUPPORTED_MEDIA_TYPE, "No offset provided.").into_response());
    }
    let offset = offset.unwrap();

    // Check if file upload has completed, if so, send offset equal to length
    let mut conn = app_state.pool.acquire().await?;
    if let Some(f) = GFile::get_by_id(&mut conn, id).await? {
        return Ok((
            [
                ("Upload-Offset", f.file_size.to_string()),
                ("Cache-Control", "no-store".to_string()),
            ],
            StatusCode::NO_CONTENT,
        )
            .into_response());
    }

    // Check that file upload exists
    if app_state.pending_uploads.read().unwrap().get(&id).is_none() {
        return Err(GISSTError::FileNotFound);
    }

    let mut pu_offset = app_state
        .pending_uploads
        .read()
        .unwrap()
        .get(&id)
        .unwrap()
        .offset;
    let pu_length = app_state
        .pending_uploads
        .read()
        .unwrap()
        .get(&id)
        .unwrap()
        .length;

    // Check that offset is correct
    if pu_offset != offset {
        return Ok((
            StatusCode::CONFLICT,
            format!(
                "Client offset ({}) does not match server offset ({})",
                pu_offset, offset
            ),
        )
            .into_response());
    }

    // Check that length is not equal to offset (otherwise there are no bytes to write)
    if pu_offset >= pu_length {
        return Ok((
            StatusCode::FORBIDDEN,
            "Upload-Offset is greater than or equal to Upload-Length.",
        )
            .into_response());
    }

    // Check that upload size is not greater than chunk size
    if body.len() > app_state.default_chunk_size {
        return Ok((
            StatusCode::FORBIDDEN,
            format!(
                "File is larger than maximum chunk size, ({}) > ({})",
                body.len(),
                app_state.default_chunk_size
            ),
        )
            .into_response());
    }

    // Append chunk to file
    let file_info = app_state
        .pending_uploads
        .read()
        .unwrap()
        .get(&id)
        .unwrap()
        .file_information
        .clone();
    StorageHandler::add_bytes_to_file(&app_state.temp_storage_path, &file_info, body.clone())
        .await?;
    app_state
        .pending_uploads
        .write()
        .unwrap()
        .get_mut(&id)
        .unwrap()
        .offset += body.len();
    pu_offset += body.len();

    // Check if upload is complete and clean up
    if pu_offset == pu_length {
        println!(
            "Got to rename file with the following, temp_path: {}, root_path:{}, file_info: {:?}",
            &app_state.temp_storage_path, &app_state.root_storage_path, &file_info
        );
        StorageHandler::rename_file_from_temp_to_storage(
            &app_state.root_storage_path,
            &app_state.temp_storage_path,
            &file_info,
        )
        .await?;
        GFile::insert(
            &mut conn,
            GFile {
                file_id: id,
                file_hash: file_info.file_hash.clone(),
                file_filename: file_info.source_filename.clone(),
                file_source_path: file_info.source_path.clone(),
                file_dest_path: file_info.dest_path.clone(),
                file_size: pu_length as i64,
                created_on: chrono::Utc::now(),
            },
        )
        .await?;
        app_state.pending_uploads.write().unwrap().remove(&id);
    }

    Ok((
        [
            ("Upload-Offset", pu_offset.to_string()),
            ("Cache-Control", "no-store".to_string()),
        ],
        StatusCode::NO_CONTENT,
    )
        .into_response())
}

pub async fn tus_creation(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
) -> Result<Response, GISSTError> {
    // Get file length header information
    // We are not allowing deferred length at this time
    let length: Option<usize> = parse_header(&headers, "Upload-Length");

    if length.is_none() {
        return Ok((StatusCode::BAD_REQUEST, "Upload-Length header is required.").into_response());
    }

    let metadata = get_metadata(&headers);

    // Upload-Metadata must supply a filename for the upload
    if metadata.is_none() {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Upload-Metadata header is required.",
        )
            .into_response());
    }

    let metadata = metadata.unwrap();
    for key in ["filename", "hash"].iter() {
        if !metadata.contains_key(*key) {
            return Ok((
                StatusCode::BAD_REQUEST,
                format!(
                    "Upload-Metadata header must contain a value for '{}' key.",
                    key
                ),
            )
                .into_response());
        }
    }

    // Initialize pending upload
    let new_uuid = Uuid::new_v4();
    let filename = metadata.get("filename").unwrap();
    let hash = metadata.get("hash").unwrap();
    let mut dest_path = StorageHandler::split_uuid_to_path_buf(new_uuid, app_state.folder_depth);
    dest_path.push(StorageHandler::get_dest_filename(hash, filename.as_str()));

    let file_info = FileInformation {
        source_filename: filename.to_string(),
        source_path: String::new(),
        dest_filename: StorageHandler::get_dest_filename(hash, filename),
        dest_path: dest_path.to_string_lossy().to_string(),
        file_hash: hash.to_string(),
    };

    // Create temp file for PATCH
    StorageHandler::create_temp_file(&app_state.temp_storage_path, &file_info).await?;

    // Add pending upload to queue
    let _ = &app_state.pending_uploads.write().unwrap().insert(
        new_uuid,
        PendingUpload {
            file_information: file_info,
            length: length.unwrap(),
            offset: 0,
        },
    );

    // Construct header response with id for resource/:id url
    Ok((
        [
            ("Tus-Resumable", "1.0.0"),
            ("Location", &format!("/resources/{}", new_uuid)),
        ],
        StatusCode::CREATED,
    )
        .into_response())
}
