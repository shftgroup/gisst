use super::LoggedInUserInfo;
use crate::{
    error::ServerError,
    server::{BASE_URL, ServerState},
    utils::parse_header,
};

use gisst::error::Table;
use gisst::models::{Environment, Instance, ObjectLink, ReplayLink, SaveLink, StateLink, Work};

use axum::{
    Extension,
    extract::{OriginalUri, Path},
    http::HeaderMap,
    response::{Html, IntoResponse},
};
use axum_extra::extract::Query;
use chrono::{DateTime, Local};
use minijinja::context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Debug)]
struct PlayerTemplateInfo {
    gisst_root: String,
    instance: Instance,
    work: Work,
    environment: Environment,
    user: LoggedInUserInfo,
    saves: Vec<SaveLink>,
    start: PlayerStartTemplateInfo,
    manifest: Vec<ObjectLink>,
    boot_into_record: bool,
}

#[derive(Deserialize, Serialize, Debug)]
struct EmbedDataInfo {
    gisst_root: String,
    instance: Instance,
    work: Work,
    environment: Environment,
    saves: Vec<SaveLink>,
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

#[derive(Deserialize, Debug, Default)]
pub struct PlayerParams {
    state: Option<Uuid>,
    replay: Option<Uuid>,
    #[serde(default)]
    save: Vec<Uuid>,
    boot_into_record: Option<bool>,
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip(app_state))]
pub async fn get_data(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    OriginalUri(uri): OriginalUri,
    Query(params): Query<PlayerParams>,
) -> Result<axum::response::Response, ServerError> {
    let mut conn = app_state.pool.acquire().await?;
    let instance = Instance::get_by_id(&mut conn, id)
        .await?
        .ok_or(ServerError::RecordMissing {
            table: Table::Instance,
            uuid: id,
        })?;
    let environment = Environment::get_by_id(&mut conn, instance.environment_id)
        .await?
        .ok_or(ServerError::RecordMissing {
            table: Table::Environment,
            uuid: instance.environment_id,
        })?;
    let work =
        Work::get_by_id(&mut conn, instance.work_id)
            .await?
            .ok_or(ServerError::RecordMissing {
                table: Table::Work,
                uuid: instance.work_id,
            })?;
    let start = match (params.state, params.replay) {
        (Some(id), None) => {
            PlayerStartTemplateInfo::State(StateLink::get_by_id(&mut conn, id).await?.ok_or(
                ServerError::RecordLinking {
                    table: Table::State,
                    uuid: id,
                },
            )?)
        }
        (None, Some(id)) => {
            PlayerStartTemplateInfo::Replay(ReplayLink::get_by_id(&mut conn, id).await?.ok_or(
                ServerError::RecordLinking {
                    table: Table::Replay,
                    uuid: id,
                },
            )?)
        }
        (None, None) => PlayerStartTemplateInfo::Cold,
        (_, _) => return Err(ServerError::Unreachable),
    };
    let saves: Vec<SaveLink> = SaveLink::get_by_ids(&mut conn, &params.save).await?;
    let manifest = ObjectLink::get_all_for_instance_id(&mut conn, instance.instance_id).await?;

    // This unwrap is safe since BASE_URL is initialized at launch
    let url_string = BASE_URL.get().unwrap();

    let url_parts: Vec<&str> = url_string.split("//").collect();
    let current_date: DateTime<Local> = Local::now();

    let citation_website_title: String = match &start {
        PlayerStartTemplateInfo::State(s) => {
            format!("GISST Citation of {} in {}", s.state_name, work.work_name)
        }
        PlayerStartTemplateInfo::Replay(r) => {
            format!("GISST Citation of {} in {}", r.replay_name, work.work_name)
        }
        PlayerStartTemplateInfo::Cold => String::new(),
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
        gisst_root: BASE_URL.get().unwrap().clone(),
        environment,
        instance,
        work,
        saves,
        start,
        manifest,
        host_url: url_parts[1].to_string(),
        host_protocol: url_parts[0].to_string(),
        citation_data: Some(citation_data),
    };

    let accept: Option<String> = parse_header(&headers, "Accept");
    Ok((if accept.is_none()
        || accept.as_ref().is_some_and(|hv| hv.contains("text/html"))
        || accept.as_ref().is_some_and(|hv| hv.contains("*/*"))
    {
        let citation_page = app_state
            .templates
            .get_template("single_citation_page.html")?;
        (
            [
                ("Access-Control-Allow-Origin", "*"),
                ("Cross-Origin-Opener-Policy", "same-origin"),
                ("Cross-Origin-Resource-Policy", "same-origin"),
                ("Cross-Origin-Embedder-Policy", "require-corp"),
            ],
            Html(citation_page.render(context! {
                base_url => BASE_URL.get(),
                embed_data => embed_data,
            })?),
        )
            .into_response()
    } else if accept
        .as_ref()
        .is_some_and(|hv| hv.contains("application/json"))
    {
        (
            [
                ("Access-Control-Allow-Origin", "*"),
                ("Cross-Origin-Opener-Policy", "cross-origin"),
                ("Cross-Origin-Resource-Policy", "cross-origin"),
                ("Cross-Origin-Embedder-Policy", "require-corp"),
            ],
            axum::Json(embed_data),
        )
            .into_response()
    } else {
        tracing::error!("unrecognized accept header {accept:?}");
        Err(ServerError::MimeType)?
    })
    .into_response())
}

