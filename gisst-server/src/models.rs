use serde::{Serialize, Deserialize};
use sqlx::{
    PgConnection,
};
use time::OffsetDateTime;


use uuid::{
    Uuid,
    uuid
};


#[derive(Debug, thiserror::Error, Serialize)]
pub enum NewRecordError {
    #[error("could not insert creator record into database")]
    CreatorError,
    #[error("could not insert environment record into database")]
    EnvironmentError,
    #[error("could not insert instance record into database")]
    InstanceError,
    #[error("could not insert image record into database")]
    ImageError,
    #[error("could not insert object record into database")]
    ObjectError,
    #[error("could not insert replay record into database")]
    ReplayError,
    #[error("could not insert save record into database")]
    SaveError,
    #[error("could not insert state record into database")]
    StateError,
    #[error("could not insert work record into database")]
    WorkError,
}

// General functions for Models

fn default_uuid() -> Uuid {
    uuid!("00000000-0000-0000-0000-000000000000")
}

const fn unix_epoch() -> OffsetDateTime {
    OffsetDateTime::UNIX_EPOCH
}
// Model definitions that should match PSQL database schema
pub enum DBModel {
    Creator(Creator),
    Environment(Environment),
    Image(Image),
    Instance(Instance),
    Object(Object),
    Replay(Replay),
    Save(Save),
    State(State),
    Work(Work),
    Void
}

impl DBModel {
    pub fn get_model_by_name(name: &str) -> DBModel {
        match name {
            "Creator" => DBModel::Creator(Creator::default()),
            "Environment" => DBModel::Environment(Environment::default()),
            "Image" => DBModel::Image(Image::default()),
            "Instance" => DBModel::Instance(Instance::default()),
            "Object" => DBModel::Object(Object::default()),
            "Replay" => DBModel::Replay(Replay::default()),
            "Save" => DBModel::Save(Save::default()),
            "State" => DBModel::State(State::default()),
            "Work" => DBModel::Work(Work::default()),
            _ => DBModel::Void
        }
    }

