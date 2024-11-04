use serde::{de, Deserialize, Deserializer, Serialize};

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use sqlx::postgres::{PgConnection, PgQueryResult};

use crate::model_enums::Framework;
use serde_with::{base64::Base64, serde_as};
use uuid::Uuid;

use crate::error::{ErrorAction, ErrorTable, RecordSQLError};

// empty_string_as_none taken from axum docs here: https://github.com/tokio-rs/axum/blob/main/examples/query-params-with-empty-strings/src/main.rs
/// Serde deserialization decorator to map empty Strings to None,
pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[derive(Debug, Serialize)]
pub struct FileRecordFlatten<T> {
    record: T,
    file_record: File,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Creator {
    pub creator_id: Uuid,
    pub creator_username: String,
    pub creator_full_name: String,
    #[serde(default = "utc_datetime_now")]
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    pub environment_id: Uuid,
    pub environment_name: String,
    pub environment_framework: Framework,
    pub environment_core_name: String,
    pub environment_core_version: String,
    pub environment_derived_from: Option<Uuid>,
    pub environment_config: Option<sqlx::types::JsonValue>,
    #[serde(default = "utc_datetime_now")]
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub file_id: Uuid,
    pub file_hash: String,
    pub file_filename: String,
    pub file_source_path: String,
    pub file_dest_path: String,
    pub file_size: i64, //PostgeSQL does not have native uint support
    #[serde(default = "utc_datetime_now")]
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instance {
    pub instance_id: Uuid,
    pub work_id: Uuid,
    pub environment_id: Uuid,
    pub instance_config: Option<sqlx::types::JsonValue>,
    #[serde(default = "utc_datetime_now")]
    pub created_on: DateTime<Utc>,
    pub derived_from_instance: Option<Uuid>,
    pub derived_from_state: Option<Uuid>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Screenshot {
    pub screenshot_id: Uuid,
    #[serde_as(as = "Base64")]
    pub screenshot_data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(rename_all = "lowercase", type_name = "object_role")]
#[serde(rename_all = "lowercase")]
pub enum ObjectRole {
    Content,
    Dependency,
    Config,
}
impl FromStr for ObjectRole {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "content" => Ok(ObjectRole::Content),
            "dependency" => Ok(ObjectRole::Dependency),
            "config" => Ok(ObjectRole::Config),
            _ => Err("Attempting to convert ObjectRole that does not exist."),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceObject {
    pub instance_id: Uuid,
    pub object_id: Uuid,
    pub object_role: ObjectRole,
    pub object_role_index: i32,
    pub instance_object_config: Option<sqlx::types::JsonValue>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Object {
    pub object_id: Uuid,
    pub file_id: Uuid,
    pub object_description: Option<String>,
    #[serde(default = "utc_datetime_now")]
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    #[serde(default = "default_uuid")]
    pub replay_id: Uuid,
    pub replay_name: String,
    pub replay_description: String,
    pub instance_id: Uuid,
    pub creator_id: Uuid,
    pub replay_forked_from: Option<Uuid>,
    pub file_id: Uuid,
    #[serde(default = "utc_datetime_now")]
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Save {
    pub save_id: Uuid,
    pub instance_id: Uuid,
    pub save_short_desc: String,
    pub save_description: String,
    pub file_id: Uuid,
    pub creator_id: Uuid,
    #[serde(default = "utc_datetime_now")]
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_uuid")]
    pub state_id: Uuid,
    pub instance_id: Uuid,
    pub is_checkpoint: bool,
    pub file_id: Uuid,
    pub state_name: String,
    pub state_description: String,
    pub screenshot_id: Uuid,
    pub replay_id: Option<Uuid>,
    pub creator_id: Uuid,
    pub state_replay_index: Option<i32>,
    pub state_derived_from: Option<Uuid>,
    #[serde(default = "utc_datetime_now")]
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Work {
    pub work_id: Uuid,
    pub work_name: String,
    pub work_version: String,
    pub work_platform: String,
    #[serde(default = "utc_datetime_now")]
    pub created_on: DateTime<Utc>,
    pub work_derived_from: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorStateInfo {
    pub work_id: Uuid,
    pub work_name: String,
    pub work_version: String,
    pub work_platform: String,
    pub state_id: Uuid,
    pub state_name: String,
    pub state_description: String,
    pub screenshot_id: Uuid,
    pub file_id: Uuid,
    pub instance_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorReplayInfo {
    pub work_id: Uuid,
    pub work_name: String,
    pub work_version: String,
    pub work_platform: String,
    pub replay_id: Uuid,
    pub replay_name: String,
    pub replay_description: String,
    pub file_id: Uuid,
    pub instance_id: Uuid,
}

impl Creator {
    // Join to allow for all creator home page information in one query
    pub async fn get_all_state_info(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> sqlx::Result<Vec<CreatorStateInfo>> {
        sqlx::query_as!(
            CreatorStateInfo,
            r#"SELECT
            work_id,
            work_name,
            work_version,
            work_platform,
            state_id,
            state_name,
            state_description,
            screenshot_id,
            file_id,
            instance_id
            FROM work JOIN instance USING (work_id)
            JOIN state USING (instance_id)
            WHERE state.creator_id = $1
            "#,
            id
        )
        .fetch_all(conn)
        .await
    }

    // Join to allow for all creator home page information in one query
    pub async fn get_all_replay_info(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> sqlx::Result<Vec<CreatorReplayInfo>> {
        sqlx::query_as!(
            CreatorReplayInfo,
            r#"SELECT
            work_id,
            work_name,
            work_version,
            work_platform,
            replay_id,
            replay_name,
            replay_description,
            file_id,
            instance_id
            FROM work JOIN instance USING (work_id)
            JOIN replay USING (instance_id)
            WHERE replay.creator_id = $1
            "#,
            id
        )
        .fetch_all(conn)
        .await
    }

    pub async fn get_all_states(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Vec<State>> {
        sqlx::query_as!(
            State,
            r#"SELECT state_id,
            instance_id,
            is_checkpoint,
            file_id,
            state_name,
            state_description,
            screenshot_id,
            replay_id,
            creator_id,
            state_replay_index,
            state_derived_from,
            created_on
            FROM state WHERE creator_id = $1"#,
            id
        )
        .fetch_all(conn)
        .await
    }

    pub async fn get_all_replays(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Vec<Replay>> {
        sqlx::query_as!(
            Replay,
            r#"SELECT replay_id,
            replay_name,
            replay_description,
            instance_id,
            creator_id,
            file_id,
            replay_forked_from,
            created_on
            FROM replay WHERE creator_id = $1"#,
            id
        )
        .fetch_all(conn)
        .await
    }
}

impl Creator {
    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("creator_id".to_string(), "Uuid".to_string()),
            ("creator_username".to_string(), "String".to_string()),
            ("creator_full_name".to_string(), "String".to_string()),
            ("created_on".to_string(), "DateTime<Utc>".to_string()),
        ]
    }

    pub fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.creator_id.to_string()),
            Some(self.creator_full_name.to_string()),
            Some(self.creator_username.to_string()),
            Some(self.created_on.to_string()),
        ]
    }

    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT creator_id,
            creator_username,
            creator_full_name,
            created_on
            FROM creator WHERE creator_id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn get_all(
        conn: &mut PgConnection,
        params: GetQueryParams,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT creator_id,
            creator_username,
            creator_full_name,
            created_on
            FROM creator
            WHERE POSITION($1 in creator_username) > 0 OR POSITION($1 in creator_full_name) > 0
            ORDER BY $2 ASC
            LIMIT $3
            OFFSET $4"#,
            params.contains.unwrap_or("".to_string()),
            params.order_by.unwrap_or("creator_username".to_string()),
            params.limit,
            params.offset
        )
        .fetch_all(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, model: Creator) -> Result<Self, RecordSQLError> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO creator(creator_id, creator_username, creator_full_name, created_on)
            VALUES($1, $2, $3, $4)
            RETURNING creator_id, creator_username, creator_full_name, created_on
            "#,
            model.creator_id,
            model.creator_username,
            model.creator_full_name,
            model.created_on
        )
        .fetch_one(conn)
        .await
        .map_err(|e| RecordSQLError {
            table: ErrorTable::Creator,
            action: ErrorAction::Insert,
            source: e,
        })
    }

    pub async fn update(conn: &mut PgConnection, creator: Creator) -> Result<Self, RecordSQLError> {
        sqlx::query_as!(
            Creator,
            r#"UPDATE creator SET
            (creator_username, creator_full_name, created_on) =
            ($1, $2, $3)
            WHERE creator_id = $4
            RETURNING creator_id, creator_username, creator_full_name, created_on"#,
            creator.creator_username,
            creator.creator_full_name,
            creator.created_on,
            creator.creator_id,
        )
        .fetch_one(conn)
        .await
        .map_err(|e| RecordSQLError {
            table: ErrorTable::Creator,
            action: ErrorAction::Update,
            source: e,
        })
    }
}

impl File {
    pub async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT file_id,
                file_hash,
                file_filename,
                file_dest_path,
                file_source_path,
                file_size,
                created_on
                FROM file
                WHERE file_hash = $1
                "#,
            hash
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn flatten_file(
        _conn: &mut PgConnection,
        record: Self,
    ) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        Ok(FileRecordFlatten {
            record: record.clone(),
            file_record: record,
        })
    }
}

impl File {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT file_id,
            file_hash,
            file_filename,
            file_dest_path,
            file_source_path,
            file_size,
            created_on
            FROM file WHERE file_id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, model: File) -> Result<Self, RecordSQLError> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO file(file_id, file_hash, file_filename, file_source_path, file_dest_path, file_size, created_on)
            VALUES($1, $2, $3, $4, $5, $6, $7)
            RETURNING file_id, file_hash, file_filename, file_source_path, file_dest_path, file_size, created_on
            "#,
            model.file_id,
            model.file_hash,
            model.file_filename,
            model.file_source_path,
            model.file_dest_path,
            model.file_size,
            model.created_on
        )
            .fetch_one(conn)
            .await
            .map_err(|e| RecordSQLError{
                table: ErrorTable::File,
                action: ErrorAction::Insert,
                source: e
            },)
    }
}

impl Instance {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT instance_id, environment_id, work_id, instance_config, created_on, derived_from_instance, derived_from_state
            FROM instance WHERE instance_id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, model: Instance) -> Result<Self, RecordSQLError> {
        sqlx::query_as!(
            Instance,
            r#"INSERT INTO instance(
            instance_id,
            environment_id,
            work_id,
            instance_config,
            created_on,
            derived_from_instance,
            derived_from_state
            )
            VALUES($1, $2, $3, $4, $5, $6, $7)
            RETURNING instance_id, environment_id, work_id, instance_config, created_on, derived_from_instance, derived_from_state"#,
            model.instance_id,
            model.environment_id,
            model.work_id,
            model.instance_config,
            model.created_on,
            model.derived_from_instance,
            model.derived_from_state
        )
        .fetch_one(conn)
        .await
        .map_err(|e| RecordSQLError {
            table: ErrorTable::Instance,
            action: ErrorAction::Insert,
            source: e,
        })
    }
}

