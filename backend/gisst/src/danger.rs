use sqlx::{PgConnection, postgres::PgQueryResult};
use uuid::Uuid;

use crate::models::{Environment, Object};
/// # Destructive Traits
/// This trait allows for potentially destructive modification of database entries.
/// This is mainly used by implementations that want to modify environments to update their core, and
/// to unlink and instance and an object.
#[allow(async_fn_in_trait)]
pub trait DestructiveEnvironment {
    async fn update_core(
        conn: &mut PgConnection,
        env_id: Uuid,
        core_name: &str,
        core_version: &str,
    ) -> sqlx::Result<PgQueryResult>;
}

impl DestructiveEnvironment for Environment {
    async fn update_core(
        conn: &mut PgConnection,
        env_id: Uuid,
        core_name: &str,
        core_version: &str,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"UPDATE environment SET environment_core_name=$1, environment_core_version=$2 WHERE environment_id=$3"#,
core_name, core_version, env_id
        ).execute(conn).await
    }
}

#[allow(async_fn_in_trait)]
pub trait DestructiveObject {
    async fn unlink(
        conn: &mut PgConnection,
        object_id: Uuid,
        instance_id: Uuid,
    ) -> sqlx::Result<PgQueryResult>;
}

impl DestructiveObject for Object {
    async fn unlink(
        conn: &mut PgConnection,
        object_id: Uuid,
        instance_id: Uuid,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"DELETE FROM instanceObject WHERE object_id=$1 AND instance_id=$2"#,
            object_id,
            instance_id
        )
        .execute(conn)
        .await
    }
}
