pub mod models;
pub mod storage;
use axum::{
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    extract::{
        Json,
        multipart::MultipartError,
    },
};
use thiserror::Error;
use serde_json::json;

#[derive(Debug, Error)]
pub enum GISSTError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
    #[error("storage error")]
    StorageError(#[from] std::io::Error),
    #[error("record creation error")]
    RecordCreateError(#[from] models::NewRecordError),
    #[error("record creation error")]
    RecordUpdateError(#[from] models::UpdateRecordError),
    #[error("template error")]
    TemplateError,
    #[error("path prefix error")]
    PathPrefixError(#[from] std::path::StripPrefixError),
    #[error("generic error")]
    Generic,
}

impl IntoResponse for GISSTError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            GISSTError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
            GISSTError::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage error"),
            GISSTError::RecordCreateError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "record creation error"),
            GISSTError::RecordUpdateError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "record update error"),
            GISSTError::PathPrefixError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "file creation error"),
            GISSTError::TemplateError => (StatusCode::INTERNAL_SERVER_ERROR, "template error"),
            GISSTError::Generic => (StatusCode::INTERNAL_SERVER_ERROR, "generic error"),
        };

        let body = Json(json!({"error": message}));

        (status, body).into_response()
    }
}

impl From<MultipartError> for GISSTError {
    fn from(error: MultipartError) -> Self {
        GISSTError::Generic
    }
}