#[tracing::instrument(skip(app_state, auth), fields(userid))]
pub async fn get_player(
    app_state: Extension<ServerState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(params): Query<PlayerParams>,
    auth: axum_login::AuthSession<crate::auth::AuthBackend>,
) -> Result<axum::response::Response, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let mut conn = app_state.pool.acquire().await?;
    let instance = Instance::get_by_id(&mut conn, id)
        .await?
        .ok_or(ServerError::RecordMissing {
            table: Table::Instance,
            uuid: id,
        })?;
    let environment = Environment::get_by_id(&mut conn, instance.environment_id)
        .await?
        .ok_or(ServerError::RecordMissing {
            table: Table::Environment,
            uuid: instance.environment_id,
        })?;
    let work =
        Work::get_by_id(&mut conn, instance.work_id)
            .await?
            .ok_or(ServerError::RecordMissing {
                table: Table::Work,
                uuid: instance.work_id,
            })?;
    let user = LoggedInUserInfo::generate_from_user(&auth.user.unwrap());
    let start = match (params.state, params.replay) {
        (Some(id), None) => {
            PlayerStartTemplateInfo::State(StateLink::get_by_id(&mut conn, id).await?.ok_or(
                ServerError::RecordLinking {
                    table: Table::State,
                    uuid: id,
                },
            )?)
        }
        (None, Some(id)) => {
            PlayerStartTemplateInfo::Replay(ReplayLink::get_by_id(&mut conn, id).await?.ok_or(
                ServerError::RecordLinking {
                    table: Table::Replay,
                    uuid: id,
                },
            )?)
        }
        (None, None) => PlayerStartTemplateInfo::Cold,
        (_, _) => return Err(ServerError::Unreachable),
    };
    let saves: Vec<SaveLink> = SaveLink::get_by_ids(&mut conn, &params.save).await?;
    let manifest = ObjectLink::get_all_for_instance_id(&mut conn, instance.instance_id).await?;
    tracing::debug!("manifest: {manifest:?}");
    Ok((
        [
            ("Access-Control-Allow-Origin", "*"),
            ("Cross-Origin-Opener-Policy", "same-origin"),
            ("Cross-Origin-Resource-Policy", "same-origin"),
            ("Cross-Origin-Embedder-Policy", "require-corp"),
        ],
        Html(
            app_state
                .templates
                .get_template("player.html")?
                .render(context!(
                    base_url => BASE_URL.get(),
                    player_params => PlayerTemplateInfo {
                        gisst_root: BASE_URL.get().unwrap().clone(),
                        environment,
                        instance,
                        work,
                        saves,
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