impl Instance {
    pub async fn get_all_for_work_id(
        conn: &mut PgConnection,
        work_id: Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT instance_id, environment_id, work_id, instance_config, created_on, derived_from_instance, derived_from_state
            FROM instance
            WHERE work_id = $1
            "#,
            work_id
        )
        .fetch_all(conn)
        .await
    }

    pub async fn get_all_states(
        conn: &mut PgConnection,
        instance_id: Uuid,
    ) -> sqlx::Result<Vec<State>> {
        sqlx::query_as!(
            State,
            r#"SELECT state_id,
            instance_id,
            is_checkpoint,
            file_id,
            state_name,
            state_description,
            screenshot_id,
            replay_id,
            creator_id,
            state_replay_index,
            state_derived_from,
            created_on
            FROM state WHERE instance_id = $1"#,
            instance_id
        )
        .fetch_all(conn)
        .await
    }

    pub async fn get_all_replays(
        conn: &mut PgConnection,
        instance_id: Uuid,
    ) -> sqlx::Result<Vec<Replay>> {
        sqlx::query_as!(
            Replay,
            r#"SELECT replay_id,
            replay_name,
            replay_description,
            instance_id,
            creator_id,
            file_id,
            replay_forked_from,
            created_on
            FROM replay WHERE instance_id = $1"#,
            instance_id
        )
        .fetch_all(conn)
        .await
    }

    pub async fn get_all_saves(
        conn: &mut PgConnection,
        instance_id: Uuid,
    ) -> sqlx::Result<Vec<Save>> {
        sqlx::query_as!(
            Save,
            r#"SELECT
            save_id,
            instance_id,
            save_short_desc,
            save_description,
            file_id,
            creator_id,
            created_on
            FROM save WHERE instance_id = $1"#,
            instance_id
        )
        .fetch_all(conn)
        .await
    }
}

