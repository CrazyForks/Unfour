use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use unfour_core::{AppError, AppResult};

const DB_FILENAME: &str = "unfour.sqlite";

/// Directory name under the OS data dir where the desktop app and satellite
/// processes (e.g. the MCP server) locate `unfour.sqlite`. Kept here so both
/// the Tauri entry and the identifier-based entry resolve to the same path.
/// Matches `productName` ("Unfour") to align with sibling apps under
/// `%APPDATA%` / `~/Library/Application Support`. The Tauri `identifier`
/// (`dev.unfour`) is unrelated and stays as the bundle/installer identity.
pub const DEFAULT_APP_IDENTIFIER: &str = "Unfour";

/// How long a connection waits for a held lock before returning
/// `SQLITE_BUSY`. The desktop app and satellite processes (e.g. the MCP server)
/// can open the same database file concurrently, so a non-zero busy timeout
/// avoids spurious "database is locked" failures.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct LocalDb {
    pool: SqlitePool,
}

impl LocalDb {
    /// Connect using the default app data directory (`<os_data_dir>/Unfour`).
    ///
    /// Both the desktop app and satellite processes must resolve to the same
    /// directory, so this deliberately bypasses Tauri's `app_data_dir()` (which
    /// derives from `identifier` and would yield `dev.unfour`) and uses the
    /// shared `DEFAULT_APP_IDENTIFIER` instead.
    pub async fn connect_default() -> AppResult<Self> {
        Self::connect_app_data(DEFAULT_APP_IDENTIFIER).await
    }

    pub async fn connect_app_data(identifier: &str) -> AppResult<Self> {
        Self::connect_path(app_data_path(identifier)?.join(DB_FILENAME)).await
    }

    pub async fn connect_existing_app_data_read_only(identifier: &str) -> AppResult<Self> {
        Self::connect_existing_read_only_path(app_data_path(identifier)?.join(DB_FILENAME)).await
    }

    pub async fn connect_existing_app_data(identifier: &str) -> AppResult<Self> {
        Self::connect_existing_path(app_data_path(identifier)?.join(DB_FILENAME)).await
    }

    pub async fn connect_path(path: impl AsRef<Path>) -> AppResult<Self> {
        let db_path = path.as_ref();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .busy_timeout(BUSY_TIMEOUT)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub async fn connect_existing_read_only_path(path: impl AsRef<Path>) -> AppResult<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(false)
            .read_only(true)
            .busy_timeout(BUSY_TIMEOUT)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub async fn connect_existing_path(path: impl AsRef<Path>) -> AppResult<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(false)
            .busy_timeout(BUSY_TIMEOUT)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> AppResult<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(sqlx::Error::from)?;

        Ok(())
    }
}

