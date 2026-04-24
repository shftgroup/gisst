use crate::{auth::AuthBackend, error::ServerError, server::ServerState};
use axum::{Extension, Router, extract::Json, routing::post};
use axum_login::login_required;
use gisst::model_enums::Framework;
use gisst::models::Environment;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/create", post(create_or_derive_environment))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
}
#[allow(clippy::struct_field_names)]
#[derive(Debug, serde::Deserialize)]
struct CreateEnvironment {
    #[serde(deserialize_with = "crate::utils::uuid_or_empty_string")]
    environment_id: Option<Uuid>,
    environment_name: String,
    environment_framework: Framework,
    environment_platform: String,
    environment_core_name: String,
    environment_core_version: String,
    #[serde(deserialize_with = "crate::utils::uuid_or_empty_string")]
    environment_derived_from: Option<Uuid>,
    environment_config: Option<sqlx::types::JsonValue>,
}
#[tracing::instrument(skip(app_state, auth), fields(userid))]
async fn create_or_derive_environment(
    app_state: Extension<ServerState>,
    auth: axum_login::AuthSession<crate::auth::AuthBackend>,
    Json(env): Json<CreateEnvironment>,
) -> Result<Json<Environment>, ServerError> {
    tracing::Span::current().record(
        "userid",
        auth.user.as_ref().map(|u| u.creator_id.to_string()),
    );
    let creator_id = auth
        .user
        .ok_or(ServerError::AuthUserNotAuthenticated)?
        .creator_id;

    let mut conn = app_state.pool.acquire().await?;

    tracing::info!("Inserting env {env:?}");
    if let Some(id) = env.environment_id
        && let Some(existing) = Environment::get_by_id(&mut conn, id).await?
    {
        // if this env differs from existing, derive a new env
        if env.environment_name == existing.environment_name
            && env.environment_framework == existing.environment_framework
            && env.environment_platform == existing.environment_platform
            && env.environment_core_name == existing.environment_core_name
            && env.environment_core_version == existing.environment_core_version
            && env.environment_config == existing.environment_config
        {
            Ok(Json(existing))
        } else {
            Ok(Json(
                Environment::insert(
                    &mut conn,
                    Environment {
                        environment_id: Uuid::new_v4(),
                        environment_name: env.environment_name,
                        environment_framework: env.environment_framework,
                        environment_platform: env.environment_platform,
                        environment_core_name: env.environment_core_name,
                        environment_core_version: env.environment_core_version,
                        environment_derived_from: Some(existing.environment_id),
                        environment_config: env.environment_config,
                        creator_id: Some(creator_id),
                        created_on: chrono::Utc::now(),
                    },
                )
                .await?,
            ))
        }
    } else {
        // insert a new env
        Ok(Json(
            Environment::insert(
                &mut conn,
                Environment {
                    environment_id: Uuid::new_v4(),
                    environment_name: env.environment_name,
                    environment_framework: env.environment_framework,
                    environment_platform: env.environment_platform,
                    environment_core_name: env.environment_core_name,
                    environment_core_version: env.environment_core_version,
                    environment_derived_from: env.environment_derived_from,
                    environment_config: env.environment_config,
                    creator_id: Some(creator_id),
                    created_on: chrono::Utc::now(),
                },
            )
            .await?,
        ))
    }
}