impl Environment {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT environment_id, environment_name, environment_framework as "environment_framework:_", environment_core_name, environment_core_version, environment_derived_from, environment_config, created_on
            FROM environment
            WHERE environment_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn insert(
        conn: &mut PgConnection,
        model: Environment,
    ) -> Result<Self, RecordSQLError> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO environment (environment_id, environment_name, environment_framework, environment_core_name, environment_core_version, environment_derived_from, environment_config, created_on)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING environment_id, environment_name, environment_framework as "environment_framework:_", environment_core_name, environment_core_version, environment_derived_from, environment_config, created_on
            "#,
            model.environment_id,
            model.environment_name,
            model.environment_framework as _,
            model.environment_core_name,
            model.environment_core_version,
            model.environment_derived_from,
            model.environment_config,
            model.created_on
        )
            .fetch_one(conn)
            .await
            .map_err(|e| RecordSQLError{
                table: ErrorTable::Environment,
                action: ErrorAction::Insert,
                source: e
            },)
    }
}

impl Object {
    pub async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT object.object_id, object.file_id, object.object_description, object.created_on
            FROM object
            JOIN file USING(file_id)
            WHERE file.file_hash = $1"#,
            hash
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn flatten_file(
        conn: &mut PgConnection,
        model: Self,
    ) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        let file_record = File::get_by_id(conn, model.file_id).await?.unwrap();
        Ok(FileRecordFlatten {
            record: model,
            file_record,
        })
    }
}

