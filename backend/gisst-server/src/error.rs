use axum::{
    extract::{multipart::MultipartError, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use gisst::models;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GISSTError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
    #[error("storage error")]
    StorageError(#[from] gisst::storage::StorageError),
    #[error("file not found")]
    FileNotFoundError,
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("record creation error")]
    RecordCreateError(#[from] models::NewRecordError),
    #[error("record creation error")]
    RecordUpdateError(#[from] models::UpdateRecordError),
    #[error("template error")]
    TemplateError,
    #[error("path prefix error")]
    PathPrefixError(#[from] std::path::StripPrefixError),
    #[error("tokio task error")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("auth error")]
    AuthError(#[from] AuthError),
    #[error("generic error")]
    Generic,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("user creation error")]
    UserCreateError,
    #[error("user update error")]
    UserUpdateError,
}

impl IntoResponse for GISSTError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            GISSTError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
            GISSTError::IO(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            GISSTError::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage error"),
            GISSTError::RecordCreateError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "record creation error")
            },
            GISSTError::RecordUpdateError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "record update error")
            },
            GISSTError::PathPrefixError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "file creation error")
            },
            GISSTError::TemplateError => (StatusCode::INTERNAL_SERVER_ERROR, "template error"),
            GISSTError::JoinError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "tokio task error"),
            GISSTError::FileNotFoundError => (StatusCode::NOT_FOUND, "file not found"),
            GISSTError::ReqwestError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "oauth reqwest error")
            },
            GISSTError::AuthError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "auth error"),
            GISSTError::Generic => (StatusCode::INTERNAL_SERVER_ERROR, "generic error"),
        };

        let body = Json(json!({ "error": message }));

        (status, body).into_response()
    }
}

impl From<MultipartError> for GISSTError {
    fn from(_error: MultipartError) -> Self {
        GISSTError::Generic
    }
}
