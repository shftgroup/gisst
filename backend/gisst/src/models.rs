#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use serde::{Deserialize, Deserializer, Serialize, de};
use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use sqlx::postgres::{PgConnection, PgQueryResult};

use crate::{model_enums::Framework, search::SearchIndexer};
use serde_with::{base64::Base64, serde_as};
use uuid::Uuid;

use crate::error::{Action, Insert, RecordSQL, Table};

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
    pub file_compressed_size: Option<i64>,
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
    pub state_derived_from: Option<Uuid>,
    pub save_derived_from: Option<Uuid>,
    pub replay_derived_from: Option<Uuid>,
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
    pub save_derived_from: Option<Uuid>,
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
    pub created_on: DateTime<Utc>,
    pub creator_id: Uuid,
    pub creator_username: String,
    pub creator_full_name: String,
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
    pub created_on: DateTime<Utc>,
    pub creator_id: Uuid,
    pub creator_username: String,
    pub creator_full_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorSaveInfo {
    pub work_id: Uuid,
    pub work_name: String,
    pub work_version: String,
    pub work_platform: String,
    pub save_id: Uuid,
    pub save_short_desc: String,
    pub save_description: String,
    pub file_id: Uuid,
    pub instance_id: Uuid,
    pub created_on: DateTime<Utc>,
    pub creator_id: Uuid,
    pub creator_username: String,
    pub creator_full_name: String,
}

impl CreatorSaveInfo {
    pub async fn get_for_save(conn: &mut PgConnection, save: &Save) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT work.work_id, work.work_name, work.work_version, work.work_platform,
                      save_id, save_short_desc, save_description,
                      file_id, instance_save.instance_id, save.created_on, creator.creator_id,
                      creator.creator_username, creator.creator_full_name
               FROM instance_save
                    JOIN save USING (save_id)
                    JOIN instance ON (instance_save.instance_id = instance.instance_id)
                    JOIN work ON (work.work_id = instance.work_id)
                    JOIN creator ON (save.creator_id = creator.creator_id)
               WHERE save.save_id = $1
               LIMIT 1000"#,
            save.save_id
        )
        .fetch_all(conn)
        .await
    }
    pub fn get_stream(conn: &mut sqlx::PgConnection) -> impl futures::Stream<Item = Self> {
        use futures::StreamExt;
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform,
                      save_id, save_short_desc, save_description,
                      file_id, instance_id, save.created_on, creator.creator_id,
                      creator.creator_username, creator.creator_full_name
               FROM work JOIN instance USING (work_id) JOIN save USING (instance_id) JOIN creator ON (save.creator_id = creator.creator_id)"#
        ).fetch(conn).filter_map(|f| futures::future::ready(f.ok()))
    }
}
impl CreatorStateInfo {
    pub async fn get_for_state(conn: &mut PgConnection, state: &State) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform,
                      state_id, state_name, state_description, state.screenshot_id,
                      file_id, instance_id, state.created_on, creator.creator_id,
                      creator.creator_username, creator.creator_full_name
               FROM work JOIN instance USING (work_id) JOIN state USING (instance_id) JOIN creator ON (state.creator_id = creator.creator_id)
               WHERE state.state_id = $1"#,
            state.state_id
        ).fetch_one(conn).await
    }
    pub fn get_stream(conn: &mut sqlx::PgConnection) -> impl futures::Stream<Item = Self> {
        use futures::StreamExt;
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform,
                      state_id, state_name, state_description, state.screenshot_id,
                      file_id, instance_id, state.created_on, creator.creator_id,
                      creator.creator_username, creator.creator_full_name
               FROM work JOIN instance USING (work_id) JOIN state USING (instance_id) JOIN creator ON (state.creator_id = creator.creator_id)"#
        ).fetch(conn).filter_map(|f| futures::future::ready(f.ok()))
    }
}
impl CreatorReplayInfo {
    pub async fn get_for_replay(conn: &mut PgConnection, replay: &Replay) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform,
                      replay_id, replay_name, replay_description,
                      file_id, instance_id, replay.created_on, creator.creator_id,
                      creator.creator_username, creator.creator_full_name
               FROM work JOIN instance USING (work_id) JOIN replay USING (instance_id) JOIN creator ON (replay.creator_id = creator.creator_id)
               WHERE replay.replay_id = $1"#,
            replay.replay_id
        ).fetch_one(conn).await
    }
    pub fn get_stream(conn: &mut sqlx::PgConnection) -> impl futures::Stream<Item = Self> {
        use futures::StreamExt;
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform,
                      replay_id, replay_name, replay_description,
                      file_id, instance_id, replay.created_on, creator.creator_id,
                      creator.creator_username, creator.creator_full_name
               FROM work JOIN instance USING (work_id) JOIN replay USING (instance_id) JOIN creator ON (replay.creator_id = creator.creator_id)"#
        ).fetch(conn).filter_map(|f| futures::future::ready(f.ok()))
    }
}

