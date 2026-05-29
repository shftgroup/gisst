use crate::{error::ServerError, server::ServerState};
use axum::{
    Extension,
    extract::{Json, Path, Query},
};
use gisst::models::{Core, InstanceWork, RDBWork};
use uuid::Uuid;
// route: platform=, filename=, hash=

// returns a struct with {work:, instance_id:} (second may be null if no instance yet exists)

#[derive(Debug, serde::Deserialize)]
pub struct LookupParams {
    platform: String,
    filename: String,
    md5: String,
}
#[derive(Debug, serde::Serialize)]
pub struct LookupResult {
    work_name: String,
    work_version: String,
    work_platform: String,
    work_derived_from: Option<Uuid>,
    instance_id: Option<Uuid>,
}

#[tracing::instrument(skip(app_state))]
pub async fn lookup_work(
    app_state: Extension<ServerState>,
    Query(LookupParams {
        platform,
        filename,
        md5,
    }): Query<LookupParams>,
) -> Result<Json<LookupResult>, ServerError> {
    use futures::stream::StreamExt;
    let mut conn = app_state.pool.acquire().await?;
    // try looking for this file in the objectlink/file tables first
    if let Some(InstanceWork {
        work_name,
        work_version,
        work_platform,
        work_derived_from,
        instance_id,
        ..
    }) = InstanceWork::get_for_file_hash(&mut conn, &md5)
        .next()
        .await
    {
        // return work of any instance that owns this file, and the instance
        Ok(Json(LookupResult {
            work_name,
            work_version,
            work_platform,
            work_derived_from,
            instance_id: Some(instance_id),
        }))
    } else if let Some(RDBWork {
        platform,
        name,
        rom_name,
        ..
    }) = RDBWork::lookup(&mut conn, &platform, &filename, Some(&md5)).await?
    {
        // return work info and no instance id
        Ok(Json(LookupResult {
            work_name: name,
            work_version: rom_name.unwrap_or_default(),
            work_platform: platform,
            work_derived_from: None,
            instance_id: None,
        }))
    } else {
        Err(ServerError::FileNotFound)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CoreList {
    cores: Vec<Core>,
}

#[tracing::instrument(skip(app_state))]
pub async fn get_cores(
    app_state: Extension<ServerState>,
    maybe_corename: Option<Path<String>>,
) -> Result<Json<CoreList>, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    Ok(Json(CoreList {
        cores: if let Some(corename) = maybe_corename {
            Core::get_versions(&mut conn, &corename).await?
        } else {
            Core::get_latest(&mut conn).await?
        },
    }))
}
