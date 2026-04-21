use crate::{error::ServerError, server::ServerState};
use axum::{
    Extension,
    extract::{Json, Path},
};
use gisst::models::Core;
use uuid::Uuid;
// route: platform=, filename=, hash=

// returns a struct with {work:, instance_id:} (second may be null if no instance yet exists)

#[expect(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct LookupParams {
    platform: String,
    filename: String,
    hash: String,
}
#[derive(Debug, serde::Serialize)]
pub struct LookupResult {
    work_name: String,
    work_version: String,
    work_platform: String,
    work_derived_from: Option<Uuid>,
    instance_id: Option<Uuid>,
}

pub async fn lookup_work(
    _app_state: Extension<ServerState>,
    Json(_params): Json<LookupParams>,
) -> Result<Json<LookupResult>, ServerError> {
    // pick rdb based on platform
    // use find_entry logic from ingest (move into gisst?)
    // alreadyhave -> return any work with an instance linked to this file
    // notinrdb -> return error
    // inrdb(rval) -> rval->name, rval->rom_name as version, platform, not derived from anything
    Err(ServerError::NotYetImplemented)
}

#[derive(Debug, serde::Serialize)]
pub struct CoreList {
    cores: Vec<Core>,
}

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
