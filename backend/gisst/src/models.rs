use serde::{de, Deserialize, Deserializer, Serialize};

use async_trait::async_trait;
use std::{fmt, str::FromStr};

use sqlx::postgres::{PgConnection, PgQueryResult};
use time::OffsetDateTime;

use crate::model_enums::Framework;
use uuid::Uuid;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum NewRecordError {
    #[error("could not insert creator record into database")]
    Creator,
    #[error("could not insert environment record into database")]
    Environment,
    #[error("could not insert file record into database")]
    File,
    #[error("could not insert instance record into database")]
    Instance,
    #[error("could not insert image record into database")]
    Image,
    #[error("could not insert object record into database")]
    Object,
    #[error("could not insert replay record into database")]
    Replay,
    #[error("could not insert save record into database")]
    Save,
    #[error("could not insert state record into database")]
    State,
    #[error("could not insert work record into database")]
    Work,
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum UpdateRecordError {
    #[error("could not update creator record in database")]
    Creator,
    #[error("could not update environment record in database")]
    Environment,
    #[error("could not update file record in database")]
    File,
    #[error("could not update instance record into database")]
    Instance,
    #[error("could not update image record in database")]
    Image,
    #[error("could not update object record in database")]
    Object,
    #[error("could not update replay record in database")]
    Replay,
    #[error("could not update save record in database")]
    Save,
    #[error("could not update state record in database")]
    State,
    #[error("could not update work record in database")]
    Work,
}

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
// Model definitions that should match PSQL database schema
#[async_trait]
pub trait DBModel: Sized {
    fn id(&self) -> &Uuid;
    fn fields() -> Vec<(String, String)>;
    fn values_to_strings(&self) -> Vec<Option<String>>;
    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>>;
    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>>;
    async fn insert(conn: &mut PgConnection, model: Self) -> Result<Self, NewRecordError>;
    async fn update(conn: &mut PgConnection, model: Self) -> Result<Self, UpdateRecordError>;
    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult>;
}

#[async_trait]
pub trait DBHashable: Sized {
    async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>>;
    async fn flatten_file(conn: &mut PgConnection, model:Self) -> Result<FileRecordFlatten<Self>, sqlx::Error>;
    fn file_id(&self) -> &Uuid;
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
    pub created_on: Option<OffsetDateTime>,
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
    pub created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub file_id: Uuid,
    pub file_hash: String,
    pub file_filename: String,
    pub file_source_path: String,
    pub file_dest_path: String,
    pub file_size: i64, //PostgeSQL does not have native uint support
    pub created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub image_id: Uuid,
    pub file_id: Uuid,
    pub image_description: Option<String>,
    pub image_config: Option<sqlx::types::JsonValue>,
    pub created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instance {
    pub instance_id: Uuid,
    pub work_id: Uuid,
    pub environment_id: Uuid,
    pub instance_config: Option<sqlx::types::JsonValue>,
    pub created_on: Option<OffsetDateTime>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type)]
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
    pub instance_object_config: Option<sqlx::types::JsonValue>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Object {
    pub object_id: Uuid,
    pub file_id: Uuid,
    pub object_description: Option<String>,
    pub created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    pub replay_id: Uuid,
    pub instance_id: Uuid,
    pub creator_id: Uuid,
    pub replay_forked_from: Option<Uuid>,
    pub file_id: Uuid,
    pub created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Save {
    pub save_id: Uuid,
    pub instance_id: Uuid,
    pub save_short_desc: String,
    pub save_description: String,
    pub file_id: Uuid,
    pub creator_id: Uuid,
    pub created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub state_id: Uuid,
    pub instance_id: Uuid,
    pub is_checkpoint: bool,
    pub file_id: Uuid,
    pub state_name: String,
    pub state_description: String,
    pub screenshot_id: Option<Uuid>,
    pub replay_id: Option<Uuid>,
    pub creator_id: Option<Uuid>,
    pub state_replay_index: Option<i32>,
    pub state_derived_from: Option<Uuid>,
    pub created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Work {
    pub work_id: Uuid,
    pub work_name: String,
    pub work_version: String,
    pub work_platform: String,
    pub created_on: Option<OffsetDateTime>,
}
#[async_trait]
impl DBModel for Creator {
    fn id(&self) -> &Uuid {
        &self.creator_id
    }
    fn fields() -> Vec<(String, String)> {
        vec![
            ("creator_id".to_string(), "Uuid".to_string()),
            ("creator_username".to_string(), "String".to_string()),
            ("creator_full_name".to_string(), "String".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.creator_id.to_string()),
            Some(self.creator_full_name.to_string()),
            Some(self.creator_username.to_string()),
            unwrap_to_option_string(&self.created_on),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
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

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT creator_id,
            creator_username,
            creator_full_name,
            created_on
            FROM creator
            ORDER BY created_on DESC
            LIMIT $1"#,
            limit
        )
        .fetch_all(conn)
        .await
    }

    async fn insert(conn: &mut PgConnection, model: Creator) -> Result<Self, NewRecordError> {
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
        .map_err(|_| NewRecordError::Creator)
    }

    async fn update(conn: &mut PgConnection, creator: Creator) -> Result<Self, UpdateRecordError> {
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
        .map_err(|_| UpdateRecordError::Creator)
    }

    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM creator WHERE creator_id = $1", id)
            .execute(conn)
            .await
    }
}

#[async_trait]
impl DBHashable for File {
    async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
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

    async fn flatten_file(_conn: &mut PgConnection, record:Self) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        Ok(FileRecordFlatten {
            record: record.clone(),
            file_record: record.clone(),
        })
    }

    fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[async_trait]
impl DBModel for File {
    fn id(&self) -> &Uuid {
        &self.file_id
    }
    fn fields() -> Vec<(String, String)> {
        vec![
            ("file_id".to_string(), "Uuid".to_string()),
            ("file_hash".to_string(), "String".to_string()),
            ("file_filename".to_string(), "String".to_string()),
            ("file_dest_path".to_string(), "String".to_string()),
            ("file_source_path".to_string(), "String".to_string()),
            ("file_size".to_string(), "u64".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.file_id.to_string()),
            Some(self.file_hash.to_string()),
            Some(self.file_filename.to_string()),
            Some(self.file_dest_path.to_string()),
            Some(self.file_size.to_string()),
            Some(self.file_source_path.to_string()),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
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

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
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
            ORDER BY created_on DESC
            LIMIT $1"#,
            limit
        )
        .fetch_all(conn)
        .await
    }

    async fn insert(conn: &mut PgConnection, model: File) -> Result<Self, NewRecordError> {
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
            .map_err(|_| NewRecordError::File)
    }

    async fn update(conn: &mut PgConnection, file: File) -> Result<Self, UpdateRecordError> {
        sqlx::query_as!(
            File,
            r#"UPDATE file SET
            (file_hash, file_filename, file_source_path, file_dest_path, file_size, created_on) =
            ($1, $2, $3, $4, $5, $6)
            WHERE file_id = $7
            RETURNING file_id, file_hash, file_filename, file_source_path, file_dest_path, file_size, created_on"#,
            file.file_hash,
            file.file_filename,
            file.file_source_path,
            file.file_dest_path,
            file.file_size,
            file.created_on,
            file.file_id,
        )
            .fetch_one(conn)
            .await
            .map_err(|_| UpdateRecordError::File)
    }

    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM file WHERE file_id = $1", id)
            .execute(conn)
            .await
    }
}

#[async_trait]
impl DBModel for Instance {
    fn id(&self) -> &Uuid {
        &self.instance_id
    }

    fn fields() -> Vec<(String, String)> {
        vec![
            ("instance_id".to_string(), "Uuid".to_string()),
            ("environment_id".to_string(), "Uuid".to_string()),
            ("work_id".to_string(), "Uuid".to_string()),
            ("instance_config".to_string(), "Json".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.instance_id.to_string()),
            Some(self.environment_id.to_string()),
            Some(self.work_id.to_string()),
            unwrap_to_option_string(&self.instance_config),
            unwrap_to_option_string(&self.created_on),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT instance_id, environment_id, work_id, instance_config, created_on
            FROM instance WHERE instance_id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT instance_id, environment_id, work_id, instance_config, created_on
            FROM instance
            ORDER BY created_on DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(conn)
        .await
    }
    async fn insert(conn: &mut PgConnection, model: Instance) -> Result<Self, NewRecordError> {
        sqlx::query_as!(
            Instance,
            r#"INSERT INTO instance(
            instance_id,
            environment_id,
            work_id,
            instance_config,
            created_on
            )
            VALUES($1, $2, $3, $4, $5)
            RETURNING instance_id, environment_id, work_id, instance_config, created_on"#,
            model.instance_id,
            model.environment_id,
            model.work_id,
            model.instance_config,
            model.created_on
        )
        .fetch_one(conn)
        .await
        .map_err(|_| NewRecordError::Instance)
    }

    async fn update(
        conn: &mut PgConnection,
        instance: Instance,
    ) -> Result<Self, UpdateRecordError> {
        sqlx::query_as!(
            Instance,
            r#"UPDATE instance SET
            (environment_id, work_id, instance_config, created_on) =
            ($1, $2, $3, $4)
            WHERE instance_id = $5
            RETURNING instance_id, environment_id, work_id, instance_config, created_on"#,
            instance.environment_id,
            instance.work_id,
            instance.instance_config,
            instance.created_on,
            instance.instance_id,
        )
        .fetch_one(conn)
        .await
        .map_err(|_| UpdateRecordError::Instance)
    }

    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM instance WHERE instance_id = $1", id)
            .execute(conn)
            .await
    }
}