impl Creator {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT * FROM creator WHERE creator_id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }
    pub fn get_stream(conn: &mut sqlx::PgConnection) -> impl futures::Stream<Item = Self> {
        use futures::StreamExt;
        sqlx::query_as!(Self, r#"SELECT * FROM creator"#)
            .fetch(conn)
            .filter_map(|f| futures::future::ready(f.ok()))
    }
    pub async fn insert(
        conn: &mut PgConnection,
        model: Creator,
        indexer: &impl SearchIndexer,
    ) -> Result<Self, Insert> {
        let record = sqlx::query_as!(
            Self,
            r#"INSERT INTO creator VALUES($1, $2, $3, $4) RETURNING *
            "#,
            model.creator_id,
            model.creator_username,
            model.creator_full_name,
            model.created_on
        )
        .fetch_one(conn.as_mut())
        .await
        .map_err(|e| RecordSQL {
            table: Table::Creator,
            action: Action::Insert,
            source: e,
        })?;
        indexer.upsert_creator(conn, &record).await?;
        Ok(record)
    }
}

impl File {
    pub async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, r#"SELECT * FROM file WHERE file_hash = $1"#, hash)
            .fetch_optional(conn)
            .await
    }
}

impl File {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, r#"SELECT * FROM file WHERE file_id = $1"#, id)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert(conn: &mut PgConnection, model: File) -> Result<Self, Insert> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO file VALUES($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *
            "#,
            model.file_id,
            model.file_hash,
            model.file_filename,
            model.file_source_path,
            model.file_dest_path,
            model.file_size,
            model.created_on,
            model.file_compressed_size,
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            Insert::Sql(RecordSQL {
                table: Table::File,
                action: Action::Insert,
                source: e,
            })
        })
    }
}

impl Instance {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, r#"SELECT * FROM instance WHERE instance_id = $1"#, id)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert(
        conn: &mut PgConnection,
        model: Instance,
        indexer: &impl crate::search::SearchIndexer,
    ) -> Result<Self, Insert> {
        let record = sqlx::query_as!(
            Instance,
            r#"INSERT INTO instance VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING *"#,
            model.instance_id,
            model.environment_id,
            model.work_id,
            model.instance_config,
            model.created_on,
            model.derived_from_instance,
            model.derived_from_state
        )
        .fetch_one(conn.as_mut())
        .await
        .map_err(|e| RecordSQL {
            table: Table::Instance,
            action: Action::Insert,
            source: e,
        })?;
        indexer.upsert_instance(conn, &record).await?;
        Ok(record)
    }
}

impl Instance {
    pub async fn get_all_for_work_id(
        conn: &mut PgConnection,
        work_id: Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT * FROM instance WHERE work_id = $1"#,
            work_id
        )
        .fetch_all(conn)
        .await
    }
}

impl Environment {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT environment_id, environment_name,
                  environment_framework as "environment_framework:_",
                  environment_core_name, environment_core_version,
                  environment_derived_from, environment_config, created_on
               FROM environment
               WHERE environment_id = $1"#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, model: Environment) -> Result<Self, Insert> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO environment
               VALUES($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING environment_id, environment_name,
                  environment_framework as "environment_framework:_",
                  environment_core_name, environment_core_version,
                  environment_derived_from, environment_config, created_on"#,
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
        .map_err(|e| {
            Insert::Sql(RecordSQL {
                table: Table::Environment,
                action: Action::Insert,
                source: e,
            })
        })
    }
}

impl Object {
    pub async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT object.* FROM object JOIN file USING(file_id) WHERE file.file_hash = $1"#,
            hash
        )
        .fetch_optional(conn)
        .await
    }
}

