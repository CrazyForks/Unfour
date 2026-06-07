use crate::LocalDb;
use chrono::Utc;
use serde_json::Value;
use unfour_core::AppResult;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActivityLogService {
    db: LocalDb,
}

impl ActivityLogService {
    pub fn new(db: LocalDb) -> Self {
        Self { db }
    }

    pub async fn record(
        &self,
        workspace_id: Option<&str>,
        action: &str,
        target: Option<&str>,
        details: Value,
    ) -> AppResult<()> {
        // This is a local activity trail, not a compliance log. Callers should
        // pass only redacted summaries and avoid routine read/UI noise.
        sqlx::query(
            r#"
            INSERT INTO activity_events (id, workspace_id, action, target, details_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(workspace_id)
        .bind(action)
        .bind(target)
        .bind(serde_json::to_string(&details)?)
        .bind(Utc::now().to_rfc3339())
        .execute(self.db.pool())
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    async fn service() -> ActivityLogService {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect in-memory sqlite");
        let db = LocalDb::from_pool(pool);
        db.migrate().await.expect("run migrations");
        ActivityLogService::new(db)
    }

    #[tokio::test]
    async fn record_inserts_event() {
        let svc = service().await;
        svc.record(
            Some("ws-1"),
            "workspace.create",
            Some("ws-1"),
            serde_json::json!({ "name": "Test" }),
        )
        .await
        .expect("record event");

        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT workspace_id, action, target FROM activity_events ORDER BY created_at",
        )
        .fetch_all(svc.db.pool())
        .await
        .expect("fetch events");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "ws-1");
        assert_eq!(rows[0].1, "workspace.create");
        assert_eq!(rows[0].2, "ws-1");
    }

    #[tokio::test]
    async fn record_multiple_events() {
        let svc = service().await;
        svc.record(Some("ws-1"), "action.a", None, serde_json::json!({}))
            .await
            .unwrap();
        svc.record(
            None,
            "action.b",
            Some("target"),
            serde_json::json!({ "key": "val" }),
        )
        .await
        .unwrap();
        svc.record(Some("ws-2"), "action.c", None, serde_json::json!({}))
            .await
            .unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM activity_events")
            .fetch_one(svc.db.pool())
            .await
            .expect("count events");
        assert_eq!(count.0, 3);
    }

    #[tokio::test]
    async fn record_stores_json_details() {
        let svc = service().await;
        svc.record(
            Some("ws-1"),
            "test.action",
            None,
            serde_json::json!({ "count": 42, "label": "hello" }),
        )
        .await
        .unwrap();

        let details: (String,) =
            sqlx::query_as("SELECT details_json FROM activity_events WHERE action = 'test.action'")
                .fetch_one(svc.db.pool())
                .await
                .expect("fetch details");

        let parsed: Value = serde_json::from_str(&details.0).expect("parse json");
        assert_eq!(parsed["count"], 42);
        assert_eq!(parsed["label"], "hello");
    }
}