impl Instance {
    pub async fn get_all_for_work_id(
        conn: &mut PgConnection,
        work_id: Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT instance_id, environment_id, work_id, instance_config, created_on
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
            r#"SELECT * FROM state WHERE instance_id = $1"#,
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
            r#"SELECT * FROM replay WHERE instance_id = $1"#,
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
            r#"SELECT * FROM save WHERE instance_id = $1"#,
            instance_id
        )
            .fetch_all(conn)
            .await
    }
}

#[async_trait]
impl DBModel for Image {
    fn id(&self) -> &Uuid {
        &self.image_id
    }

    fn fields() -> Vec<(String, String)> {
        vec![
            ("image_id".to_string(), "Uuid".to_string()),
            ("file_id".to_string(), "String".to_string()),
            ("image_config".to_string(), "Json".to_string()),
            ("image_description".to_string(), "String".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.image_id.to_string()),
            Some(self.file_id.to_string()),
            unwrap_to_option_string(&self.image_config),
            unwrap_to_option_string(&self.image_description),
            unwrap_to_option_string(&self.created_on),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT image_id,
            file_id,
            image_config,
            image_description,
            created_on
            FROM image WHERE image_id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT image_id,
            file_id,
            image_config,
            image_description,
            created_on
            FROM image
            ORDER BY created_on DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(conn)
        .await
    }

    async fn insert(conn: &mut PgConnection, model: Image) -> Result<Self, NewRecordError> {
        sqlx::query_as!(
            Image,
            r#"INSERT INTO image(
            image_id,
            file_id,
            image_config,
            image_description,
            created_on
            )
            VALUES($1, $2, $3, $4, $5)
            RETURNING image_id, file_id, image_config, image_description, created_on"#,
            model.image_id,
            model.file_id,
            model.image_config,
            model.image_description,
            model.created_on
        )
        .fetch_one(conn)
        .await
        .map_err(|_| NewRecordError::Image)
    }