impl Object {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT object_id, file_id, object_description, created_on
            FROM object
            WHERE object_id = $1"#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, object: Object) -> Result<Self, RecordSQLError> {
        // Note: the "!" following the AS statements after RETURNING are forcing not-null status on those fields
        // from: https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns
        sqlx::query_as!(
            Object,
            r#"INSERT INTO object (
            object_id, file_id, object_description, created_on )
            VALUES ($1, $2, $3, current_timestamp)
            RETURNING object_id, file_id, object_description, created_on
            "#,
            object.object_id,
            object.file_id,
            object.object_description
        )
        .fetch_one(conn)
        .await
        .map_err(|e| RecordSQLError {
            table: ErrorTable::Object,
            action: ErrorAction::Insert,
            source: e,
        })
    }
}

impl Object {
    pub async fn link_object_to_instance(
        conn: &mut PgConnection,
        object_id: Uuid,
        instance_id: Uuid,
        role: ObjectRole,
        role_index: usize,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"INSERT INTO instanceObject(instance_id, object_id, object_role, object_role_index)  VALUES ($1, $2, $3, $4)"#,
            instance_id,
            object_id,
            role as _,
            role_index as i32
        )
        .execute(conn)
        .await
    }

    pub async fn get_object_instance_by_ids(
        conn: &mut PgConnection,
        object_id: Uuid,
        instance_id: Uuid,
    ) -> sqlx::Result<Option<InstanceObject>> {
        sqlx::query_as!(InstanceObject, r#"SELECT object_id, instance_id, instance_object_config, object_role as "object_role:_", object_role_index FROM instanceObject WHERE object_id = $1 AND instance_id = $2"#, object_id, instance_id).fetch_optional(conn).await
    }

    pub async fn delete_object_instance_links_by_id(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM instanceObject WHERE object_id = $1", id)
            .execute(conn)
            .await
    }
}

impl Replay {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT replay_id,
            replay_name,
            replay_description,
            instance_id,
            creator_id,
            file_id,
            replay_forked_from,
            created_on
            FROM replay WHERE replay_id = $1
            "#,
            id,
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, model: Self) -> Result<Self, RecordSQLError> {
        sqlx::query_as!(
            Replay,
            r#"INSERT INTO replay (
            replay_id,
            replay_name,
            replay_description,
            instance_id,
            creator_id,
            file_id,
            replay_forked_from,
            created_on
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
            replay_id,
            replay_name,
            replay_description,
            instance_id,
            creator_id,
            file_id,
            replay_forked_from,
            created_on
            "#,
            model.replay_id,
            model.replay_name,
            model.replay_description,
            model.instance_id,
            model.creator_id,
            model.file_id,
            model.replay_forked_from,
            model.created_on,
        )
        .fetch_one(conn)
        .await
        .map_err(|e| RecordSQLError {
            table: ErrorTable::Replay,
            action: ErrorAction::Insert,
            source: e,
        })
    }
}