impl Object {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, r#"SELECT * FROM object WHERE object_id = $1"#, id)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert(conn: &mut PgConnection, object: Object) -> Result<Self, Insert> {
        // Note: the "!" following the AS statements after RETURNING are forcing not-null status on those fields
        // from: https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns
        sqlx::query_as!(
            Object,
            r#"INSERT INTO object VALUES ($1, $2, $3, current_timestamp) RETURNING *"#,
            object.object_id,
            object.file_id,
            object.object_description
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            Insert::Sql(RecordSQL {
                table: Table::Object,
                action: Action::Insert,
                source: e,
            })
        })
    }
}

impl Object {
    pub async fn link_object_to_instance(
        conn: &mut PgConnection,
        object_id: Uuid,
        instance_id: Uuid,
        role: ObjectRole,
        role_index: u16,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"INSERT INTO instanceObject VALUES ($1, $2, $3, null, $4)"#,
            instance_id,
            object_id,
            role as _,
            i32::from(role_index)
        )
        .execute(conn)
        .await
    }

    pub async fn get_object_instance_by_ids(
        conn: &mut PgConnection,
        object_id: Uuid,
        instance_id: Uuid,
    ) -> sqlx::Result<Option<InstanceObject>> {
        sqlx::query_as!(
            InstanceObject,
            r#"SELECT object_id, instance_id, instance_object_config,
                  object_role as "object_role:_", object_role_index
               FROM instanceObject
               WHERE object_id = $1 AND instance_id = $2"#,
            object_id,
            instance_id
        )
        .fetch_optional(conn)
        .await
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
        sqlx::query_as!(Self, r#"SELECT * FROM replay WHERE replay_id = $1"#, id,)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert(
        conn: &mut PgConnection,
        model: Self,
        indexer: &impl crate::search::SearchIndexer,
    ) -> Result<Self, Insert> {
        let record = sqlx::query_as!(
            Replay,
            r#"INSERT INTO replay VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#,
            model.replay_id,
            model.replay_name,
            model.replay_description,
            model.instance_id,
            model.creator_id,
            model.replay_forked_from,
            model.file_id,
            model.created_on,
        )
        .fetch_one(conn.as_mut())
        .await
        .map_err(|e| RecordSQL {
            table: Table::Replay,
            action: Action::Insert,
            source: e,
        })?;
        indexer.upsert_replay(conn, &record).await?;
        Ok(record)
    }
}

impl Save {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, r#"SELECT * FROM save WHERE save_id = $1"#, id)
            .fetch_optional(conn)
            .await
    }
    pub async fn insert(
        conn: &mut PgConnection,
        model: Self,
        indexer: &impl crate::search::SearchIndexer,
    ) -> Result<Self, Insert> {
        let result =         // First, insert the new save
        sqlx::query_as!(
            Self,
            r#"INSERT INTO save VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *"#,
            model.save_id,
            model.instance_id,
            model.save_short_desc,
            model.save_description,
            model.file_id,
            model.creator_id,
            model.created_on,
            model.state_derived_from,
            model.save_derived_from,
            model.replay_derived_from
        )
        .fetch_one(conn.as_mut())
        .await
        .map_err(|e| RecordSQL {
            table: Table::Save,
            action: Action::Insert,
            source: e,
        })?;
        /* ensure instance_save is updated! if this is derived from
         * another save, copy over all the records, and also make sure
         * this instance is linked to this save */
        if let Some(save_parent_id) = model.save_derived_from {
            let parent_instances: Vec<Uuid> = sqlx::query_scalar!(
                r#"SELECT instance_id FROM save WHERE save_id=$1"#,
                save_parent_id
            )
            .fetch_all(conn.as_mut())
            .await
            .map_err(|e| RecordSQL {
                table: Table::Save,
                action: Action::Insert,
                source: e,
            })?;
            let save_count = i32::try_from(parent_instances.len()).unwrap_or_else(|_e| {
                tracing::error!(
                    "Single sram has {} instances, which is more than 2^16; instances won't be copied over", parent_instances.len()
                );
                0
            });
            if save_count > 0 {
                sqlx::query!(
                    r#"INSERT INTO instance_save (instance_id, save_id)
                     SELECT * FROM UNNEST($1::uuid[], array_fill($2::uuid, array[$3]::integer[])::uuid[])"#,
                    &parent_instances,
                    model.save_id,
                    save_count
                )
                    .execute(conn.as_mut())
                    .await
                    .map_err(|e| RecordSQL {
                        table: Table::Save,
                        action: Action::Insert,
                        source: e,
                    })?;
            }
        }
        sqlx::query!(
            r#"INSERT INTO instance_save (instance_id, save_id) VALUES ($1, $2)"#,
            model.instance_id,
            model.save_id
        )
        .execute(conn.as_mut())
        .await
        .map_err(|e| RecordSQL {
            table: Table::Save,
            action: Action::Insert,
            source: e,
        })?;
        indexer.upsert_save(conn, &result).await?;
        Ok(result)
    }
    pub async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT
            save.*
            FROM save
            JOIN file USING (file_id)
            WHERE file.file_hash = $1
            "#,
            hash
        )
        .fetch_optional(conn)
        .await
    }
}

