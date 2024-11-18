use crate::error::GISSTError;
use gisst::models::{Environment, Instance, ObjectLink, ReplayLink, Save, StateLink, Work};
use std::collections::HashMap;
use tower::ServiceBuilder;

use crate::{
    auth, db,
    routes::{
        creator_router, instance_router, object_router, replay_router, save_router, state_router,
        work_router,
    },
    serverconfig::ServerConfig,
    tus::{tus_creation, tus_head, tus_patch},
};
use anyhow::Result;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, Path, Query},
    http::HeaderMap,
    response::{Html, IntoResponse},
    routing::method_routing::{get, patch, post},
    Extension, Router,
};

use axum_login::{AuthLayer, RequireAuthorizationLayer};

use crate::auth::{AuthContext, User};
use crate::routes::screenshot_router;
use crate::utils::parse_header;
use axum::extract::OriginalUri;
use axum_login::axum_sessions::async_session::MemoryStore;
use axum_login::axum_sessions::{SameSite, SessionLayer};
use chrono::{DateTime, Local};
use gisst::error::ErrorTable;
use gisst::storage::{PendingUpload, StorageHandler};
use minijinja::context;
use oauth2::basic::BasicClient;
use rand::Rng;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::debug;
use uuid::Uuid;

#[derive(Clone)]
pub struct ServerState {
    pub pool: PgPool,
    pub root_storage_path: String,
    pub temp_storage_path: String,
    pub folder_depth: u8,
    pub default_chunk_size: usize,
    pub base_url: String,
    pub pending_uploads: Arc<RwLock<HashMap<Uuid, PendingUpload>>>,
    pub templates: minijinja::Environment<'static>,
    pub oauth_client: BasicClient,
    pub user_whitelist: Vec<String>,
}

pub async fn launch(config: &ServerConfig) -> Result<()> {
    use crate::selective_serve_dir;
    StorageHandler::init_storage(
        &config.storage.root_folder_path,
        &config.storage.temp_folder_path,
    )?;

    let mut template_environment = minijinja::Environment::new();
    template_environment.set_loader(minijinja::path_loader("gisst-server/src/templates"));

    debug!("Configured URL is {}", config.http.base_url.clone());

    let mut user_whitelist_sorted: Vec<String> = config.auth.user_whitelist.to_vec();

    user_whitelist_sorted.sort();

    let app_state = ServerState {
        pool: db::new_pool(config).await?,
        root_storage_path: config.storage.root_folder_path.clone(),
        temp_storage_path: config.storage.temp_folder_path.clone(),
        folder_depth: config.storage.folder_depth,
        default_chunk_size: config.storage.chunk_size,
        base_url: config.http.base_url.clone(),
        pending_uploads: Default::default(),
        templates: template_environment,
        user_whitelist: user_whitelist_sorted,
        oauth_client: auth::build_oauth_client(
            &config.http.base_url,
            config.auth.google_client_id.expose_secret(),
            config.auth.google_client_secret.expose_secret(),
        ),
    };

    let secret = rand::thread_rng().gen::<[u8; 64]>();
    let user_pool = PgPoolOptions::new()
        .connect(config.database.database_url.expose_secret())
        .await
        .unwrap();

    let user_store = auth::PostgresStore::new(user_pool.clone());
    let auth_layer: AuthLayer<auth::PostgresStore, i32, auth::User, auth::Role> =
        AuthLayer::new(user_store, &secret);
    let session_store = MemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret)
        .with_secure(false)
        .with_same_site_policy(SameSite::Lax);

    let builder = ServiceBuilder::new()
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::AllowOrigin::any())
                .allow_headers(tower_http::cors::AllowHeaders::any())
                .expose_headers([
                    axum::http::header::ACCEPT_RANGES,
                    axum::http::header::CONTENT_LENGTH,
                    axum::http::header::RANGE,
                    axum::http::header::CONTENT_RANGE,
                ])
                .allow_methods([axum::http::Method::GET]),
        )
        .layer(HandleErrorLayer::new(handle_error))
        // This map_err is needed to get the types to work out after handleerror and before servedir.
        .map_err(|e| panic!("{:?}", e));

    let app = Router::new()
        .route("/play/:instance_id", get(get_player))
        .route("/resources/:id", patch(tus_patch).head(tus_head))
        .route("/resources", post(tus_creation))
        .nest("/objects", object_router())
        .route_layer(RequireAuthorizationLayer::<i32, auth::User, auth::Role>::login_or_redirect(Arc::new("/login".into()), None))
        .route("/data/:instance_id", get(get_data))
        .route("/login", get(auth::login_handler))
        .route("/auth/google/callback", get(auth::oauth_callback_handler))
        .route("/logout", get(auth::logout_handler))
        //.route("/debug/tus_test", get(get_upload_form))
        .nest("/creators", creator_router())
        .nest("/instances", instance_router())
        .nest("/replays", replay_router())
        .nest("/saves", save_router())
        .nest("/screenshots", screenshot_router())
        .nest("/states", state_router())
        .nest("/works", work_router())
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
        .layer(DefaultBodyLimit::max(33554432))
        .layer(auth_layer)
        .layer(TraceLayer::new_for_http())
        .layer(session_layer);

    let addr = SocketAddr::new(
        IpAddr::V4(config.http.listen_address),
        config.http.listen_port,
    );

    if config.http.dev_ssl {
        // use axum_server::tls_rustls::RustlsConfig;
        // let tlsconfig = RustlsConfig::from_pem_file(
        //     config
        //         .http
        //         .dev_cert
        //         .expect("In dev SSL mode, must supply cert in config"),
        //     config
        //         .http
        //         .dev_key
        //         .expect("In dev SSL mode, must supply key in config"),
        // )
        // .await
        // .unwrap();

        // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        // println!("listening on {}", addr);
        // axum::Server::bind_rustls(addr, tlsconfig)
        //     .serve(app.into_make_service())
        //     .await
        //     .unwrap();
    } else {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .expect("could not launch GISST HTTP server on port 3000");
    }

    Ok(())
}
async fn handle_error(error: axum::BoxError) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", error),
    )
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoggedInUserInfo {
    email: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    username: Option<String>,
    creator_id: Uuid,
}