impl Replay {
    pub async fn flatten_file(
        conn: &mut PgConnection,
        model: Self,
    ) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        let file_record = File::get_by_id(conn, model.file_id).await?.unwrap();
        Ok(FileRecordFlatten {
            record: model,
            file_record,
        })
    }
}

impl Save {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT
            save_id,
            instance_id,
            save_short_desc,
            save_description,
            file_id,
            creator_id,
            created_on
            FROM save WHERE save_id = $1"#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, model: Self) -> Result<Self, RecordSQLError> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO save (
            save_id,
            instance_id,
            save_short_desc,
            save_description,
            file_id,
            creator_id,
            created_on
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
            save_id,
            instance_id,
            save_short_desc,
            save_description,
            file_id,
            creator_id,
            created_on
            "#,
            model.save_id,
            model.instance_id,
            model.save_short_desc,
            model.save_description,
            model.file_id,
            model.creator_id,
            model.created_on,
        )
        .fetch_one(conn)
        .await
        .map_err(|e| RecordSQLError {
            table: ErrorTable::Save,
            action: ErrorAction::Insert,
            source: e,
        })
    }
}

impl Save {
    pub async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT
            save.save_id,
            save.instance_id,
            save.save_short_desc,
            save.save_description,
            save.file_id,
            save.creator_id,
            save.created_on
            FROM save
            JOIN file USING (file_id)
            WHERE file.file_hash = $1
            "#,
            hash
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn flatten_file(
        conn: &mut PgConnection,
        model: Self,
    ) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        let file_record = File::get_by_id(conn, model.file_id).await?.unwrap();
        Ok(FileRecordFlatten {
            record: model,
            file_record,
        })
    }
}

impl State {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT state_id,
            instance_id,
            is_checkpoint,
            file_id,
            state_name,
            state_description,
            screenshot_id,
            replay_id,
            creator_id,
            state_replay_index,
            state_derived_from,
            created_on
            FROM state WHERE state_id = $1
            "#,
            id,
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, state: Self) -> Result<Self, RecordSQLError> {
        // Note: the "!" following the AS statements after RETURNING are forcing not-null status on those fields
        // from: https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns
        sqlx::query_as!(
            State,
            r#"INSERT INTO state (
            state_id,
            instance_id,
            is_checkpoint,
            file_id,
            state_name,
            state_description,
            screenshot_id,
            replay_id,
            creator_id,
            state_replay_index,
            state_derived_from,
            created_on
 )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING state_id,
            instance_id,
            is_checkpoint,
            file_id,
            state_name,
            state_description,
            screenshot_id,
            replay_id,
            creator_id,
            state_replay_index,
            state_derived_from,
            created_on
            "#,
            state.state_id,
            state.instance_id,
            state.is_checkpoint,
            state.file_id,
            state.state_name,
            state.state_description,
            state.screenshot_id,
            state.replay_id,
            state.creator_id,
            state.state_replay_index,
            state.state_derived_from,
            state.created_on,
        )
        .fetch_one(conn)
        .await
        .map_err(|e| RecordSQLError {
            table: ErrorTable::State,
            action: ErrorAction::Insert,
            source: e,
        })
    }
}

impl State {
    pub async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT state.state_id,
                state.instance_id,
                state.is_checkpoint,
                state.file_id,
                state.state_name,
                state.state_description,
                state.screenshot_id,
                state.replay_id,
                state.creator_id,
                state.state_replay_index,
                state.state_derived_from,
                state.created_on
                FROM state
                JOIN file USING (file_id)
                WHERE file.file_hash = $1
                "#,
            hash
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn flatten_file(
        conn: &mut PgConnection,
        model: Self,
    ) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        let file_record = File::get_by_id(conn, model.file_id).await?.unwrap();
        Ok(FileRecordFlatten {
            record: model,
            file_record,
        })
    }
}