    async fn update(conn: &mut PgConnection, image: Image) -> Result<Self, UpdateRecordError> {
        sqlx::query_as!(
            Image,
            r#"UPDATE image SET
            (file_id, image_config, image_description, created_on) =
            ($1, $2, $3, $4)
            WHERE image_id = $5
            RETURNING image_id, file_id, image_config, image_description, created_on"#,
            image.file_id,
            image.image_config,
            image.image_description,
            image.created_on,
            image.image_id,
        )
        .fetch_one(conn)
        .await
        .map_err(|_| UpdateRecordError::Image)
    }

    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM image WHERE image_id = $1", id)
            .execute(conn)
            .await
    }
}

#[async_trait]
impl DBHashable for Image {
    async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT image.image_id,
                image.file_id,
                image.image_config,
                image.image_description,
                image.created_on
                FROM image
                JOIN file USING(file_id)
                WHERE file.file_hash = $1
                "#,
            hash
        )
        .fetch_optional(conn)
        .await
    }

    async fn flatten_file(conn: &mut PgConnection, model: Self) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        let file_record = File::get_by_id(conn, model.file_id.clone()).await?.unwrap();
        Ok(FileRecordFlatten {
            record: model,
            file_record,
        })
    }

    fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

impl Image {
    pub async fn link_image_to_environment(
        conn: &mut PgConnection,
        image_id: Uuid,
        environment_id: Uuid,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "INSERT INTO environmentImage (environment_id, image_id) VALUES ($1, $2)",
            environment_id,
            image_id
        )
        .execute(conn)
        .await
    }

    pub async fn get_all_for_environment_id(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT image_id, file_id, image_config, image_description, image.created_on
            FROM image
            JOIN environmentImage USING(image_id)
            JOIN environment USING(environment_id)
            WHERE environment_id = $1
            "#,
            id
        )
        .fetch_all(conn)
        .await
    }
}

