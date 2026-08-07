#![allow(unused,reason="Initial implementation without API, just with tests")]

use std::{fmt, str::FromStr};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgConnection;
use sqlx::Connection;
use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
pub enum TimeoutTaskError {
    #[error("duration conversion error {0:?}")]
    InvalidDuration(std::time::Duration),
    #[error("database error")]
    Sqlx(#[from] sqlx::Error)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(rename_all = "lowercase", type_name = "task_state")]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    Idle,
    Active,
    Error,
    Cancel,
    Done
}

impl FromStr for TaskState {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "idle" => Ok(TaskState::Idle),
            "active" => Ok(TaskState::Active),
            "error" => Ok(TaskState::Error),
            "cancel" => Ok(TaskState::Cancel),
            "done" => Ok(TaskState::Done),
            _ => Err("Unrecognized TaskState enum value"),
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f,"idle"),
            Self::Active => write!(f,"active"),
            Self::Error => write!(f,"error"),
            Self::Cancel => write!(f,"cancel"),
            Self::Done => write!(f,"done")
        }
    }
}

#[expect(clippy::struct_field_names, reason="Has to match database columns")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub task_id: Uuid,
    pub task_created_on: DateTime<Utc>,
    pub task_retry_count: i32,
    pub task_type: String,
    pub task_claimant: Option<String>, // a task runner ID
    pub task_claimed_on: Option<DateTime<Utc>>,
    pub task_updated_on: DateTime<Utc>,
    pub task_state: TaskState,
    pub task_status: sqlx::types::JsonValue,
    pub task_last_status: Option<sqlx::types::JsonValue>,
    pub task_input: sqlx::types::JsonValue,
    pub task_output: sqlx::types::JsonValue,
}

const TASK_RETRY_LIMIT:i32 = 5;