impl Work {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform, created_on, work_derived_from FROM work WHERE work_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn insert(conn: &mut PgConnection, work: Self) -> Result<Self, RecordSQLError> {
        // Note: the "!" following the AS statements after RETURNING are forcing not-null status on those fields
        // from: https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns
        sqlx::query_as!(
            Work,
            r#"INSERT INTO work (
            work_id, work_name, work_version, work_platform, created_on, work_derived_from )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING work_id, work_name, work_version, work_platform, created_on, work_derived_from
            "#,
            work.work_id,
            work.work_name,
            work.work_version,
            work.work_platform,
            work.created_on,
            work.work_derived_from
        )
        .fetch_one(conn)
        .await
        .map_err(|e| RecordSQLError {
            table: ErrorTable::Work,
            action: ErrorAction::Insert,
            source: e,
        })
    }
}

impl Work {
    pub async fn get_by_name(conn: &mut PgConnection, name: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform, created_on, work_derived_from FROM work WHERE work_name = $1"#,
            name
        )
            .fetch_all(conn)
            .await
    }

    pub async fn get_works_for_platform(
        conn: &mut PgConnection,
        platform: &str,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform, created_on, work_derived_from FROM work WHERE work_platform = $1"#,
            platform
        )
            .fetch_all(conn)
            .await
    }
}

impl Screenshot {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Screenshot,
            r#" SELECT screenshot_id, screenshot_data FROM screenshot
            WHERE screenshot_id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, model: Self) -> Result<Self, RecordSQLError> {
        sqlx::query_as!(
            Screenshot,
            r#"INSERT INTO screenshot (screenshot_id, screenshot_data) VALUES ($1, $2)
            RETURNING screenshot_id, screenshot_data
            "#,
            model.screenshot_id,
            model.screenshot_data
        )
        .fetch_one(conn)
        .await
        .map_err(|e| RecordSQLError {
            table: ErrorTable::Screenshot,
            action: ErrorAction::Insert,
            source: e,
        })
    }
}

pub fn default_uuid() -> Uuid {
    Uuid::new_v4()
}

fn utc_datetime_now() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Serialize)]
pub struct InstanceWork {
    pub work_id: Uuid,
    pub work_name: String,
    pub work_version: String,
    pub work_platform: String,
    pub instance_id: Uuid,
}

