use serde::{Serialize, Deserialize};
use sqlx::{
    PgConnection,
    FromRow,
};
use time::OffsetDateTime;

use uuid::{
    Uuid,
    uuid
};

// Model definitions that should match PSQL database schema

#[derive(Debug, Default, FromRow, Serialize, Deserialize)]
pub struct ContentItem{
    content_id: Uuid,
    content_title: Option<String>,
    content_version: Option<String>,
    content_path: Option<String>,
    content_filename: Option<String>,
    pub platform_id: Option<Uuid>,
    content_parent_id: Option<Uuid>,
    created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Core{
    core_id: Uuid,
    core_name: Option<String>,
    core_version: Option<String>,
    core_manifest: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Creator{
    creator_id: Uuid,
    creator_username: Option<String>,
    creator_full_name: Option<String>,
    created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image{
    image_id: Uuid,
    image_filename: Option<String>,
    image_parent_id: Option<Uuid>,
    image_path: Option<String>,
    created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Platform{
    platform_id: Uuid,
    pub core_id: Option<Uuid>,
    platform_framework: Option<String>,
    created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Save{
    save_id: Uuid,
    save_short_desc: Option<String>,
    save_description: Option<String>,
    save_filename: Option<String>,
    save_path: Option<String>,
    creator_id: Option<Uuid>,
    content_id: Option<Uuid>,
    core_id: Option<Uuid>,
    created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Replay{
    replay_id: Uuid,
    content_id: Option<Uuid>,
    save_id: Option<Uuid>,
    creator_id: Option<Uuid>,
    replay_forked_from: Option<Uuid>,
    replay_filename: Option<String>,
    replay_path: Option<String>,
    core_id: Option<Uuid>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State{
    state_id: Uuid,
    screenshot_id: Option<Uuid>,
    replay_id: Option<Uuid>,
    content_id: Option<Uuid>,
    creator_id: Option<Uuid>,
    state_replay_index: Option<i32>,
    is_checkpoint: Option<bool>,
    state_path: Option<String>,
    state_filename: Option<String>,
    state_name: Option<String>,
    state_description: Option<String>,
    core_id: Option<Uuid>,
    state_derived_from: Option<Uuid>,
}

fn default_uuid() -> Uuid {
    uuid!("00000000-0000-0000-0000-000000000000")
}

impl ContentItem {
    pub fn default() -> Self {
        Self { content_id: default_uuid(), ..Default::default()}
    }

    pub async fn get_by_id(
        conn: &mut PgConnection,
        id: Uuid
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT content_id,
            content_title, content_version,
            content_path, content_filename,
            platform_id, content_parent_id,
            created_on
            FROM content
            WHERE content_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn get_all_by_platform_id(
        conn: &mut PgConnection,
        id: Uuid
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT content_id,
            content_title, content_version,
            content_path, content_filename,
            platform_id, content_parent_id,
            created_on
            FROM content
            WHERE platform_id = $1"#,
            id
        )
            .fetch_all(conn)
            .await
    }

    pub async fn insert(conn: &mut PgConnection, content_item: ContentItem) -> Result<Self, NewRecordError> {
        // Note: the "!" following the AS statements after RETURNING are forcing not-null status on those fields
        // from: https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns
        sqlx::query_as!(ContentItem,
            r#"INSERT INTO content (
            content_title, content_version,
            content_path, content_filename,
            platform_id, content_parent_id, created_on)
            VALUES ($1, $2, $3, $4, $5, $6, current_timestamp)
            RETURNING
            content_id,
            content_title,
            content_version,
            content_path,
            content_filename,
            platform_id,
            content_parent_id, created_on
            "#,
            content_item.content_title,
            content_item.content_version,
            content_item.content_path,
            content_item.content_filename,
            content_item.platform_id,
            content_item.content_parent_id,
        )
            .fetch_one(conn)
            .await
            .map_err(|_| NewRecordError::ContentError)
    }


}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum NewRecordError {
    #[error("could not insert content record into database")]
    ContentError,
    #[error("could not insert core record into database")]
    CoreError,
    #[error("could not insert platform record into database")]
    PlatformError,
    #[error("could not insert replay record into database")]
    ReplayError,
    #[error("could not insert creator record into database")]
    CreatorError,
}


impl Core {
    pub fn default() -> Self {
        Self{core_id: default_uuid(), ..Default::default()}
    }

    pub async fn get_by_id(
        conn: &mut PgConnection,
        id: Uuid
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT core_id, core_name, core_version, core_manifest FROM core WHERE core_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn get_by_name(
        conn: &mut PgConnection,
        name: &str
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT core_id, core_name, core_version, core_manifest FROM core WHERE core_name = $1"#,
            name
        )
            .fetch_optional(conn)
            .await
    }
}

impl Save {
    pub fn default() -> Self {
        Self { save_id: default_uuid(), ..Default::default()}
    }

    pub async fn get_by_id(conn: &mut PgConnection, id:Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT
            save_id,
            save_short_desc,
            save_description,
            save_filename,
            save_path,
            creator_id,
            content_id,
            core_id,
            created_on
            FROM save WHERE save_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }
}

impl Platform {
    pub fn default() -> Self{
        Self { platform_id: default_uuid(), ..Default::default()}
    }
    pub async fn get_by_id(conn: &mut PgConnection, id:Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT platform_id, core_id, platform_framework, created_on
            FROM platform WHERE platform_id = $1
            "#,
            id
        )
            .fetch_optional(conn)
            .await
    }
}

impl Replay {
    pub fn default() -> Self{
        Self { replay_id: default_uuid(), ..Default::default()}
    }

    pub async fn get_by_id(conn: &mut PgConnection, id:Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT replay_id,
            content_id,
            save_id,
            creator_id,
            replay_forked_from,
            replay_filename,
            replay_path,
            core_id
            FROM replay WHERE replay_id = $1
            "#,
            id,
        )
            .fetch_optional(conn)
            .await
    }
}

impl State {
    pub fn default() -> Self{
        Self { state_id: default_uuid(), ..Default::default()}
    }


    pub async fn get_by_id(conn: &mut PgConnection, id:Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT state_id,
            screenshot_id,
            replay_id,
            content_id,
            creator_id,
            state_replay_index,
            is_checkpoint,
            state_path,
            state_filename,
            state_name,
            state_description,
            core_id,
            state_derived_from
            FROM state WHERE state_id = $1
            "#,
            id,
        )
            .fetch_optional(conn)
            .await
    }
}