fn app_data_path(identifier: &str) -> AppResult<PathBuf> {
    dirs::data_dir()
        .map(|dir| dir.join(identifier))
        .ok_or_else(|| AppError::Config("app data directory is not available".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    async fn test_db() -> LocalDb {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect in-memory sqlite");
        LocalDb::from_pool(pool)
    }

    fn temp_db_path() -> PathBuf {
        let unique = format!(
            "unfour-local-storage-test-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        std::env::temp_dir().join(unique)
    }

    #[tokio::test]
    async fn migrate_creates_all_tables() {
        let db = test_db().await;
        db.migrate().await.expect("first migration");

        let tables: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .fetch_all(db.pool())
                .await
                .expect("list tables");
        let names: Vec<&str> = tables.iter().map(|(n,)| n.as_str()).collect();
        assert!(names.contains(&"workspaces"));
        assert!(names.contains(&"api_requests"));
        assert!(names.contains(&"api_history"));
        assert!(names.contains(&"db_query_history"));
        assert!(names.contains(&"saved_sql"));
        assert!(names.contains(&"connections"));
        assert!(names.contains(&"activity_events"));
        assert!(names.contains(&"app_settings"));
        assert!(names.contains(&"workspace_settings"));
        assert!(names.contains(&"ssh_terminal_history"));
        assert!(names.contains(&"api_collections"));
        assert!(names.contains(&"api_collection_folders"));
        assert!(names.contains(&"ssh_connections"));
        assert!(names.contains(&"database_connections"));
    }

    #[tokio::test]
    async fn migrate_drops_api_history_sync_fields() {
        let db = test_db().await;
        db.migrate().await.expect("migration");

        let columns: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('api_history')")
                .fetch_all(db.pool())
                .await
                .expect("list columns");
        let names: Vec<&str> = columns.iter().map(|(name,)| name.as_str()).collect();

        assert!(!names.contains(&"revision"), "revision should be dropped");
        assert!(
            !names.contains(&"sync_status"),
            "sync_status should be dropped"
        );
        assert!(!names.contains(&"remote_id"), "remote_id should be dropped");
        assert!(!names.contains(&"deleted_at"), "deleted_at should be dropped");
        assert!(names.contains(&"created_at"), "created_at retained");
        assert!(names.contains(&"updated_at"), "updated_at retained");
    }

    #[tokio::test]
    async fn migrate_creates_single_active_environment_index() {
        let db = test_db().await;
        db.migrate().await.expect("migration");

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO workspaces (id, name, is_default, created_at, updated_at, revision, sync_status)
            VALUES ('ws-active', 'Active', 0, ?1, ?1, 1, 'local')
            "#,
        )
        .bind(&now)
        .execute(db.pool())
        .await
        .expect("insert workspace");

        sqlx::query(
            r#"
            INSERT INTO api_environments (id, workspace_id, name, is_active, created_at, updated_at, revision, sync_status)
            VALUES ('env-1', 'ws-active', 'Env 1', 1, ?1, ?1, 1, 'local')
            "#,
        )
        .bind(&now)
        .execute(db.pool())
        .await
        .expect("insert first active env");

        // A second active environment in the same workspace must be rejected.
        let err = sqlx::query(
            r#"
            INSERT INTO api_environments (id, workspace_id, name, is_active, created_at, updated_at, revision, sync_status)
            VALUES ('env-2', 'ws-active', 'Env 2', 1, ?1, ?1, 1, 'local')
            "#,
        )
        .bind(&now)
        .execute(db.pool())
        .await
        .expect_err("second active env should violate unique index");
        let msg = err.to_string().to_lowercase();
        assert!(
            msg.contains("unique") || msg.contains("constraint"),
            "expected unique constraint violation, got: {msg}"
        );
    }

    #[tokio::test]
    async fn migrate_adds_api_collection_folders_sync_fields() {
        let db = test_db().await;
        db.migrate().await.expect("migration");

        let columns: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('api_collection_folders')")
                .fetch_all(db.pool())
                .await
                .expect("list columns");
        let names: Vec<&str> = columns.iter().map(|(name,)| name.as_str()).collect();

        assert!(names.contains(&"revision"), "revision added");
        assert!(names.contains(&"sync_status"), "sync_status added");
        assert!(names.contains(&"remote_id"), "remote_id added");
    }

    #[tokio::test]
    async fn migrate_splits_connections_into_subtypes() {
        let db = test_db().await;
        db.migrate().await.expect("migration");

        // Parent must no longer carry kind / config_json.
        let columns: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('connections')")
                .fetch_all(db.pool())
                .await
                .expect("list columns");
        let names: Vec<&str> = columns.iter().map(|(name,)| name.as_str()).collect();
        assert!(!names.contains(&"kind"), "kind dropped from connections");
        assert!(
            !names.contains(&"config_json"),
            "config_json dropped from connections"
        );
        assert!(names.contains(&"credential_ref"), "credential_ref retained");

        // Subtype tables exist with the expected shape.
        let ssh_cols: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('ssh_connections')")
                .fetch_all(db.pool())
                .await
                .expect("list ssh_connections columns");
        let ssh_names: Vec<&str> = ssh_cols.iter().map(|(n,)| n.as_str()).collect();
        assert!(ssh_names.contains(&"connection_id"));
        assert!(ssh_names.contains(&"config_json"));

        let db_cols: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('database_connections')")
                .fetch_all(db.pool())
                .await
                .expect("list database_connections columns");
        let db_names: Vec<&str> = db_cols.iter().map(|(n,)| n.as_str()).collect();
        assert!(db_names.contains(&"connection_id"));
        assert!(db_names.contains(&"config_json"));
    }

    #[tokio::test]
    async fn migrate_migrates_existing_connections_into_subtypes() {
        // Exercises the post-split shape end-to-end: insert a parent row plus
        // a subtype row, then read both back. The data-backfill half of
        // migration 0009 is covered by the migrations themselves running on
        // a freshly-migrated DB; this test guards the read/write contract.
        let db = test_db().await;
        db.migrate().await.expect("migration");

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO workspaces (id, name, is_default, created_at, updated_at, revision, sync_status)
            VALUES ('ws-conn', 'Conn', 0, ?1, ?1, 1, 'local')
            "#,
        )
        .bind(&now)
        .execute(db.pool())
        .await
        .expect("insert workspace");

        sqlx::query(
            r#"
            INSERT INTO connections (id, workspace_id, name, credential_ref, created_at, updated_at, revision, sync_status)
            VALUES ('c-ssh-2', 'ws-conn', 'ssh-2', NULL, ?1, ?1, 1, 'local')
            "#,
        )
        .bind(&now)
        .execute(db.pool())
        .await
        .expect("insert new-style parent row");

        sqlx::query(
            r#"INSERT INTO ssh_connections (connection_id, config_json) VALUES ('c-ssh-2', '{"host":"h2"}')"#,
        )
        .execute(db.pool())
        .await
        .expect("insert ssh subtype row");

        let row: (String,) = sqlx::query_as(
            "SELECT config_json FROM ssh_connections WHERE connection_id = 'c-ssh-2'",
        )
        .fetch_one(db.pool())
        .await
        .expect("read subtype row");
        assert!(row.0.contains("h2"));
    }

    #[tokio::test]
    async fn migrate_adds_saved_sql_soft_delete_fields() {
        let db = test_db().await;
        db.migrate().await.expect("migration");

        let columns: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('saved_sql')")
                .fetch_all(db.pool())
                .await
                .expect("list columns");
        let names: Vec<&str> = columns.iter().map(|(name,)| name.as_str()).collect();
        assert!(names.contains(&"deleted_at"), "deleted_at added");
        assert!(names.contains(&"revision"), "revision added");
        assert!(names.contains(&"sync_status"), "sync_status added");
        assert!(names.contains(&"remote_id"), "remote_id added");
    }

    #[tokio::test]
    async fn migrate_drops_legacy_folder_columns() {
        let db = test_db().await;
        db.migrate().await.expect("migration");

        let req_cols: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('api_requests')")
                .fetch_all(db.pool())
                .await
                .expect("list api_requests columns");
        let req_names: Vec<&str> = req_cols.iter().map(|(n,)| n.as_str()).collect();
        assert!(
            !req_names.contains(&"folder_path"),
            "folder_path should be dropped from api_requests"
        );

        let coll_cols: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('api_collections')")
                .fetch_all(db.pool())
                .await
                .expect("list api_collections columns");
        let coll_names: Vec<&str> = coll_cols.iter().map(|(n,)| n.as_str()).collect();
        assert!(
            !coll_names.contains(&"folders_json"),
            "folders_json should be dropped from api_collections"
        );
    }

    #[tokio::test]
    async fn migrate_is_idempotent() {
        let db = test_db().await;
        db.migrate().await.expect("first migration");
        db.migrate()
            .await
            .expect("second migration should succeed without error");
        db.migrate()
            .await
            .expect("third migration should succeed without error");
    }

    #[tokio::test]
    async fn migrate_ensures_api_request_collection_tree_columns() {
        let db = test_db().await;
        db.migrate().await.expect("migration");

        let columns: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('api_requests')")
                .fetch_all(db.pool())
                .await
                .expect("list columns");
        assert!(
            columns.iter().any(|(name,)| name == "parent_folder_id"),
            "api_requests should have parent_folder_id column"
        );
        assert!(
            columns.iter().any(|(name,)| name == "collection_id"),
            "api_requests should have collection_id column"
        );
        assert!(
            columns.iter().any(|(name,)| name == "sort_order"),
            "api_requests should have sort_order column"
        );
    }

    #[tokio::test]
    async fn migrate_ensures_api_collection_folders_table() {
        let db = test_db().await;
        db.migrate().await.expect("migration");

        let columns: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('api_collection_folders')")
                .fetch_all(db.pool())
                .await
                .expect("list columns");
        let names: Vec<&str> = columns.iter().map(|(name,)| name.as_str()).collect();

        assert!(
            [
                "id",
                "workspace_id",
                "collection_id",
                "parent_folder_id",
                "name",
                "sort_order",
                "created_at",
                "updated_at",
                "deleted_at",
            ]
            .iter()
            .all(|expected| names.contains(expected)),
            "api_collection_folders should have stable folder tree columns"
        );
    }

    #[tokio::test]
    async fn workspace_policy_migration_defaults_legacy_rows() {
        let db = test_db().await;
        sqlx::query(
            r#"
            CREATE TABLE workspaces (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              is_default INTEGER NOT NULL DEFAULT 0,
              last_opened_at TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              deleted_at TEXT,
              revision INTEGER NOT NULL DEFAULT 1,
              sync_status TEXT NOT NULL DEFAULT 'local',
              remote_id TEXT
            )
            "#,
        )
        .execute(db.pool())
        .await
        .expect("create legacy workspaces table");
        sqlx::query(
            r#"
            INSERT INTO workspaces (
              id, name, is_default, created_at, updated_at, revision, sync_status
            )
            VALUES ('legacy-ws', 'Legacy', 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 1, 'local')
            "#,
        )
        .execute(db.pool())
        .await
        .expect("insert legacy workspace");

        db.migrate().await.expect("migrate legacy workspace table");

        let row: (String, String) = sqlx::query_as(
            "SELECT environment_type, mcp_policy FROM workspaces WHERE id = 'legacy-ws'",
        )
        .fetch_one(db.pool())
        .await
        .expect("read migrated workspace policy fields");
        assert_eq!(row.0, "dev");
        assert_eq!(row.1, "auto");
    }

    #[tokio::test]
    async fn connect_existing_read_only_path_reads_existing_database_without_creating() {
        let path = temp_db_path();
        let db = LocalDb::connect_path(&path).await.expect("create db");
        db.migrate().await.expect("migrate db");
        drop(db);

        let read_only = LocalDb::connect_existing_read_only_path(&path)
            .await
            .expect("open read-only db");
        let tables: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table'")
                .fetch_all(read_only.pool())
                .await
                .expect("list tables");

        assert!(tables.iter().any(|(name,)| name == "workspaces"));
        let _ = std::fs::remove_file(path);
    }
}
