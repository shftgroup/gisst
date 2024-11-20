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
    Sql(#[from] sqlx::Error),
    #[error("storage error")]
    Storage(#[from] gisst::error::StorageError),
    #[error("file not found")]
    FileNotFound,
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("error with record SQL manipulation")]
    RecordManipulation(#[from] gisst::error::RecordSQLError),
    #[error("{} record with uuid {} is missing", .table, .uuid)]
    RecordMissing { table: ErrorTable, uuid: Uuid },
    #[error("path prefix error")]
    PathPrefix(#[from] std::path::StripPrefixError),
    #[error("tokio task error")]
    Join(#[from] tokio::task::JoinError),
    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("error rendering minijinja template")]
    MiniJinja(#[from] minijinja::Error),
    #[error("user login error")]
    AuthUserSerdeLogin(#[from] serde_json::Error),
    #[error("session error")]
    AuthSession(#[from] axum_login::tower_sessions::session::Error),
    #[error("auth backend error")]
    AuthBackend(#[from] AuthError),
    #[error("auth system error")]
    AuthBackendSystem(#[from] axum_login::Error<crate::auth::AuthBackend>),
    #[error("incorrect mimetype for request")]
    MimeType,
    #[error("user not logged in")]
    AuthUserNotAuthenticated,
    #[error("error linking uuid: {} to {} reference", .table, .uuid)]
    RecordLinking { table: ErrorTable, uuid: Uuid },
    #[error("multipart request error")]
    Multipart(#[from] MultipartError),
    #[error("filesystem listing error")]
    FSList(#[from] gisst::error::FSListError),
    #[error("subobject access error")]
    Subobject(String),
    #[error("state required for clone command")]
    StateRequired,
    #[error("clone execution error")]
    V86Clone(#[from] gisst::error::V86CloneError),
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
            GISSTError::Sql(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
            GISSTError::IO(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            GISSTError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage error"),
            GISSTError::RecordManipulation(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "record manipulation error",
            ),
            GISSTError::RecordLinking { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "record creation error")
            }
            GISSTError::RecordMissing { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "record update error")
            }
            GISSTError::PathPrefix(_) => (StatusCode::INTERNAL_SERVER_ERROR, "file creation error"),
            GISSTError::Join(_) => (StatusCode::INTERNAL_SERVER_ERROR, "tokio task error"),
            GISSTError::FileNotFound => (StatusCode::NOT_FOUND, "file not found"),
            GISSTError::Reqwest(_) => (StatusCode::INTERNAL_SERVER_ERROR, "oauth reqwest error"),
            GISSTError::AuthUserSerdeLogin(_) => (StatusCode::INTERNAL_SERVER_ERROR, "auth error"),
            GISSTError::AuthUserNotAuthenticated => {
                (StatusCode::INTERNAL_SERVER_ERROR, "auth error")
            }
            GISSTError::MiniJinja(_) => (StatusCode::INTERNAL_SERVER_ERROR, "minijinja error"),
            GISSTError::MimeType => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "incompatible mimetype for request",
            ),
            GISSTError::Multipart(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "multipart request error")
            }
            GISSTError::FSList(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "filesystem listing error",
            ),
            GISSTError::Subobject(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "subobject access error")
            }
            GISSTError::StateRequired => (StatusCode::BAD_REQUEST, "need a state to make a clone"),
            GISSTError::V86Clone(_) => (StatusCode::INTERNAL_SERVER_ERROR, "v86 clone failed"),
            GISSTError::Unreachable => (StatusCode::INTERNAL_SERVER_ERROR, "uh oh error"),
            GISSTError::AuthSession(_) => (StatusCode::INTERNAL_SERVER_ERROR, "auth session error"),
            GISSTError::AuthBackend(_) => (StatusCode::INTERNAL_SERVER_ERROR, "auth backend error"),
            GISSTError::AuthBackendSystem(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "auth backend internal system error",
            ),
        };

        let body = Html(
            error_template
                .render(context!(status_code => status.clone().to_string(), message => message))
                .unwrap(),
        );

        (status, body).into_response()
    }
}

#[derive(thiserror::Error)]
pub enum AuthError {
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("csrf error")]
    CsrfMissing,
    #[error("user not allowed to access resource")]
    UserNotPermitted,
    #[error("token request error")]
    TokenRequest(
        #[from]
        oauth2::RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ),
    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("missing user auth profile information: {}", .field)]
    MissingProfileInfo { field: String },
    #[error("error with record SQL manipulation")]
    RecordManipulation(#[from] gisst::error::RecordSQLError),
}
impl Debug for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?;
        if let Some(source) = self.source() {
            writeln!(f, "Caused by:\n\t{}", source)?;
        }
        Ok(())
    }
}