impl InstanceWork {
    pub async fn get_all(
        conn: &mut sqlx::PgConnection,
        params: GetQueryParams,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT work.work_id, work.work_name, work.work_version, work.work_platform, instance.instance_id
            FROM work
            JOIN instance USING(work_id)
            WHERE POSITION($1 in work.work_name) > 0
            ORDER BY $2 ASC
            LIMIT $3
            OFFSET $4
            "#,
            params.contains.unwrap_or("".to_string()),
            params.order_by.unwrap_or("work.work_name".to_string()),
            params.limit,
            params.offset,
        )
        .fetch_all(conn)
        .await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectLink {
    pub object_id: Uuid,
    pub object_role: ObjectRole,
    pub object_role_index: i32,
    pub file_hash: String,
    pub file_filename: String,
    pub file_source_path: String,
    pub file_dest_path: String,
}
impl ObjectLink {
    pub async fn get_all_for_instance_id(
        conn: &mut sqlx::PgConnection,
        id: Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT object_id, instanceObject.object_role as "object_role:_", instanceObject.object_role_index, file.file_hash as file_hash, file.file_filename as file_filename, file.file_source_path as file_source_path, file.file_dest_path as file_dest_path
            FROM object
            JOIN instanceObject USING(object_id)
            JOIN instance USING(instance_id)
            JOIN file USING(file_id)
            WHERE instance_id = $1
            "#,
            id
        )
        .fetch_all(conn)
        .await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplayLink {
    pub replay_id: Uuid,
    pub replay_name: String,
    pub replay_description: String,
    pub instance_id: Uuid,
    pub creator_id: Uuid,
    pub replay_forked_from: Option<Uuid>,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
    pub file_id: Uuid,
    pub file_hash: String,
    pub file_filename: String,
    pub file_source_path: String,
    pub file_dest_path: String,
}
impl ReplayLink {
    pub async fn get_by_id(conn: &mut sqlx::PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT replay_id,
            replay_name,
            replay_description,
            instance_id,
            creator_id,
            replay_forked_from,
            replay.created_on,
            file_id,
            file.file_hash as file_hash,
            file.file_filename as file_filename,
            file.file_source_path as file_source_path,
            file.file_dest_path as file_dest_path
            FROM replay
            JOIN file USING(file_id)
            WHERE replay_id = $1
            "#,
            id,
        )
        .fetch_optional(conn)
        .await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateLink {
    pub state_id: Uuid,
    pub instance_id: Uuid,
    pub is_checkpoint: bool,
    pub state_name: String,
    pub state_description: String,
    pub screenshot_id: Option<Uuid>,
    pub replay_id: Option<Uuid>,
    pub creator_id: Option<Uuid>,
    pub state_replay_index: Option<i32>,
    pub state_derived_from: Option<Uuid>,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
    pub file_id: Uuid,
    pub file_hash: String,
    pub file_filename: String,
    pub file_source_path: String,
    pub file_dest_path: String,
}
impl StateLink {
    pub async fn get_by_id(conn: &mut sqlx::PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT
            state_id,
            instance_id,
            is_checkpoint,
            state_name,
            state_description,
            screenshot_id,
            replay_id,
            creator_id,
            state_replay_index,
            state_derived_from,
            state.created_on,
            file_id,
            file.file_hash as file_hash,
            file.file_filename as file_filename,
            file.file_source_path as file_source_path,
            file.file_dest_path as file_dest_path
            FROM state
            JOIN file USING(file_id)
            WHERE state_id = $1
            "#,
            id,
        )
        .fetch_optional(conn)
        .await
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_file_object(
    conn: &mut sqlx::PgConnection,
    storage_root: &str,
    depth: u8,
    path: &std::path::Path,
    filename_override: Option<String>,
    object_description: Option<String>,
    file_source_path: String,
    duplicate: Duplicate,
) -> Result<Uuid, crate::error::InsertFileError> {
    use crate::error::InsertFileError;
    use crate::storage::StorageHandler;
    use tracing::info;
    let file_name = filename_override
        .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string());
    let created_on = chrono::Utc::now();
    let file_size = std::fs::metadata(path)?.len() as i64;
    let hash = StorageHandler::get_file_hash(path)?;
    if let Some(file_info) = File::get_by_hash(conn, &hash).await? {
        let object_id = match duplicate {
            Duplicate::ReuseData => {
                info!("adding duplicate file record for {path:?}");
                let file_id = Uuid::new_v4();
                let file_record = File {
                    file_id,
                    file_hash: file_info.file_hash,
                    file_filename: file_name,
                    file_source_path,
                    file_dest_path: file_info.file_dest_path,
                    file_size: file_info.file_size,
                    created_on,
                };
                File::insert(conn, file_record).await?;
                let object_id = Uuid::new_v4();
                let object = Object {
                    object_id,
                    file_id,
                    object_description,
                    created_on,
                };
                Object::insert(conn, object).await?;
                Some(object_id)
            }
            Duplicate::ReuseObject => {
                info!("skipping duplicate record for {path:?}, reusing object");
                Object::get_by_hash(conn, &file_info.file_hash)
                    .await?
                    .map(|o| o.object_id)
            }
        };
        object_id.ok_or(InsertFileError::ObjectMissing(hash))
    } else {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let file_uuid = Uuid::new_v4();
        info!("Do write file {file_name}");
        let file_info = StorageHandler::write_file_to_uuid_folder(
            storage_root,
            depth,
            file_uuid,
            &file_name,
            path,
        )
        .await?;
        info!(
            "Wrote file {} to {}",
            file_info.dest_filename, file_info.dest_path
        );
        let file_record = File {
            file_id: file_uuid,
            file_hash: file_info.file_hash,
            file_filename: file_info.source_filename,
            file_source_path,
            file_dest_path: file_info.dest_path,
            file_size,
            created_on,
        };
        File::insert(conn, file_record).await?;
        let object_id = Uuid::new_v4();
        let object = Object {
            object_id,
            file_id: file_uuid,
            object_description,
            created_on,
        };
        Object::insert(conn, object).await?;
        Ok(object_id)
    }
}

pub enum Duplicate {
    ReuseObject,
    ReuseData,
}

#[derive(Deserialize)]
pub struct GetQueryParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub order_by: Option<String>,
    pub contains: Option<String>,
}
