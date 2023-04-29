use serde::{Serialize, Deserialize};
use serde::__private::de::TagOrContentField::Content;
use sqlx::{FromRow, PgConnection};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentItem{
    content_id: i32,
    content_uuid: Uuid,
    content_title: String,
    content_version: Option<String>,
    content_path: String,
    content_filename: String,
    platform_id: i32,
    content_parent_id: Option<i32>,
    created_on: time::PrimitiveDateTime,
}

impl ContentItem {

    pub fn empty() -> Self {
        Self {
            content_id: -1,
            content_uuid: Uuid::parse_str("0").unwrap(),
            content_title: String::from(""),
            content_version: None,
            content_path: String::from(""),
            content_filename: String::from(""),
            platform_id: -1,
            content_parent_id: None,
            created_on: time::PrimitiveDateTime()
        }
    }

    pub async fn get_by_id(
        conn: &mut PgConnection,
        id: i32
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT content_id, content_uuid, content_title, content_version, content_path, content_filename, platform_id, content_parent_id, created_on WHERE content_id = $1",
            id
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn get_by_uuid(
        conn: &mut PgConnection,
        uuid: Uuid
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT content_id, content_uuid, content_title, content_version, content_path, content_filename, platform_id, content_parent_id, created_on WHERE content_uuid = $1",
            uuid
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn get_all_by_platform_id(
        conn: &mut PgConnection,
        id: i32
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT content_id, content_uuid, content_title, content_version, content_path, content_filename, platform_id, content_parent_id, created_on WHERE platform_id = $1",
            id
        )
            .fetch_all(conn)
            .await
    }

    pub async fn insert(conn: &mut PgConnection, content_item: ContentItem) -> Result<Self, NewContentItemError> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO content (
            content_uuid,
            content_title, content_version,
            content_path, content_filename,
            platform_id, content_parent_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id"#,
            content_item.content_uuid,
            content_item.content_version,
            content_item.content_path,
            content_item.content_filename,
            content_item.platform_id,
            content_item.content_parent_id
        )
            .fetch_one(conn)
            .await
            .map_err(|_| NewContentItemError::DatabaseError)
    }


}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum NewContentItemError {
    #[error("could not insert into database")]
    DatabaseError,
}

#[derive(Debug, Serialize, Deserialize)]
struct Core{
    core_id: i32,
    core_name: String,
    core_version: String,
    core_manifest: serde_json::Value,
}

impl Core {

    pub async fn get_by_id(
        conn: &mut PgConnection,
        id: i32
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT core_id, core_name, core_version, core_manifest WHERE core_id = $1",
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
            "SELECT core_id, core_name, core_version, core_manifest WHERE core_name = $1",
            name
        )
            .fetch_optional(conn)
            .await
    }


}

#[derive(Debug, Serialize)]
struct Save{
    id: i32,
    uuid: uuid::Uuid,
    short_description: String,
    description: String,
    filename: String,
    path: std::path::Path,
}

#[derive(Debug, Serialize)]
struct Replay{
    id: i32,
    uuid: uuid::Uuid,
    content_id: i32,
    save_id: i32,
    filename: String,
    path: std::path::Path,
}

#[derive(Debug, Serialize)]
struct Platform{
    platform_id: i32,
    core_id: i32,
    platform_framework: String,
}

#[derive(Debug, Serialize)]
struct State{
    id: i32,
    uuid: uuid::Uuid,
    screenshot: Vec<u8>,
    replay_id: i32,
    content_id: i32,
    replay_time_index: u16,
    is_checkpoint: bool,
    path: String,
    filename: String,
    name: String,
    description: String,
}