impl State {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, r#"SELECT * FROM state WHERE state_id = $1"#, id,)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert(
        conn: &mut PgConnection,
        state: Self,
        indexer: &impl crate::search::SearchIndexer,
    ) -> Result<Self, Insert> {
        let record = sqlx::query_as!(
            State,
            r#"INSERT INTO state VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
               RETURNING *"#,
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
            state.save_derived_from,
        )
        .fetch_one(conn.as_mut())
        .await
        .map_err(|e| RecordSQL {
            table: Table::State,
            action: Action::Insert,
            source: e,
        })?;
        indexer.upsert_state(conn, &record).await?;
        Ok(record)
    }
}

impl State {
    pub async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT state.* FROM state JOIN file USING (file_id) WHERE file.file_hash = $1"#,
            hash
        )
        .fetch_optional(conn)
        .await
    }
}

impl Work {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, r#"SELECT * FROM work WHERE work_id = $1"#, id)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert(conn: &mut PgConnection, work: Self) -> Result<Self, Insert> {
        sqlx::query_as!(
            Work,
            r#"INSERT INTO work VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
            work.work_id,
            work.work_name,
            work.work_version,
            work.work_platform,
            work.created_on,
            work.work_derived_from
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            Insert::Sql(RecordSQL {
                table: Table::Work,
                action: Action::Insert,
                source: e,
            })
        })
    }
}

impl Work {
    pub async fn get_by_name(conn: &mut PgConnection, name: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(Self, r#"SELECT * FROM work WHERE work_name = $1"#, name)
            .fetch_all(conn)
            .await
    }

    pub async fn get_works_for_platform(
        conn: &mut PgConnection,
        platform: &str,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT * FROM work WHERE work_platform = $1"#,
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
            r#" SELECT * FROM screenshot WHERE screenshot_id = $1"#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn insert(conn: &mut PgConnection, model: Self) -> Result<Self, Insert> {
        sqlx::query_as!(
            Screenshot,
            r#"INSERT INTO screenshot VALUES ($1, $2) RETURNING *"#,
            model.screenshot_id,
            model.screenshot_data
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            Insert::Sql(RecordSQL {
                table: Table::Screenshot,
                action: Action::Insert,
                source: e,
            })
        })
    }
}

#[must_use]
pub fn default_uuid() -> Uuid {
    Uuid::new_v4()
}

#[must_use]
fn utc_datetime_now() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceWork {
    pub work_id: Uuid,
    pub work_name: String,
    pub work_version: String,
    pub work_platform: String,
    pub instance_id: Uuid,
    pub row_num: i64,
}

impl InstanceWork {
    pub fn get_stream(conn: &mut sqlx::PgConnection) -> impl futures::Stream<Item = Self> {
        use futures::StreamExt;
        sqlx::query_as!(
            Self,
            r#"SELECT work_id as "work_id!", work_name as "work_name!",
                      work_version as "work_version!", work_platform as "work_platform!",
                      instance_id as "instance_id!", row_num as "row_num!"
               FROM instancework"#
        )
        .fetch(conn)
        .filter_map(|f| futures::future::ready(f.ok()))
    }
    pub async fn get_for_instance(
        conn: &mut sqlx::PgConnection,
        instance_id: Uuid,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id as "work_id!", work_name as "work_name!",
               work_version as "work_version!", work_platform as "work_platform!",
               instance_id as "instance_id!", row_num as "row_num!"
               FROM instancework WHERE instance_id=$1"#,
            instance_id
        )
        .fetch_one(conn)
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
            r#"SELECT object_id, instanceObject.object_role as "object_role:_",
                   instanceObject.object_role_index,
                   file.file_hash, file.file_filename,
                   file.file_source_path, file.file_dest_path
               FROM object
               JOIN instanceObject USING(object_id)
               JOIN instance USING(instance_id)
               JOIN file USING(file_id)
               WHERE instance_id = $1"#,
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
            r#"SELECT replay.*,
                      file.file_hash, file.file_filename,
                      file.file_source_path, file.file_dest_path
               FROM replay
               JOIN file USING(file_id)
               WHERE replay_id = $1"#,
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
    pub save_derived_from: Option<Uuid>,
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
            r#"SELECT state.*,
                      file.file_hash, file.file_filename,
                      file.file_source_path, file.file_dest_path
               FROM state
               JOIN file USING(file_id)
               WHERE state_id = $1"#,
            id,
        )
        .fetch_optional(conn)
        .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveLink {
    pub save_id: Uuid,
    pub instance_id: Uuid,
    pub save_short_desc: String,
    pub save_description: String,
    pub creator_id: Uuid,
    pub save_derived_from: Option<Uuid>,
    pub state_derived_from: Option<Uuid>,
    pub replay_derived_from: Option<Uuid>,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
    pub file_id: Uuid,
    pub file_hash: String,
    pub file_filename: String,
    pub file_source_path: String,
    pub file_dest_path: String,
}