    pub fn get_model_fields(model: DBModel) -> Vec<(String,String)> {
        match model {
            DBModel::Creator(_) => Creator::fields(),
            DBModel::Environment(_) => Environment::fields(),
            DBModel::Image(_) => Image::fields(),
            DBModel::Instance(_) => Instance::fields(),
            DBModel::Object(_) => Object::fields(),
            DBModel::Replay(_) => Replay::fields(),
            DBModel::Save(_) => Save::fields(),
            DBModel::State(_) => State::fields(),
            DBModel::Work(_) => Work::fields(),
            DBModel::Void => vec![("".to_string(), "".to_string())]
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Creator{
    creator_id: Uuid,
    creator_username: String,
    creator_full_name: String,
    created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Environment {
    environment_id: Uuid,
    environment_name: String,
    core_name: String,
    core_version: String,
    environment_derived_from: Option<Uuid>,
    environment_config: Option<sqlx::types::JsonValue>,
    created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Image{
    image_id: Uuid,
    image_filename: String,
    image_path: String,
    image_hash: String,
    image_config: Option<sqlx::types::JsonValue>,
    created_on: Option<OffsetDateTime>,
}


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Instance {
    instance_id: Uuid,
    work_id: Uuid,
    pub environment_id: Uuid,
    instance_framework: String,
    instance_config: Option<sqlx::types::JsonValue>,
    created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Object{
    pub object_id: Uuid,
    pub object_hash: String,
    pub object_filename: String,
    pub object_path: String,
    pub created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Replay{
    replay_id: Uuid,
    instance_id: Uuid,
    creator_id: Uuid,
    replay_forked_from: Option<Uuid>,
    replay_filename: String,
    replay_path: String,
    replay_hash: String,
    created_on: Option<OffsetDateTime>
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Save{
    save_id: Uuid,
    instance_id: Uuid,
    save_short_desc: String,
    save_description: String,
    save_filename: String,
    save_path: String,
    creator_id: Uuid,
    created_on: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State{
    state_id: Uuid,
    instance_id: Uuid,
    is_checkpoint: bool,
    state_path: String,
    state_hash: String,
    state_filename: String,
    state_name: String,
    state_description: String,
    screenshot_id: Option<Uuid>,
    replay_id: Option<Uuid>,
    creator_id: Option<Uuid>,
    state_replay_index: Option<i32>,
    state_derived_from: Option<Uuid>,
    created_on: Option<OffsetDateTime>
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Work {
    work_id: Uuid,
    work_name: String,
    work_version: String,
    work_platform: String,
    created_on: Option<OffsetDateTime>
}

impl Creator {
    pub fn default() -> Self{
        Self { creator_id: default_uuid(), ..Default::default()}
    }

    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("creator_id".to_string(), "Uuid".to_string()),
            ("creator_username".to_string(), "String".to_string()),
            ("creator_full_name".to_string(), "String".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string())
        ]
    }
}

impl Instance {
    pub fn default() -> Self{
        Self { instance_id: default_uuid(), ..Default::default()}
    }

    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("instance_id".to_string(), "Uuid".to_string()),
            ("environment_id".to_string(), "Uuid".to_string()),
            ("work_id".to_string(), "Uuid".to_string()),
            ("instance_framework".to_string(), "String".to_string()),
            ("instance_config".to_string(), "Json".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string())
        ]
    }

    pub async fn get_by_id(conn: &mut PgConnection, id:Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT instance_id, environment_id, work_id, instance_framework, instance_config, created_on
            FROM instance WHERE instance_id = $1
            "#,
            id
        )
            .fetch_optional(conn)
            .await
    }

}

impl Image {
    pub fn default() -> Self {
        Self { image_id: default_uuid(), ..Default::default()}
    }

    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("image_id".to_string(), "Uuid".to_string()),
            ("image_filename".to_string(), "String".to_string()),
            ("image_path".to_string(), "String".to_string()),
            ("image_hash".to_string(), "String".to_string()),
            ("image_config".to_string(), "Json".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string())
        ]
    }
    pub async fn get_by_id(conn: &mut PgConnection, id:Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT image_id,
            image_filename,
            image_path,
            image_hash,
            image_config,
            created_on
            FROM image WHERE image_id = $1
            "#,
            id
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn get_all_for_environment_id(
        conn: &mut PgConnection,
        id: Uuid
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT image_id, image_filename, image_path, image_hash, image_config, image.created_on
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

impl Environment {

    pub fn default() -> Self {
        Self { environment_id: default_uuid(), ..Default::default()}
    }

    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("environment_id".to_string(), "Uuid".to_string()),
            ("environment_name".to_string(), "String".to_string()),
            ("core_name".to_string(), "String".to_string()),
            ("core_version".to_string(), "String".to_string()),
            ("environment_derive_from".to_string(), "Uuid".to_string()),
            ("environment_config".to_string(), "Json".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string())
        ]
    }

    pub async fn get_by_id(
        conn: &mut PgConnection,
        id: Uuid
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT environment_id, environment_name, core_name, core_version, environment_derived_from, environment_config, created_on
            FROM environment
            WHERE environment_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }
}

impl Object {

    pub fn default() -> Self {
        Self { object_id: default_uuid(), ..Default::default()}
    }

    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("object_id".to_string(), "Uuid".to_string()),
            ("object_hash".to_string(), "String".to_string()),
            ("object_filename".to_string(), "String".to_string()),
            ("object_path".to_string(), "String".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string())
        ]
    }

    pub async fn get_by_hash(
        conn: &mut PgConnection,
        hash: &str
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT object_id, object_hash, object_filename, object_path, created_on
            FROM object
            WHERE object_hash = $1"#,
            hash
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn get_by_id(
        conn: &mut PgConnection,
        id: Uuid
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT object_id, object_hash, object_filename, object_path, created_on
            FROM object
            WHERE object_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn get_all_for_instance_id(
        conn: &mut PgConnection,
        id: Uuid
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT object_id, object_hash, object_filename, object_path, object.created_on
            FROM object
            JOIN instanceObject USING(object_id)
            JOIN instance USING(instance_id)
            WHERE instance_id = $1
            "#,
            id
        )
            .fetch_all(conn)
            .await
    }

    pub async fn insert(conn: &mut PgConnection, object: Object) -> Result<Self, NewRecordError> {
        // Note: the "!" following the AS statements after RETURNING are forcing not-null status on those fields
        // from: https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns
        sqlx::query_as!(Object,
            r#"INSERT INTO object (
            object_id, object_hash, object_filename, object_path, created_on )
            VALUES ($1, $2, $3, $4, current_timestamp)
            RETURNING object_id, object_hash, object_filename, object_path, created_on
            "#,
            object.object_id,
            object.object_hash,
            object.object_filename,
            object.object_path,
        )
            .fetch_one(conn)
            .await
            .map_err(|_| NewRecordError::ObjectError)
    }


}

impl Replay {
    pub fn default() -> Self{
        Self { replay_id: default_uuid(), ..Default::default()}
    }

    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("replay_id".to_string(), "Uuid".to_string()),
            ("creator_id".to_string(), "Uuid".to_string()),
            ("replay_forked_from".to_string(), "Uuid".to_string()),
            ("replay_hash".to_string(), "String".to_string()),
            ("replay_filename".to_string(), "String".to_string()),
            ("replay_path".to_string(), "String".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string())
        ]
    }