#[async_trait]
impl DBModel for Environment {
    fn id(&self) -> &Uuid {
        &self.environment_id
    }

    fn fields() -> Vec<(String, String)> {
        vec![
            ("environment_id".to_string(), "Uuid".to_string()),
            ("environment_name".to_string(), "String".to_string()),
            ("environment_framework".to_string(), "String".to_string()),
            ("environment_core_name".to_string(), "String".to_string()),
            ("environment_core_version".to_string(), "String".to_string()),
            ("environment_derive_from".to_string(), "Uuid".to_string()),
            ("environment_config".to_string(), "Json".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.environment_id.to_string()),
            Some(self.environment_name.to_string()),
            Some(self.environment_framework.to_string()),
            Some(self.environment_core_name.to_string()),
            Some(self.environment_core_version.to_string()),
            unwrap_to_option_string(&self.environment_derived_from),
            unwrap_to_option_string(&self.environment_config),
            unwrap_to_option_string(&self.created_on),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
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

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT environment_id, environment_name, environment_framework as "environment_framework:_", environment_core_name, environment_core_version, environment_derived_from, environment_config, created_on
            FROM environment
            ORDER BY created_on DESC LIMIT $1"#,
            limit
        )
            .fetch_all(conn)
            .await
    }

    async fn insert(conn: &mut PgConnection, model: Environment) -> Result<Self, NewRecordError> {
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
            .map_err(|_| NewRecordError::Environment)
    }

    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM environment WHERE environment_id = $1", id)
            .execute(conn)
            .await
    }

    async fn update(conn: &mut PgConnection, env: Environment) -> Result<Self, UpdateRecordError> {
        sqlx::query_as!(
            Environment,
            r#"UPDATE environment SET
            (environment_name, environment_framework, environment_core_name, environment_core_version, environment_derived_from, environment_config, created_on) =
            ($1, $2, $3, $4, $5, $6, $7)
            WHERE environment_id = $8
            RETURNING environment_id, environment_name, environment_framework as "environment_framework:_", environment_core_name, environment_core_version, environment_derived_from, environment_config, created_on"#,
            env.environment_name,
            env.environment_framework as _,
            env.environment_core_name,
            env.environment_core_version,
            env.environment_derived_from,
            env.environment_config,
            env.created_on,
            env.environment_id,
        )
            .fetch_one(conn)
            .await
            .map_err(|_| UpdateRecordError::Environment)
    }
}

impl Environment {
    pub async fn delete_environment_image_links_by_id(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM environmentImage WHERE environment_id = $1", id)
            .execute(conn)
            .await
    }
}

#[async_trait]
impl DBHashable for Object {
    async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
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

    async fn flatten_file(conn: &mut PgConnection, model: Self) -> Result<FileRecordFlatten<Self>, sqlx::Error>{
        let file_record = File::get_by_id(conn, model.file_id.clone()).await?.unwrap();
        Ok(FileRecordFlatten {
            record: model,
            file_record,
        })
    }

    fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[async_trait]
impl DBModel for Object {
    fn id(&self) -> &Uuid {
        &self.object_id
    }
    fn fields() -> Vec<(String, String)> {
        vec![
            ("object_id".to_string(), "Uuid".to_string()),
            ("file_id".to_string(), "String".to_string()),
            ("object_description".to_string(), "String".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.object_id.to_string()),
            Some(self.file_id.to_string()),
            unwrap_to_option_string(&self.object_description),
            unwrap_to_option_string(&self.created_on),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
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

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT object_id, file_id, object_description, created_on
            FROM object
            ORDER BY created_on DESC LIMIT $1
            "#,
            limit
        )
        .fetch_all(conn)
        .await
    }
    async fn insert(conn: &mut PgConnection, object: Object) -> Result<Self, NewRecordError> {
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
        .map_err(|_| NewRecordError::Object)
    }