impl Task {
    pub async fn get_by_id(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT task_id, task_created_on, task_retry_count, task_type,
                      task_claimant, task_claimed_on, task_updated_on,
                      task_state as "task_state:_",
                      task_status, task_last_status, task_input, task_output
               FROM task WHERE task_id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }
    pub async fn get_tasks(conn: &mut PgConnection, task_state:Option<TaskState>, task_type:Option<&str>) -> sqlx::Result<Vec<Self>> {
        match (task_state, task_type) {
            (Some(state), Some(ttype)) => {
                sqlx::query_as!(
                    Self,
                    r#"SELECT task_id, task_created_on, task_retry_count, task_type,
                              task_claimant, task_claimed_on, task_updated_on,
                              task_state as "task_state:_",
                              task_status, task_last_status, task_input, task_output
                       FROM task
                       WHERE task_state = $1 AND task_type = $2"#,
                    state as _, ttype
                ).fetch_all(conn).await
            },
            (None, Some(ttype)) => {
                sqlx::query_as!(
                    Self,
                    r#"SELECT task_id, task_created_on, task_retry_count, task_type,
                              task_claimant, task_claimed_on, task_updated_on,
                              task_state as "task_state:_",
                              task_status, task_last_status, task_input, task_output
                       FROM task
                       WHERE task_type = $1"#,
                    ttype
                ).fetch_all(conn).await
            },
            (Some(state), None) => {
                sqlx::query_as!(
                    Self,
                    r#"SELECT task_id, task_created_on, task_retry_count, task_type,
                              task_claimant, task_claimed_on, task_updated_on,
                              task_state as "task_state:_",
                              task_status, task_last_status, task_input, task_output
                       FROM task
                       WHERE task_state = $1"#,
                    state as _
                ).fetch_all(conn).await
            }
            (None, None) => {
                sqlx::query_as!(
                    Self,
                    r#"SELECT task_id, task_created_on, task_retry_count, task_type,
                              task_claimant, task_claimed_on, task_updated_on,
                              task_state as "task_state:_",
                              task_status, task_last_status, task_input, task_output
                       FROM task"#
                ).fetch_all(conn).await
            }
        }
    }
    pub async fn claim_available(conn: &mut PgConnection, task_type:Option<&str>, claimant_id:&str) -> sqlx::Result<Option<Self>> {
        let mut tx = conn.begin().await?;
        let task = if let Some(task_type) = task_type {
            sqlx::query_as!(
                Self,
                r#"SELECT task_id, task_created_on, task_retry_count, task_type,
                          task_claimant, task_claimed_on, task_updated_on,
                          task_state as "task_state:_", task_status, task_last_status, task_input, task_output
                   FROM task
                   WHERE task_type = $1 AND
                     (task_state = 'idle' OR
                      (task_state = 'error' AND task_retry_count < $2))"#,
                task_type, TASK_RETRY_LIMIT
            )
                .fetch_optional(tx.as_mut())
                .await?
        } else {
            sqlx::query_as!(
                Self,
                r#"SELECT task_id, task_created_on, task_retry_count, task_type,
                          task_claimant, task_claimed_on, task_updated_on,
                          task_state as "task_state:_", task_status, task_last_status, task_input, task_output
                   FROM task
                   WHERE task_state = 'idle' OR
                     (task_state = 'error' AND task_retry_count < $1)"#,
                TASK_RETRY_LIMIT
            )
                .fetch_optional(tx.as_mut())
                .await?
        }.map(|t| Task{
            task_claimant: Some(claimant_id.to_string()),
            task_claimed_on: Some(Utc::now()),
            task_updated_on: Utc::now(),
            task_state: TaskState::Active,
            task_retry_count: t.task_retry_count+1,
            ..t
        });
        if let Some(task) = task {
            sqlx::query!(
                r#"UPDATE task
                   SET task_state='active', task_retry_count=$2,
                       task_claimant=$3, task_claimed_on=$4, task_updated_on=$5
                   WHERE task_id=$1"#,
                task.task_id, task.task_retry_count,
                task.task_claimant, task.task_claimed_on, task.task_updated_on
            ).execute(tx.as_mut()).await?;
            tx.commit().await?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }
    pub async fn create_conntest(conn:&mut PgConnection) -> sqlx::Result<Self> {
        let task = Task {
            task_id: Uuid::new_v4(),
            task_created_on: Utc::now(),
            task_retry_count: 0,
            task_type: "conntest".to_string(),
            task_claimant: None,
            task_claimed_on: None,
            task_updated_on: Utc::now(),
            task_state: TaskState::Idle,
            task_status: json!({}),
            task_last_status: None,
            task_input: json!({"example":0}),
            task_output: json!({}),
        };
        sqlx::query_as!(
            Self,
            r#"INSERT INTO task
               VALUES($1, $2, $3, $4, $5, $6, current_timestamp, $7, $8, $9, $10, $11)
               RETURNING task_id, task_created_on, task_retry_count,
                  task_type, task_claimant, task_claimed_on, task_updated_on,
                  task_state as "task_state:_",
                  task_status, task_last_status, task_input, task_output"#,
            task.task_id, task.task_created_on, task.task_retry_count,
            task.task_type, task.task_claimant, task.task_claimed_on,
            task.task_state as _,
            task.task_status, task.task_last_status, task.task_input, task.task_output,
        )
        .fetch_one(conn)
        .await
    }
    pub async fn update_status(conn:&mut PgConnection,id:Uuid,status:sqlx::types::JsonValue) -> sqlx::Result<()> {
        sqlx::query!(
            r#"UPDATE task
               SET task_status=$2, task_updated_on=current_timestamp
               WHERE task_id=$1"#,
            id,
            status
        ).execute(conn).await.map(|_qr| ())
    }
    pub async fn complete(conn:&mut PgConnection, task_id:Uuid, result:sqlx::types::JsonValue) -> sqlx::Result<()> {
        sqlx::query!(
            r#"UPDATE task
               SET task_output=$2, task_updated_on=current_timestamp, task_state='done'
               WHERE task_id=$1"#,
            task_id,
            result
        ).execute(conn).await.map(|_qr| ())
    }
    pub async fn error(conn:&mut PgConnection, task_id:Uuid, status:sqlx::types::JsonValue) -> sqlx::Result<()> {
        sqlx::query!(
            r#"UPDATE task
               SET task_last_status=$2, task_updated_on=current_timestamp, task_state='error'
               WHERE task_id=$1"#,
            task_id,
            status
        ).execute(conn).await.map(|_qr| ())
    }
    pub async fn timeout_stale(conn:&mut PgConnection, interval:std::time::Duration) -> Result<Vec<Self>, TimeoutTaskError> {
        sqlx::query_as!(
            Self,
            r#"UPDATE task
               SET task_updated_on=current_timestamp,task_state='error',
                   task_last_status=jsonb_set('{"reason":"timeout","status":{}}'::jsonb, '{status}', task_status),
                   task_status='{}'::jsonb
               WHERE current_timestamp - task_updated_on > $1
               RETURNING task_id, task_created_on, task_retry_count,
                  task_type, task_claimant, task_claimed_on, task_updated_on,
                  task_state as "task_state:_",
                  task_status, task_last_status, task_input, task_output"#,
            sqlx::postgres::types::PgInterval::try_from(interval).map_err(|_e| TimeoutTaskError::InvalidDuration(interval))?,
        ).fetch_all(conn).await.map_err(TimeoutTaskError::Sqlx)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPool;
    #[sqlx::test(migrations = "../migrations/")]
    async fn sorting_filtering(pool:PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let uuid_1 = Uuid::new_v4();
        let uuid_2 = Uuid::new_v4();
        let uuid_3 = Uuid::new_v4();
        let uuid_4 = Uuid::new_v4();
        let uuid_5 = Uuid::new_v4();
        sqlx::query!(r#"INSERT INTO task VALUES ($1, current_timestamp, 10, 'a', 'runner1', null, current_timestamp,'error','{"reason":"unknown", "progress":0.5}'::jsonb, '{"example":0}'::jsonb, '{}'::jsonb)"#, uuid_1).execute(conn.as_mut()).await?;
        sqlx::query!(r#"INSERT INTO task VALUES ($1, current_timestamp, 1, 'a', null, null, current_timestamp, 'error', '{}'::jsonb, '{"example":0}'::jsonb, '{}'::jsonb)"#, uuid_2).execute(conn.as_mut()).await?;
        sqlx::query!(r#"INSERT INTO task VALUES ($1, current_timestamp, 1, 'a', null, null, current_timestamp, 'done', '{"progress":1.0}'::jsonb, '{"example":0}'::jsonb, '{"example":0}'::jsonb)"#, uuid_3).execute(conn.as_mut()).await?;
        sqlx::query!(r#"INSERT INTO task VALUES ($1, current_timestamp, 1, 'b', null, null, current_timestamp, 'idle', '{}'::jsonb, '{"example":2}'::jsonb, '{}'::jsonb)"#, uuid_4).execute(conn.as_mut()).await?;
        sqlx::query!(r#"INSERT INTO task VALUES ($1, current_timestamp, 1, 'b', null, null, current_timestamp, 'error', '{}'::jsonb, '{"example":0}'::jsonb, '{}'::jsonb)"#, uuid_5).execute(conn.as_mut()).await?;
        assert_eq!(Task::get_tasks(conn.as_mut(), None, None).await?.len(), 5);
        assert_eq!(Task::get_tasks(conn.as_mut(), Some(TaskState::Done), None).await?.len(), 1);
        assert_eq!(Task::get_tasks(conn.as_mut(), None, Some("b")).await?.len(), 2);
        assert_eq!(Task::get_tasks(conn.as_mut(), Some(TaskState::Error), Some("a")).await?.len(), 2);
        let a_claim = Task::claim_available(conn.as_mut(), Some("a"), "test").await?.unwrap();
        assert_eq!(a_claim.task_id, uuid_2);
        Ok(())
    }
    #[sqlx::test(migrations = "../migrations/")]
    async fn updating(pool:PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let task = Task::create_conntest(conn.as_mut()).await?;
        let conntest_claim = Task::claim_available(conn.as_mut(), Some("conntest"), "test").await?.unwrap();
        assert_eq!(Task::claim_available(conn.as_mut(), Some("conntest"), "test").await?, None);
        assert_eq!(task.task_id, conntest_claim.task_id);
        assert_eq!(Task::get_by_id(conn.as_mut(), conntest_claim.task_id).await?.unwrap().task_state, TaskState::Active);
        Task::update_status(conn.as_mut(), conntest_claim.task_id, json!({"changed":1})).await?;
        assert_eq!(Task::get_by_id(conn.as_mut(), conntest_claim.task_id).await?.unwrap().task_status, json!({"changed":1}));
        Task::complete(conn.as_mut(), conntest_claim.task_id, json!({"finished":1})).await?;
        assert_eq!(Task::get_by_id(conn.as_mut(), conntest_claim.task_id).await?.unwrap().task_state, TaskState::Done);
        assert_eq!(Task::claim_available(conn.as_mut(), Some("conntest"), "test").await?, None);
        let task = Task::create_conntest(conn.as_mut()).await?;
        for i in 0..TASK_RETRY_LIMIT {
            let conntest_claim = Task::claim_available(conn.as_mut(), Some("conntest"), "test").await?.unwrap();
            Task::error(conn.as_mut(), conntest_claim.task_id, json!({"attempt":i})).await?;
            assert_eq!(Task::get_by_id(conn.as_mut(), task.task_id).await?.unwrap().task_state, TaskState::Error);
        }
        assert_eq!(Task::claim_available(conn.as_mut(), Some("conntest"), "test").await?, None);
        Ok(())
    }
    #[sqlx::test(migrations = "../migrations/")]
    async fn timeout_stale(pool:PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let t1id = Uuid::new_v4();
        sqlx::query!(r#"INSERT INTO task VALUES ($1, current_timestamp - interval '02:00.00', 1, 'a', 'test1', current_timestamp - interval '01:00.00', current_timestamp - interval '01:00.00', 'active', '{}'::jsonb, '{"example":2}'::jsonb, '{}'::jsonb)"#, t1id).execute(conn.as_mut()).await?;
        sqlx::query!(r#"INSERT INTO task VALUES (gen_random_uuid(), current_timestamp - interval '00:10.00', 1, 'a', 'test2', current_timestamp - interval '00:01.00', current_timestamp - interval '00:01.00', 'active', '{}'::jsonb, '{"example":2}'::jsonb, '{}'::jsonb)"#).execute(conn.as_mut()).await?;
        let cancelled = Task::timeout_stale(conn.as_mut(), std::time::Duration::from_secs(5)).await.unwrap();
        assert_eq!(cancelled.len(), 1);
        assert_eq!(cancelled[0].task_id, t1id);
        assert_eq!(cancelled[0].task_state, TaskState::Error);
        Ok(())
    }
    #[sqlx::test(migrations = "../migrations/")]
    async fn not_cancelled(pool:PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let t1id = Uuid::new_v4();
        sqlx::query!(r#"INSERT INTO task VALUES ($1, current_timestamp - interval '00:20.00', 1, 'a', 'test1', current_timestamp - interval '00:01.00', current_timestamp - interval '00:01.00', 'active', '{}'::jsonb, '{"example":2}'::jsonb, '{}'::jsonb)"#, t1id).execute(conn.as_mut()).await?;
        sqlx::query!(r#"INSERT INTO task VALUES (gen_random_uuid(), current_timestamp - interval '00:10.00', 1, 'a', 'test2', current_timestamp - interval '00:01.00', current_timestamp - interval '00:01.00', 'active', '{}'::jsonb, '{"example":2}'::jsonb, '{}'::jsonb)"#).execute(conn.as_mut()).await?;
        let cancelled = Task::timeout_stale(conn.as_mut(), std::time::Duration::from_secs(5)).await.unwrap();
        assert_eq!(cancelled.len(), 0);
        Ok(())
    }
}
