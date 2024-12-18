use axum::{
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use gisst::error::ErrorTable;
use minijinja::{context, Environment};
use std::error::Error;
use std::fmt::Debug;
use tracing::error;
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum GISSTError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
    #[error("storage error")]
    StorageError(#[from] gisst::error::StorageError),
    #[error("file not found")]
    FileNotFoundError,
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("error with record SQL manipulation")]
    RecordManipulationError(#[from] gisst::error::RecordSQLError),
    #[error("{} record with uuid {} is missing", .table, .uuid)]
    RecordMissingError { table: ErrorTable, uuid: Uuid },
    #[error("path prefix error")]
    PathPrefixError(#[from] std::path::StripPrefixError),
    #[error("tokio task error")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("error rendering minijinja template")]
    MiniJinjaError(#[from] minijinja::Error),
    #[error("user login error")]
    AuthUserSerdeLoginError(#[from] serde_json::Error),
    #[error("missing user auth profile information: {}", .field)]
    AuthMissingProfileInfoError { field: String },
    #[error("incorrect mimetype for request")]
    MimeTypeError,
    #[error("user not logged in")]
    AuthUserNotAuthenticatedError,
    #[error("user not permitted to access this resource")]
    AuthUserNotPermittedError,
    #[error("token response error")]
    AuthTokenResponseError,
    #[error("error linking uuid: {} to {} reference", .table, .uuid)]
    RecordLinkingError { table: ErrorTable, uuid: Uuid },
    #[error("multipart request error")]
    MultipartError(#[from] MultipartError),
    #[error("filesystem listing error")]
    FSListError(#[from] gisst::error::FSListError),
    #[error("subobject access error")]
    SubobjectError(String),
    #[error("state required for clone command")]
    StateRequiredError,
    #[error("clone execution error")]
    V86CloneError(#[from] gisst::error::V86CloneError),
    #[error("this should not be reachable!")]
    Unreachable,
}

impl Debug for GISSTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?;
        if let Some(source) = self.source() {
            writeln!(f, "Caused by:\n\t{}", source)?;
        }
        Ok(())
    }
}

impl IntoResponse for GISSTError {
    fn into_response(self) -> Response {
        let mut env = Environment::new();
        env.add_template("error.html", r#"

        <div id="embedExample" style="width:320px; height: 280px"></div>
        <h1>{{status_code }}</h1>
        <p>Oops! We've encountered a {{ status_code }}, please go back to the previous page while we sort this out.</p>
        <h2> Cryptic Error Message </h2>
        <p>{{ message }}</p>
        <link rel="stylesheet" href="/embed/style.css">
        <script type="module">
            import {embed} from '/embed/embed.js';
            embed("https://gisst.pomona.edu/data/62f78345-b4b0-458d-a6ea-5c08724a6415?state=e32b9c0f-f56e-4a84-b2e6-e4996a82e35a", document.getElementById("embedExample"));
        </script>
        "#).unwrap();
        error!("{self}");
        let mut err: &(dyn std::error::Error + 'static) = &self;
        while let Some(source) = err.source() {
            error!("Caused by: {source}");
            err = source;
        }
        let error_template = env.get_template("error.html").unwrap();
        let (status, message) = match self {
            GISSTError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
            GISSTError::IO(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            GISSTError::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage error"),
            GISSTError::RecordManipulationError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "record manipulation error",
            ),
            GISSTError::RecordLinkingError { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "record creation error")
            }
            GISSTError::RecordMissingError { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "record update error")
            }
            GISSTError::PathPrefixError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "file creation error")
            }
            GISSTError::JoinError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "tokio task error"),
            GISSTError::FileNotFoundError => (StatusCode::NOT_FOUND, "file not found"),
            GISSTError::ReqwestError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "oauth reqwest error")
            }
            GISSTError::AuthUserSerdeLoginError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "auth error")
            }
            GISSTError::AuthMissingProfileInfoError { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "auth error, missing profile field",
            ),
            GISSTError::AuthUserNotAuthenticatedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "auth error")
            }
            GISSTError::AuthUserNotPermittedError => {
                (StatusCode::FORBIDDEN, "user not permitted error")
            }
            GISSTError::AuthTokenResponseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "oauth token response error",
            ),
            GISSTError::MiniJinjaError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "minijinja error"),
            GISSTError::MimeTypeError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "incompatible mimetype for request",
            ),
            GISSTError::MultipartError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "multipart request error")
            }
            GISSTError::FSListError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "filesystem listing error",
            ),
            GISSTError::SubobjectError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "subobject access error")
            }
            GISSTError::StateRequiredError => {
                (StatusCode::BAD_REQUEST, "need a state to make a clone")
            }
            GISSTError::V86CloneError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "v86 clone failed"),
            GISSTError::Unreachable => (StatusCode::INTERNAL_SERVER_ERROR, "uh oh error"),
        };

        let body = Html(
            error_template
                .render(context!(status_code => status.clone().to_string(), message => message))
                .unwrap(),
        );

        (status, body).into_response()
    }
}
