use axum::{
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use gisst::error::Table;
use minijinja::{Environment, context};
use std::error::Error;
use std::fmt::Debug;
use tracing::error;
use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
#[derive(thiserror::Error)]
pub enum ServerError {
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("search index error")]
    SearchIndex(#[from] gisst::error::SearchIndex),
    #[error("search error")]
    Search(#[from] gisst::error::Search),
    #[error("insert error")]
    Insert(#[from] gisst::error::Insert),
    #[error("storage error")]
    Storage(#[from] gisst::error::Storage),
    #[error("file not found")]
    FileNotFound,
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("error with record SQL manipulation")]
    RecordManipulation(#[from] gisst::error::RecordSQL),
    #[error("{} record with uuid {} is missing", .table, .uuid)]
    RecordMissing { table: Table, uuid: Uuid },
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
    RecordLinking { table: Table, uuid: Uuid },
    #[error("multipart request error")]
    Multipart(#[from] MultipartError),
    #[error("filesystem listing error")]
    FSList(#[from] gisst::error::FSList),
    #[error("subobject access error")]
    Subobject(String),
    #[error("state required for clone command")]
    StateRequired,
    #[error("clone execution error")]
    V86Clone(#[from] gisst::error::V86Clone),
    #[error("TUS upload too big to store metadata in postgres int")]
    UploadTooBig(std::num::TryFromIntError),
    #[error("this should not be reachable!")]
    Unreachable,
}

impl Debug for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self}")?;
        if let Some(source) = self.source() {
            writeln!(f, "Caused by:\n\t{source}")?;
        }
        Ok(())
    }
}

#[allow(clippy::too_many_lines)]
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let mut env = Environment::new();
        // This unwrap is safe because errors can't happen before the serverstate is initialized
        env.add_template("error.html", r#"
<!DOCTYPE "html"/>
<html class="h-100" lang="en">
<head>
  <title>GISST Error</title>
</head>
<body>
  <script type="module" src="{{ base_url }}/embed/embed.js"></script>
  <gisst-embed src="https://gisst.pomona.edu/data/62f78345-b4b0-458d-a6ea-5c08724a6415?state=e32b9c0f-f56e-4a84-b2e6-e4996a82e35a" controller="off" width="320px" height="280px"></gisst-embed>
  <h1>{{ status_code }}</h1>
  <p>Oops! We've encountered a {{ status_code }}, please go back to the previous page while we sort this out.</p>
  <h2> Cryptic Error Message </h2>
  <p>{{ message }}</p>
</body>
        "#).unwrap();
        error!("{self}");
        let mut err: &(dyn std::error::Error + 'static) = &self;
        while let Some(source) = err.source() {
            error!("Caused by: {source}");
            err = source;
        }
        let error_template = env.get_template("error.html").unwrap();
        let (status, message) = match self {
            ServerError::Sql(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
            ServerError::IO(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            ServerError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage error"),
            ServerError::RecordManipulation(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "record manipulation error",
            ),
            ServerError::UploadTooBig(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "file upload error")
            }
            ServerError::RecordLinking { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "record creation error")
            }
            ServerError::RecordMissing { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "record update error")
            }
            ServerError::PathPrefix(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "file creation error")
            }
            ServerError::Join(_) => (StatusCode::INTERNAL_SERVER_ERROR, "tokio task error"),
            ServerError::FileNotFound => (StatusCode::NOT_FOUND, "file not found"),
            ServerError::Reqwest(_) => (StatusCode::INTERNAL_SERVER_ERROR, "oauth reqwest error"),
            ServerError::AuthUserSerdeLogin(_) => (StatusCode::INTERNAL_SERVER_ERROR, "auth error"),
            ServerError::AuthUserNotAuthenticated => {
                (StatusCode::INTERNAL_SERVER_ERROR, "auth error")
            }
            ServerError::MiniJinja(_) => (StatusCode::INTERNAL_SERVER_ERROR, "minijinja error"),
            ServerError::MimeType => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "incompatible mimetype for request",
            ),
            ServerError::Multipart(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "multipart request error")
            }
            ServerError::FSList(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "filesystem listing error",
            ),
            ServerError::Subobject(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "subobject access error")
            }
            ServerError::StateRequired => (StatusCode::BAD_REQUEST, "need a state to make a clone"),
            ServerError::V86Clone(_) => (StatusCode::INTERNAL_SERVER_ERROR, "v86 clone failed"),
            ServerError::Unreachable => (StatusCode::INTERNAL_SERVER_ERROR, "uh oh error"),
            ServerError::AuthSession(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "auth session error")
            }
            ServerError::AuthBackend(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "auth backend error")
            }
            ServerError::AuthBackendSystem(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "auth backend internal system error",
            ),
            ServerError::SearchIndex(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "search index error")
            }
            ServerError::Search(_) => (StatusCode::INTERNAL_SERVER_ERROR, "search error"),
            ServerError::Insert(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "database or index error")
            }
        };

        let base_url = crate::server::BASE_URL.get();
        let body = Html(
            error_template
                .render(context!(base_url => base_url, status_code => status.clone().to_string(), message => message))
                .unwrap(),
        );

        (
            status,
            (
                [
                    ("Access-Control-Allow-Origin", "*"),
                    ("Cross-Origin-Opener-Policy", "same-origin"),
                    ("Cross-Origin-Resource-Policy", "same-origin"),
                    ("Cross-Origin-Embedder-Policy", "require-corp"),
                ],
                body,
            ),
        )
            .into_response()
    }
}

#[derive(thiserror::Error)]
pub enum AuthError {
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("creator insert error")]
    Insert(#[from] gisst::error::Insert),
    #[error("csrf error")]
    CsrfMissing,
    #[error("user not allowed to access resource")]
    UserNotPermitted,
    #[error("token request error")]
    TokenRequest(
        #[from]
        oauth2::RequestTokenError<
            oauth2::HttpClientError<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ),
    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("missing user auth profile information: {}", .field)]
    MissingProfileInfo { field: String },
    #[error("error with record SQL manipulation")]
    RecordManipulation(#[from] gisst::error::RecordSQL),
}
impl Debug for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self}")?;
        if let Some(source) = self.source() {
            writeln!(f, "Caused by:\n\t{source}")?;
        }
        Ok(())
    }
}