impl LoggedInUserInfo {
    pub fn generate_from_user(user: &User) -> LoggedInUserInfo {
        LoggedInUserInfo {
            email: user.email.clone(),
            name: user.name.clone(),
            given_name: user.given_name.clone(),
            family_name: user.family_name.clone(),
            username: user.preferred_username.clone(),
            creator_id: user.creator_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct PlayerTemplateInfo {
    instance: Instance,
    work: Work,
    environment: Environment,
    user: LoggedInUserInfo,
    save: Option<Save>,
    start: PlayerStartTemplateInfo,
    manifest: Vec<ObjectLink>,
    boot_into_record: bool,
}

#[derive(Deserialize, Serialize, Debug)]
struct EmbedDataInfo {
    instance: Instance,
    work: Work,
    environment: Environment,
    save: Option<Save>,
    start: PlayerStartTemplateInfo,
    manifest: Vec<ObjectLink>,
    host_url: String,
    host_protocol: String,
    citation_data: Option<CitationDataInfo>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CitationDataInfo {
    website_title: String,
    url: String,
    gs_page_view_date: String,
    mla_page_view_date: String,
    bibtex_page_view_date: String,
    site_published_year: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
enum PlayerStartTemplateInfo {
    Cold,
    State(StateLink),
    Replay(ReplayLink),
}

#[derive(Deserialize)]
struct PlayerParams {
    state: Option<Uuid>,
    replay: Option<Uuid>,
    boot_into_record: Option<bool>,
}

async fn get_about(
    app_state: Extension<ServerState>,
) -> Result<axum::response::Response, GISSTError> {
    Ok(Html(
        app_state
            .templates
            .get_template("about.html")?
            .render(context!())?,
    )
    .into_response())
}
async fn get_homepage(
    app_state: Extension<ServerState>,
) -> Result<axum::response::Response, GISSTError> {
    Ok(Html(
        app_state
            .templates
            .get_template("index.html")?
            .render(context!())?,
    )
    .into_response())
}

async fn get_data(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    OriginalUri(uri): OriginalUri,
    Query(params): Query<PlayerParams>,
) -> Result<axum::response::Response, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let instance =
        Instance::get_by_id(&mut conn, id)
            .await?
            .ok_or(GISSTError::RecordMissingError {
                table: ErrorTable::Instance,
                uuid: id,
            })?;
    let environment = Environment::get_by_id(&mut conn, instance.environment_id)
        .await?
        .ok_or(GISSTError::RecordMissingError {
            table: ErrorTable::Environment,
            uuid: instance.environment_id,
        })?;
    let work = Work::get_by_id(&mut conn, instance.work_id).await?.ok_or(
        GISSTError::RecordMissingError {
            table: ErrorTable::Work,
            uuid: instance.work_id,
        },
    )?;
    let start = match dbg!((params.state, params.replay)) {
        (Some(id), None) => {
            PlayerStartTemplateInfo::State(StateLink::get_by_id(&mut conn, id).await?.ok_or(
                GISSTError::RecordLinkingError {
                    table: ErrorTable::State,
                    uuid: id,
                },
            )?)
        }
        (None, Some(id)) => {
            PlayerStartTemplateInfo::Replay(ReplayLink::get_by_id(&mut conn, id).await?.ok_or(
                GISSTError::RecordLinkingError {
                    table: ErrorTable::Replay,
                    uuid: id,
                },
            )?)
        }
        (None, None) => PlayerStartTemplateInfo::Cold,
        (_, _) => return Err(GISSTError::Unreachable),
    };
    let manifest = ObjectLink::get_all_for_instance_id(&mut conn, instance.instance_id).await?;
    debug!("{manifest:?}");

    let url_string = app_state.base_url.clone();

    let url_parts: Vec<&str> = url_string.split("//").collect();
    let current_date: DateTime<Local> = Local::now();

    let citation_website_title: String = match &start {
        PlayerStartTemplateInfo::State(s) => {
            format!("GISST Citation of {} in {}", s.state_name, work.work_name)
        }
        PlayerStartTemplateInfo::Replay(r) => {
            format!("GISST Citation of {} in {}", r.replay_name, work.work_name)
        }
        PlayerStartTemplateInfo::Cold => "".to_string(),
    };

    let citation_data: CitationDataInfo = CitationDataInfo {
        website_title: citation_website_title,
        url: uri.to_string(),
        gs_page_view_date: current_date.format("%Y, %B %d").to_string(),
        mla_page_view_date: current_date.format("%d %b. %Y").to_string(),
        bibtex_page_view_date: current_date.format("%Y-%m-%d").to_string(),
        site_published_year: current_date.format("%Y").to_string(),
    };

    let embed_data = EmbedDataInfo {
        environment,
        instance,
        work,
        save: None,
        start,
        manifest,
        host_url: url_parts[1].to_string(),
        host_protocol: url_parts[0].to_string(),
        citation_data: Some(citation_data),
    };

    let accept: Option<String> = parse_header(&headers, "Accept");
    Ok(
        (if accept.is_none() || accept.as_ref().is_some_and(|hv| hv.contains("text/html")) {
            let citation_page = app_state
                .templates
                .get_template("single_citation_page.html")?;
            Html(citation_page.render(context! {
                embed_data => embed_data,
            })?)
            .into_response()
        } else if accept
            .as_ref()
            .is_some_and(|hv| hv.contains("application/json"))
        {
            (
                [("Access-Control-Allow-Origin", "*")],
                axum::Json(embed_data),
            )
                .into_response()
        } else {
            Err(GISSTError::MimeTypeError)?
        })
        .into_response(),
    )
}

async fn get_player(
    app_state: Extension<ServerState>,
    _headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(params): Query<PlayerParams>,
    auth: AuthContext,
) -> Result<axum::response::Response, GISSTError> {
    let mut conn = app_state.pool.acquire().await?;
    let instance =
        Instance::get_by_id(&mut conn, id)
            .await?
            .ok_or(GISSTError::RecordMissingError {
                table: ErrorTable::Instance,
                uuid: id,
            })?;
    let environment = Environment::get_by_id(&mut conn, instance.environment_id)
        .await?
        .ok_or(GISSTError::RecordMissingError {
            table: ErrorTable::Environment,
            uuid: instance.environment_id,
        })?;
    let work = Work::get_by_id(&mut conn, instance.work_id).await?.ok_or(
        GISSTError::RecordMissingError {
            table: ErrorTable::Work,
            uuid: instance.work_id,
        },
    )?;
    let user = LoggedInUserInfo::generate_from_user(&auth.current_user.unwrap());
    let start = match dbg!((params.state, params.replay)) {
        (Some(id), None) => {
            PlayerStartTemplateInfo::State(StateLink::get_by_id(&mut conn, id).await?.ok_or(
                GISSTError::RecordLinkingError {
                    table: ErrorTable::State,
                    uuid: id,
                },
            )?)
        }
        (None, Some(id)) => {
            PlayerStartTemplateInfo::Replay(ReplayLink::get_by_id(&mut conn, id).await?.ok_or(
                GISSTError::RecordLinkingError {
                    table: ErrorTable::Replay,
                    uuid: id,
                },
            )?)
        }
        (None, None) => PlayerStartTemplateInfo::Cold,
        (_, _) => return Err(GISSTError::Unreachable),
    };
    let manifest =
        dbg!(ObjectLink::get_all_for_instance_id(&mut conn, instance.instance_id).await?);
    Ok((
        [("Access-Control-Allow-Origin", "*")],
        Html(
            app_state
                .templates
                .get_template("player.html")?
                .render(context!(
                    player_params => PlayerTemplateInfo {
                        environment,
                        instance,
                        work,
                        save: None,
                        user,
                        start,
                        manifest,
                        boot_into_record: params.boot_into_record.unwrap_or_default(),
                    }
                ))?,
        ),
    )
        .into_response())
}