impl SaveLink {
    pub async fn get_by_id(
        conn: &mut sqlx::PgConnection,
        save_id: Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT save.*,
                      file.file_hash, file.file_filename,
                      file.file_source_path, file.file_dest_path
               FROM save
               JOIN file USING(file_id)
               WHERE save_id = $1"#,
            save_id,
        )
        .fetch_optional(conn)
        .await
    }
    pub async fn get_by_ids(
        conn: &mut sqlx::PgConnection,
        save_ids: &[Uuid],
    ) -> sqlx::Result<Vec<Self>> {
        let mut results = sqlx::query_as!(
            Self,
            r#"SELECT save.*,
                      file.file_hash, file.file_filename,
                      file.file_source_path, file.file_dest_path
               FROM save
               JOIN file USING(file_id)
               WHERE save_id = ANY($1)"#,
            save_ids,
        )
        .fetch_all(conn)
        .await?;
        results.sort_by_key(|save| save_ids.iter().position(|sid| *sid == save.save_id));
        Ok(results)
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(conn))]
pub async fn insert_file_object(
    conn: &mut sqlx::PgConnection,
    storage_root: &str,
    depth: u8,
    path: &std::path::Path,
    filename_override: Option<String>,
    object_description: Option<String>,
    file_source_path: String,
    duplicate: Duplicate,
) -> Result<Uuid, crate::error::InsertFile> {
    use crate::error::InsertFile;
    use crate::inc_metric;
    use crate::storage::StorageHandler;
    use tracing::info;
    inc_metric!(conn, ins_file_attempts, 1);
    let file_name = path
        .file_name()
        .ok_or_else(|| InsertFile::Path(path.to_path_buf()))?
        .to_string_lossy()
        .to_string();
    if !path.exists() {
        return Err(InsertFile::Path(path.to_path_buf()));
    }
    let file_name = filename_override.unwrap_or(file_name);
    let created_on = chrono::Utc::now();
    let hash = StorageHandler::get_file_hash(path)?;
    let object_id = if let Duplicate::ForceUuid(object_id) = duplicate {
        object_id
    } else {
        Uuid::new_v4()
    };

    if let Some(file_info) = File::get_by_hash(conn, &hash).await? {
        let object_id = match duplicate {
            Duplicate::ReuseData | Duplicate::ForceUuid(_) => {
                info!("adding duplicate file record for {path:?}");
                inc_metric!(conn, ins_duplicate_reused_data, 1);
                let file_id = Uuid::new_v4();
                let file_record = File {
                    file_id,
                    file_filename: file_name,
                    file_source_path,
                    created_on,
                    ..file_info
                };
                File::insert(conn, file_record).await?;
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
                inc_metric!(conn, ins_duplicate_reused_object, 1);
                Object::get_by_hash(conn, &file_info.file_hash)
                    .await?
                    .map(|o| o.object_id)
            }
        };
        object_id.ok_or(InsertFile::ObjectMissing(hash))
    } else {
        let file_uuid = Uuid::new_v4();
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
        inc_metric!(conn, ins_new_file, 1);
        let file_record = File {
            file_id: file_uuid,
            file_hash: file_info.file_hash,
            file_filename: file_info.source_filename,
            file_source_path,
            file_dest_path: file_info.dest_path,
            file_size: file_info.file_size,
            file_compressed_size: file_info.file_compressed_size,
            created_on,
        };
        File::insert(conn, file_record).await?;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Duplicate {
    ReuseObject,
    ReuseData,
    ForceUuid(Uuid),
}