    async fn update(conn: &mut PgConnection, object: Object) -> Result<Self, UpdateRecordError> {
        sqlx::query_as!(
            Object,
            r#"UPDATE object SET
            (file_id, object_description, created_on) =
            ($1, $2, $3)
            WHERE object_id = $4
            RETURNING object_id, file_id, object_description, created_on"#,
            object.file_id,
            object.object_description,
            object.created_on,
            object.object_id,
        )
        .fetch_one(conn)
        .await
        .map_err(|_| UpdateRecordError::Object)
    }
    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM object WHERE object_id = $1", id)
            .execute(conn)
            .await
    }
}

impl Object {
    pub async fn link_object_to_instance(
        conn: &mut PgConnection,
        object_id: Uuid,
        instance_id: Uuid,
        role: ObjectRole,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"INSERT INTO instanceObject(instance_id, object_id, object_role)  VALUES ($1, $2, $3)"#,
            instance_id,
            object_id,
            role as _
        )
        .execute(conn)
        .await
    }

    pub async fn get_object_instance_by_ids(
        conn: &mut PgConnection,
        object_id: Uuid,
        instance_id: Uuid,
    ) -> sqlx::Result<Option<InstanceObject>> {
        sqlx::query_as!(InstanceObject, r#"SELECT object_id, instance_id, instance_object_config, object_role as "object_role:_" FROM instanceObject WHERE object_id = $1 AND instance_id = $2"#, object_id, instance_id).fetch_optional(conn).await
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

#[async_trait]
impl DBModel for Replay {
    fn id(&self) -> &Uuid {
        &self.replay_id
    }
    fn fields() -> Vec<(String, String)> {
        vec![
            ("replay_id".to_string(), "Uuid".to_string()),
            ("creator_id".to_string(), "Uuid".to_string()),
            ("file_id".to_string(), "Uuid".to_string()),
            ("replay_forked_from".to_string(), "Uuid".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.replay_id.to_string()),
            Some(self.creator_id.to_string()),
            Some(self.file_id.to_string()),
            unwrap_to_option_string(&self.replay_forked_from),
            unwrap_to_option_string(&self.created_on),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT replay_id,
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

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT replay_id,
            instance_id,
            creator_id,
            file_id,
            replay_forked_from,
            created_on
            FROM replay ORDER BY created_on DESC LIMIT $1
            "#,
            limit
        )
        .fetch_all(conn)
        .await
    }

    async fn insert(conn: &mut PgConnection, model: Self) -> Result<Self, NewRecordError> {
        sqlx::query_as!(
            Replay,
            r#"INSERT INTO replay (
            replay_id,
            instance_id,
            creator_id,
            file_id,
            replay_forked_from,
            created_on
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
            replay_id,
            instance_id,
            creator_id,
            file_id,
            replay_forked_from,
            created_on
            "#,
            model.replay_id,
            model.instance_id,
            model.creator_id,
            model.file_id,
            model.replay_forked_from,
            model.created_on,
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            dbg!(e);
            NewRecordError::Replay
        })
    }

    async fn update(conn: &mut PgConnection, replay: Replay) -> Result<Self, UpdateRecordError> {
        sqlx::query_as!(
            Replay,
            r#"UPDATE replay SET
            (instance_id, creator_id, replay_forked_from, file_id, created_on) =
            ($1, $2, $3, $4, $5)
            WHERE replay_id = $6
            RETURNING replay_id, instance_id, creator_id, replay_forked_from, file_id, created_on"#,
            replay.instance_id,
            replay.creator_id,
            replay.replay_forked_from,
            replay.file_id,
            replay.created_on,
            replay.replay_id,
        )
        .fetch_one(conn)
        .await
        .map_err(|_| UpdateRecordError::Replay)
    }

    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM replay WHERE replay_id = $1", id)
            .execute(conn)
            .await
    }
}

