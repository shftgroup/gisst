use gisst::danger::{DestructiveEnvironment, DestructiveObject};
use gisst::models::{Environment, Object};
use sqlx::PgPool;
use uuid::Uuid;
use crate::common::{instance_id, object_id};

mod common;

// --------------------------------------------------------------------
// Environment::update_core
// --------------------------------------------------------------------

#[sqlx::test(migrations = "../migrations/", fixtures("core", "environment"))]
async fn update_core_change_values(pool:PgPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    let result = Environment::update_core(&mut conn, common::env_id(), "updated-core", "2.0").await?;
    assert_eq!(result.rows_affected(), 1, "expected 1 row to be affected");

    let row = sqlx::query!(
        "SELECT environment_core_name, environment_core_version \
         FROM environment WHERE environment_id = $1",
        common::env_id()
    )
        .fetch_one(&mut *conn)
        .await?;

    assert_eq!(row.environment_core_name, "updated-core");
    assert_eq!(row.environment_core_version, "2.0");

    Ok(())
}

// TODO: May want to change this to error on attempts to update non-existent environments
#[sqlx::test(migrations = "../migrations/", fixtures("core", "environment"))]
async fn update_core_missing_env(pool:PgPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    let missing = Uuid::new_v4();
    let result = Environment::update_core(&mut conn, missing, "missing-core", "2.0").await?;

    assert_eq!(result.rows_affected(), 0, "expected no rows to be affected");

    Ok(())
}

// --------------------------------------------------------------------
// Object::unlink
// --------------------------------------------------------------------

#[sqlx::test(migrations = "../migrations/", fixtures("core", "environment", "work", "file", "object", "instance", "instance_object"))]
async fn unlink_does_not_effect_object_record(pool:PgPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    Object::unlink(&mut conn, object_id(), instance_id()).await?;

    let obj = Object::get_by_id(&mut conn, object_id()).await?;
    assert!(obj.is_some(), "object row exists after unlink");
    Ok(())
}

#[sqlx::test(migrations = "../migrations/", fixtures("core", "environment", "work", "file", "object", "instance", "instance_object"))]
async fn unlink_removes_instance_object_record(pool:PgPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    let result = Object::unlink(&mut conn, object_id(), instance_id()).await?;
    assert_eq!(result.rows_affected(), 1, "expected 1 row to be affected");

    let count: Option<i64> = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM instanceobject WHERE object_id = $1 AND instance_id = $2",
        object_id(),
        instance_id()
    ).fetch_one(&mut *conn).await?;

    assert_eq!(count, Some(0), "instance object row should no longer exist");
    Ok(())
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures("core", "environment", "work", "file", "object", "instance")
)]
async fn unlink_nonexistent_link_is_not_an_error(pool: PgPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    // No instance_object fixture loaded, so no row exists.
    let result = Object::unlink(&mut conn, object_id(), instance_id()).await?;

    assert_eq!(result.rows_affected(), 0);

    Ok(())
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures("core", "environment", "work", "file", "object", "instance", "instance_object")
)]
async fn unlink_wrong_object_affects_zero_rows(pool: PgPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    let other_object = Uuid::new_v4();

    let result = Object::unlink(&mut conn, other_object, instance_id()).await?;

    assert_eq!(result.rows_affected(), 0);

    // Original link is untouched.
    let count: Option<i64> = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM instanceObject WHERE object_id = $1 AND instance_id = $2",
        object_id(),
        instance_id()
    )
        .fetch_one(&mut *conn)
        .await?;

    assert_eq!(count, Some(1), "original instanceObject row should be unaffected");

    Ok(())
}