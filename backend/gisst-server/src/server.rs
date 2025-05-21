use crate::error::ServerError;
use tower::ServiceBuilder;

use crate::{
    auth::{self, AuthBackend},
    db,
    routes::{
        creator_router, instance_router, object_router, players, replay_router, save_router,
        screenshot_router, state_router, work_router,
    },
    serverconfig::ServerConfig,
    tus,
};
use anyhow::Result;
use axum::{
    Extension, Router,
    error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    response::{Html, IntoResponse},
    routing::method_routing::{get, patch, post},
};

use axum_login::tower_sessions::{MemoryStore, SessionManagerLayer, cookie::SameSite};
use gisst::storage::{PendingUpload, StorageHandler};
use minijinja::context;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

pub static BASE_URL: std::sync::OnceLock<String> = std::sync::OnceLock::new();

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct ServerState {
    pub pool: PgPool,
    pub root_storage_path: String,
    pub temp_storage_path: String,
    pub folder_depth: u8,
    pub default_chunk_size: usize,
    pub pending_uploads: Arc<RwLock<HashMap<Uuid, PendingUpload>>>,
    pub templates: minijinja::Environment<'static>,
    pub indexer: gisst::search::MeiliIndexer,
    pub search: gisst::search::MeiliSearch,
}
impl ServerState {
    async fn with_config(config: &ServerConfig) -> Result<Self, ServerError> {
        assert!(
            !config.http.base_url.ends_with('/'),
            "base_url must not end with slash"
        );
        let mut user_whitelist_sorted: Vec<String> = config.auth.user_whitelist.clone();
        user_whitelist_sorted.sort();
        let mut template_environment = minijinja::Environment::new();
        template_environment.set_loader(minijinja::path_loader("gisst-server/src/templates"));
        let indexer = gisst::search::MeiliIndexer::new(
            &config.search.meili_url,
            &config.search.meili_api_key,
        )?;
        let search = gisst::search::MeiliSearch::new(
            &config.search.meili_url,
            &config.search.meili_external_url,
            &config.search.meili_search_key,
        )?;
        Ok(Self {
            pool: db::new_pool(config).await?,
            root_storage_path: config.storage.root_folder_path.clone(),
            temp_storage_path: config.storage.temp_folder_path.clone(),
            folder_depth: config.storage.folder_depth,
            default_chunk_size: config.storage.chunk_size,
            pending_uploads: Arc::default(),
            templates: template_environment,
            indexer,
            search,
        })
    }
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(name = "launch")]
pub async fn launch(config: &ServerConfig) -> Result<()> {
    use crate::selective_serve_dir;

    let base_url = &config.http.base_url;
    // Unwrap here is fine since the server can't be launched more than once
    BASE_URL.set(base_url.clone()).unwrap();

    StorageHandler::init_storage(
        &config.storage.root_folder_path,
        &config.storage.temp_folder_path,
    )?;
    info!("Configured URL is {}", config.http.base_url.clone());

    let mut user_whitelist_sorted: Vec<String> = config.auth.user_whitelist.clone();
    user_whitelist_sorted.sort();

    let app_state = ServerState::with_config(config).await?;

    let user_pool = sqlx::postgres::PgPoolOptions::new()
        .connect(config.database.database_url.expose_secret())
        .await
        .unwrap();
    let metrics_pool = user_pool.clone();
    let user_store = auth::AuthBackend::new(
        user_pool,
        auth::build_oauth_client(
            &config.http.base_url,
            config.auth.google_client_id.expose_secret(),
            config.auth.google_client_secret.expose_secret(),
        ),
        user_whitelist_sorted,
        gisst::search::MeiliIndexer::new(&config.search.meili_url, &config.search.meili_api_key)?,
    );
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_same_site(SameSite::Lax)
        .with_name("gisst.sid");
    let auth_layer = axum_login::AuthManagerLayerBuilder::new(user_store, session_layer).build();

    gisst::metrics::start_reporting(metrics_pool).await;

    let builder = ServiceBuilder::new()
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::AllowOrigin::any())
                .allow_headers(tower_http::cors::AllowHeaders::any())
                .expose_headers([
                    axum::http::header::ACCEPT_RANGES,
                    axum::http::header::CONTENT_LENGTH,
                    axum::http::header::CONTENT_ENCODING,
                    axum::http::header::RANGE,
                    axum::http::header::CONTENT_RANGE,
                    "Content-Security-Policy".parse().unwrap(),
                    "Cross-Origin-Opener-Policy".parse().unwrap(),
                    "Cross-Origin-Resource-Policy".parse().unwrap(),
                    "Cross-Origin-Embedder-Policy".parse().unwrap(),
                ])
                .allow_methods([axum::http::Method::GET, axum::http::Method::HEAD]),
        )
        .layer(axum::middleware::map_response(set_embeddable_headers))
        .layer(HandleErrorLayer::new(handle_error))
        // This map_err is needed to get the types to work out after handleerror and before servedir.
        .map_err(|e| unreachable!("somehow a handled error wasn't actually handled {e:?}"));
    let app = Router::new()
        .route("/play/{instance_id}", get(players::get_player))
        .route("/resources/{id}", patch(tus::patch).head(tus::head))
        .route("/resources", post(tus::creation))
        .nest("/objects", object_router())
        .route("/logout", get(auth::logout_handler))
        .nest("/creators", creator_router())
        .nest("/instances", instance_router())
        .nest("/replays", replay_router())
        .nest("/saves", save_router())
        .nest("/screenshots", screenshot_router())
        .nest("/states", state_router())
        .nest("/works", work_router())
        .route_layer(
            // This is ugly, but it achieves the goal; the unwrap is fine
            // because BASE_URL was initialized earlier in this function.
            axum_login::login_required!(AuthBackend, login_url=&{format!("{}/login", BASE_URL.get().unwrap())})
        )
        .route("/data/{instance_id}", get(players::get_data))
        .route("/login", get(auth::login_handler))
        .route("/auth/google/callback", get(auth::oauth_callback_handler))
        .nest_service(
            "/ui",
            builder.clone()
                .service(selective_serve_dir::SelectiveServeDir::new("ui-dist")),
        )
        .nest_service(
            "/storage",
            builder.clone()
                /* if the x-accept-encoding header is present, dispatch to the custom servedir that does not serve precompressed stuff */
                .service(selective_serve_dir::SelectiveServeDir::new("storage")),
        )
        .nest_service(
            "/assets",
            builder.clone()
                .service(selective_serve_dir::SelectiveServeDir::new("web-dist/assets")),
        )
        .nest_service(
            "/cores",
            builder.clone()
                .service(selective_serve_dir::SelectiveServeDir::new("web-dist/cores")),
        )
        .nest_service(
            "/media",
            builder.clone()
                .service(selective_serve_dir::SelectiveServeDir::new("web-dist/media")),
        )
        .nest_service(
            "/ra",
            builder.clone().service(selective_serve_dir::SelectiveServeDir::new("web-dist/ra")),
        )
        .nest_service(
            "/v86",
            builder.clone()
                .service(selective_serve_dir::SelectiveServeDir::new("web-dist/v86")),
        )
        .nest_service(
            "/embed",
            builder.clone()
                .service(selective_serve_dir::SelectiveServeDir::new("embed-dist")),
        )
        .route("/", get(get_homepage))
        .route("/about", get(get_about))
        .layer(Extension(app_state))
        .layer(DefaultBodyLimit::max(33_554_432))
        .layer(TraceLayer::new_for_http()
               .make_span_with(tower_http::trace::DefaultMakeSpan::new()
                               .include_headers(config.env.trace_include_headers)))
        .layer(auth_layer);

    let addr = SocketAddr::new(
        IpAddr::V4(config.http.listen_address),
        config.http.listen_port,
    );

    if config.http.dev_ssl {
        use axum_server::tls_rustls::RustlsConfig;
        let tlsconfig = RustlsConfig::from_pem_file(
            config
                .http
                .dev_cert
                .as_ref()
                .expect("In dev SSL mode, must supply cert in config"),
            config
                .http
                .dev_key
                .as_ref()
                .expect("In dev SSL mode, must supply key in config"),
        )
        .await
        .unwrap();

        axum_server::bind_rustls(addr, tlsconfig)
            .serve(app.into_make_service())
            .await
            .expect("could not launch GISST HTTP server on port 3000");
    } else {
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .expect("could not launch GISST HTTP server on port 3000");
    }
    Ok(())
}