#[async_trait]
impl DBHashable for Replay {
    async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT
           replay.replay_id,
           replay.instance_id,
           replay.creator_id,
           replay.replay_forked_from,
           replay.file_id,
           replay.created_on
           FROM replay
           JOIN file USING (file_id)
           WHERE file.file_hash = $1"#,
            hash
        )
        .fetch_optional(conn)
        .await
    }

    async fn flatten_file(conn: &mut PgConnection, model: Self) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        let file_record = File::get_by_id(conn, model.file_id.clone()).await?.unwrap();
        Ok(FileRecordFlatten {
            record: model,
            file_record,
        })
    }

    fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[async_trait]
impl DBModel for Save {
    fn id(&self) -> &Uuid {
        &self.save_id
    }

    fn fields() -> Vec<(String, String)> {
        vec![
            ("save_id".to_string(), "Uuid".to_string()),
            ("instance_id".to_string(), "Uuid".to_string()),
            ("save_short_desc".to_string(), "Uuid".to_string()),
            ("save_description".to_string(), "String".to_string()),
            ("file_id".to_string(), "String".to_string()),
            ("creator_id".to_string(), "Uuid".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.save_id.to_string()),
            Some(self.instance_id.to_string()),
            Some(self.save_short_desc.to_string()),
            Some(self.save_description.to_string()),
            Some(self.file_id.to_string()),
            Some(self.creator_id.to_string()),
            unwrap_to_option_string(&self.created_on),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
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

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
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
            FROM save ORDER BY created_on DESC LIMIT $1"#,
            limit
        )
        .fetch_all(conn)
        .await
    }

    async fn insert(conn: &mut PgConnection, model: Self) -> Result<Self, NewRecordError> {
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
        .map_err(|_| NewRecordError::Save)
    }

    async fn update(conn: &mut PgConnection, save: Save) -> Result<Self, UpdateRecordError> {
        sqlx::query_as!(
            Save,
            r#"UPDATE save SET
            (instance_id, save_short_desc, save_description, file_id, creator_id, created_on) =
            ($1, $2, $3, $4, $5, $6)
            WHERE save_id = $7
            RETURNING save_id, instance_id, save_short_desc, save_description, file_id, creator_id, created_on"#,
            save.instance_id,
            save.save_short_desc,
            save.save_description,
            save.file_id,
            save.creator_id,
            save.created_on,
            save.save_id,
        )
            .fetch_one(conn)
            .await
            .map_err(|_| UpdateRecordError::Save)
    }

    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM save WHERE save_id = $1", id)
            .execute(conn)
            .await
    }
}

#[async_trait]
impl DBHashable for Save {
    async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
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

    async fn flatten_file(conn: &mut PgConnection, model: Self) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        let file_record = File::get_by_id(conn, model.file_id.clone()).await?.unwrap();
        Ok(FileRecordFlatten {
            record: model,
            file_record,
        })
    }

    fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[async_trait]
impl DBModel for State {
    fn id(&self) -> &Uuid {
        &self.state_id
    }

    fn fields() -> Vec<(String, String)> {
        vec![
            ("state_id".to_string(), "Uuid".to_string()),
            ("instance_id".to_string(), "Uuid".to_string()),
            ("is_checkpoint".to_string(), "bool".to_string()),
            ("file_id".to_string(), "String".to_string()),
            ("state_name".to_string(), "String".to_string()),
            ("state_description".to_string(), "String".to_string()),
            ("screenshot_id".to_string(), "Uuid".to_string()),
            ("replay_id".to_string(), "Uuid".to_string()),
            ("creator_id".to_string(), "Uuid".to_string()),
            ("state_replay_index".to_string(), "i32".to_string()),
            ("state_derived_from".to_string(), "Uuid".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.state_id.to_string()),
            Some(self.instance_id.to_string()),
            Some(self.is_checkpoint.to_string()),
            Some(self.file_id.to_string()),
            Some(self.state_name.to_string()),
            Some(self.state_description.to_string()),
            unwrap_to_option_string(&self.screenshot_id),
            unwrap_to_option_string(&self.creator_id),
            unwrap_to_option_string(&self.replay_id),
            unwrap_to_option_string(&self.state_replay_index),
            unwrap_to_option_string(&self.state_derived_from),
            unwrap_to_option_string(&self.created_on),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
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

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
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
            FROM state ORDER BY created_on DESC LIMIT $1
            "#,
            limit
        )
        .fetch_all(conn)
        .await
    }