    pub async fn get_by_id(conn: &mut PgConnection, id:Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT replay_id,
            instance_id,
            creator_id,
            replay_forked_from,
            replay_filename,
            replay_hash,
            replay_path,
            created_on
            FROM replay WHERE replay_id = $1
            "#,
            id,
        )
            .fetch_optional(conn)
            .await
    }
}

impl Save {
    pub fn default() -> Self {
        Self { save_id: default_uuid(), ..Default::default()}
    }

    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("save_id".to_string(), "Uuid".to_string()),
            ("instance_id".to_string(), "Uuid".to_string()),
            ("save_short_desc".to_string(), "Uuid".to_string()),
            ("save_description".to_string(), "String".to_string()),
            ("save_filename".to_string(), "String".to_string()),
            ("save_path".to_string(), "String".to_string()),
            ("creator_id".to_string(), "Uuid".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string())
        ]
    }

    pub async fn get_by_id(conn: &mut PgConnection, id:Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT
            save_id,
            instance_id,
            save_short_desc,
            save_description,
            save_filename,
            save_path,
            creator_id,
            created_on
            FROM save WHERE save_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }
}

impl State {
    pub fn default() -> Self{
        Self { state_id: default_uuid(), ..Default::default()}
    }

    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("state_id".to_string(), "Uuid".to_string()),
            ("instance_id".to_string(), "Uuid".to_string()),
            ("is_checkpoint".to_string(), "bool".to_string()),
            ("state_filename".to_string(), "String".to_string()),
            ("state_path".to_string(), "String".to_string()),
            ("state_name".to_string(), "String".to_string()),
            ("state_description".to_string(), "String".to_string()),
            ("screenshot_id".to_string(), "Uuid".to_string()),
            ("replay_id".to_string(), "Uuid".to_string()),
            ("creator_id".to_string(), "Uuid".to_string()),
            ("state_replay_index".to_string(), "i32".to_string()),
            ("state_derived_from".to_string(), "Uuid".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string())
        ]
    }

    pub async fn get_by_id(conn: &mut PgConnection, id:Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT state_id,
            instance_id,
            is_checkpoint,
            state_filename,
            state_path,
            state_hash,
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
}

impl Work {
    pub fn default() -> Self {
        Self{work_id: default_uuid(), ..Default::default()}
    }

    pub fn fields() -> Vec<(String, String)> {
        vec![
            ("work_id".to_string(), "Uuid".to_string()),
            ("work_name".to_string(), "String".to_string()),
            ("work_version".to_string(), "String".to_string()),
            ("work_platform".to_string(), "String".to_string()),
            ("created_on".to_string(), "OffsetDateTime".to_string())
        ]
    }
    pub async fn get_by_id(
        conn: &mut PgConnection,
        id: Uuid
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT work_id, work_name, work_version, work_platform, created_on FROM work WHERE work_id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }

    pub async fn get_by_name(
        conn: &mut PgConnection,
        name: &str
    ) -> sqlx::Result<Vec<Self>> {
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
        platform: &str
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

enum Framework {
    RetroArch,
    V86,
}

impl TryFrom<String> for Framework {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            "retroarch" => Ok(Framework::RetroArch),
            "v86" => Ok(Framework::V86),
            _ => Err("Attempting to convert Framework that does not exist.")
        }
    }
}

pub enum Platform {
    NES,
    SNES,
    DOS,
}

impl TryFrom<String> for Platform {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            "Microsoft Disk Operating System" => Ok(Platform::DOS),
            "Nintendo Entertainment System" => Ok(Platform::NES),
            "Super Nintendo Entertainment System" => Ok(Platform::SNES),
            _ => Err("Attempting to convert Platform that does not exist")
        }
    }
}