async fn handle_error(error: axum::BoxError) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {error}"),
    )
}

async fn get_about(
    app_state: Extension<ServerState>,
) -> Result<axum::response::Response, ServerError> {
    Ok(Html(
        app_state
            .templates
            .get_template("about.html")?
            .render(context!(base_url => BASE_URL.get()))?,
    )
    .into_response())
}

#[tracing::instrument(skip(app_state))]
async fn get_homepage(
    app_state: Extension<ServerState>,
) -> Result<axum::response::Response, ServerError> {
    info!("Generating homepage");
    Ok(Html(
        app_state
            .templates
            .get_template("index.html")?
            .render(context!(base_url => BASE_URL.get()))?,
    )
    .into_response())
}

async fn set_embeddable_headers<B>(
    mut response: axum::response::Response<B>,
) -> axum::response::Response<B> {
    let headers = response.headers_mut();
    let _ = headers.insert("Cross-Origin-Opener-Policy", "same-origin".parse().unwrap());
    let _ = headers.insert(
        "Cross-Origin-Resource-Policy",
        "cross-origin".parse().unwrap(),
    );
    let _ = headers.insert(
        "Cross-Origin-Embedder-Policy",
        "require-corp".parse().unwrap(),
    );
    let _ = headers.insert(
        "Content-Security-Policy",
        "script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval' blob: https://localhost:3000/; worker-src 'self' blob: https://localhost:3000/;"
            .parse()
            .unwrap(),
    );
    response
}