    async fn insert(conn: &mut PgConnection, state: Self) -> Result<Self, NewRecordError> {
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
        .map_err(|_| NewRecordError::State)
    }

    async fn update(conn: &mut PgConnection, state: State) -> Result<Self, UpdateRecordError> {
        sqlx::query_as!(
            State,
            r#"UPDATE state SET
            (instance_id,
            is_checkpoint,
            file_id,
            state_name,
            state_description,
            screenshot_id,
            replay_id,
            creator_id,
            state_replay_index,
            state_derived_from,
            created_on) =
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            WHERE state_id = $12
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
            created_on"#,
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
            state.state_id,
        )
        .fetch_one(conn)
        .await
        .map_err(|_| UpdateRecordError::State)
    }

    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM state WHERE state_id = $1", id)
            .execute(conn)
            .await
    }
}

#[async_trait]
impl DBHashable for State {
    async fn get_by_hash(conn: &mut PgConnection, hash: &str) -> sqlx::Result<Option<Self>> {
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

    async fn flatten_file(conn: &mut PgConnection, model: Self) -> Result<FileRecordFlatten<Self>, sqlx::Error> {
        let file_record = File::get_by_id(conn, model.file_id.clone()).await?.unwrap();
        Ok(FileRecordFlatten {
            record: model,
            file_record,
        })
    }

    fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[async_trait]
impl DBModel for Work {
    fn id(&self) -> &Uuid {
        &self.work_id
    }

    fn fields() -> Vec<(String, String)> {
        vec![
            ("work_id".to_string(), "Uuid".to_string()),
            ("work_name".to_string(), "String".to_string()),
            ("work_version".to_string(), "String".to_string()),
            ("work_platform".to_string(), "String".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string()),
        ]
    }

    fn values_to_strings(&self) -> Vec<Option<String>> {
        vec![
            Some(self.work_id.to_string()),
            Some(self.work_name.to_string()),
            Some(self.work_version.to_string()),
            Some(self.work_platform.to_string()),
            unwrap_to_option_string(&self.created_on),
        ]
    }

    async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform, created_on FROM work WHERE work_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }

    async fn get_all(conn: &mut PgConnection, limit: Option<i64>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform, created_on FROM work ORDER BY created_on DESC LIMIT $1"#,
            limit
        )
            .fetch_all(conn)
            .await
    }

    async fn insert(conn: &mut PgConnection, work: Self) -> Result<Self, NewRecordError> {
        // Note: the "!" following the AS statements after RETURNING are forcing not-null status on those fields
        // from: https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns
        sqlx::query_as!(
            Work,
            r#"INSERT INTO work (
            work_id, work_name, work_version, work_platform, created_on )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING work_id, work_name, work_version, work_platform, created_on
            "#,
            work.work_id,
            work.work_name,
            work.work_version,
            work.work_platform,
            work.created_on,
        )
        .fetch_one(conn)
        .await
        .map_err(|_| NewRecordError::Work)
    }

    async fn update(conn: &mut PgConnection, work: Work) -> Result<Self, UpdateRecordError> {
        sqlx::query_as!(
            Work,
            r#"UPDATE work SET
            (work_name, work_version, work_platform, created_on) =
            ($1, $2, $3, $4)
            WHERE work_id = $5
            RETURNING work_id, work_name, work_version, work_platform, created_on"#,
            work.work_name,
            work.work_version,
            work.work_platform,
            work.created_on,
            work.work_id,
        )
        .fetch_one(conn)
        .await
        .map_err(|_| UpdateRecordError::Work)
    }

    async fn delete_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM work WHERE work_id = $1", id)
            .execute(conn)
            .await
    }
}

impl Work {
    pub async fn get_by_name(conn: &mut PgConnection, name: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform, created_on FROM work WHERE work_name = $1"#,
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
            r#"SELECT work_id, work_name, work_version, work_platform, created_on FROM work WHERE work_platform = $1"#,
            platform
        )
            .fetch_all(conn)
            .await
    }
}

fn unwrap_to_option_string<T: ToString>(o: &Option<T>) -> Option<String> {
    o.as_ref().map(T::to_string)
}
