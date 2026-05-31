use crate::app_error::{AppError, AppResult};
use crate::local_db::LocalDb;
use crate::models::{KeyValue, Workspace, WorkspaceEnvironment, WorkspaceState};
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct WorkspaceService {
    db: LocalDb,
}

impl WorkspaceService {
    pub fn new(db: LocalDb) -> Self {
        Self { db }
    }

    pub async fn ensure_default_workspace(&self) -> AppResult<()> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE deleted_at IS NULL")
                .fetch_one(self.db.pool())
                .await?;

        if count.0 > 0 {
            return Ok(());
        }

        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO workspaces (
              id, name, is_default, last_opened_at, created_at, updated_at,
              revision, sync_status
            )
            VALUES (?1, 'Default Workspace', 1, ?2, ?2, ?2, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&now)
        .execute(self.db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO workspace_settings (
              workspace_id, layout_json, env_json, created_at, updated_at,
              revision, sync_status
            )
            VALUES (?1, '{}', '{}', ?2, ?2, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&now)
        .execute(self.db.pool())
        .await?;

        self.write_setting("active_workspace_id", &id).await
    }

    pub async fn state(&self) -> AppResult<WorkspaceState> {
        let workspaces = self.list().await?;
        let active_workspace_id = self.active_workspace_id(&workspaces).await?;

        Ok(WorkspaceState {
            active_workspace_id,
            workspaces,
        })
    }

    pub async fn list(&self) -> AppResult<Vec<Workspace>> {
        let items = sqlx::query_as::<_, Workspace>(
            r#"
            SELECT
              id, name, is_default, last_opened_at, created_at, updated_at,
              deleted_at, revision, sync_status, remote_id
            FROM workspaces
            WHERE deleted_at IS NULL
            ORDER BY is_default DESC, last_opened_at DESC, created_at ASC
            "#,
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(items)
    }

    pub async fn create(&self, name: String) -> AppResult<Workspace> {
        let name = normalize_name(name)?;
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO workspaces (
              id, name, is_default, last_opened_at, created_at, updated_at,
              revision, sync_status
            )
            VALUES (?1, ?2, 0, ?3, ?3, ?3, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&name)
        .bind(&now)
        .execute(self.db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO workspace_settings (
              workspace_id, layout_json, env_json, created_at, updated_at,
              revision, sync_status
            )
            VALUES (?1, '{}', '{}', ?2, ?2, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&now)
        .execute(self.db.pool())
        .await?;

        self.write_setting("active_workspace_id", &id).await?;
        self.get(&id).await
    }

    pub async fn rename(&self, workspace_id: String, name: String) -> AppResult<Workspace> {
        let name = normalize_name(name)?;
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE workspaces
            SET name = ?1, updated_at = ?2, revision = revision + 1, sync_status = 'pending'
            WHERE id = ?3 AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .bind(now)
        .bind(&workspace_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("workspace".to_string()));
        }

        self.get(&workspace_id).await
    }

    pub async fn delete(&self, workspace_id: String) -> AppResult<WorkspaceState> {
        let active_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE deleted_at IS NULL")
                .fetch_one(self.db.pool())
                .await?;

        if active_count.0 <= 1 {
            return Err(AppError::Validation(
                "at least one workspace must remain".to_string(),
            ));
        }

        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"
            UPDATE workspaces
            SET deleted_at = ?1, updated_at = ?1, revision = revision + 1, sync_status = 'deleted'
            WHERE id = ?2 AND deleted_at IS NULL
            "#,
        )
        .bind(&now)
        .bind(&workspace_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("workspace".to_string()));
        }

        let active = self.read_setting("active_workspace_id").await?;
        if active.as_deref() == Some(&workspace_id) {
            let next: (String,) = sqlx::query_as(
                r#"
                SELECT id FROM workspaces
                WHERE deleted_at IS NULL
                ORDER BY is_default DESC, updated_at DESC
                LIMIT 1
                "#,
            )
            .fetch_one(self.db.pool())
            .await?;
            self.write_setting("active_workspace_id", &next.0).await?;
        }

        self.state().await
    }

    pub async fn set_active(&self, workspace_id: String) -> AppResult<WorkspaceState> {
        self.get(&workspace_id).await?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE workspaces SET last_opened_at = ?1, updated_at = ?1 WHERE id = ?2",
        )
        .bind(&now)
        .bind(&workspace_id)
        .execute(self.db.pool())
        .await?;

        self.write_setting("active_workspace_id", &workspace_id).await?;
        self.state().await
    }

    pub async fn environment(&self, workspace_id: String) -> AppResult<WorkspaceEnvironment> {
        self.get(&workspace_id).await?;

        let row: (String, String) = sqlx::query_as(
            r#"
            SELECT env_json, updated_at
            FROM workspace_settings
            WHERE workspace_id = ?1 AND deleted_at IS NULL
            "#,
        )
        .bind(&workspace_id)
        .fetch_one(self.db.pool())
        .await?;

        let variables = serde_json::from_str::<Vec<KeyValue>>(&row.0).unwrap_or_default();

        Ok(WorkspaceEnvironment {
            workspace_id,
            variables,
            updated_at: row.1,
        })
    }

    pub async fn update_environment(
        &self,
        workspace_id: String,
        variables: Vec<KeyValue>,
    ) -> AppResult<WorkspaceEnvironment> {
        self.get(&workspace_id).await?;
        validate_environment(&variables)?;

        let now = Utc::now().to_rfc3339();
        let env_json = serde_json::to_string(&variables)?;

        sqlx::query(
            r#"
            UPDATE workspace_settings
            SET env_json = ?1, updated_at = ?2, revision = revision + 1, sync_status = 'pending'
            WHERE workspace_id = ?3 AND deleted_at IS NULL
            "#,
        )
        .bind(env_json)
        .bind(&now)
        .bind(&workspace_id)
        .execute(self.db.pool())
        .await?;

        self.environment(workspace_id).await
    }

    async fn get(&self, workspace_id: &str) -> AppResult<Workspace> {
        let workspace = sqlx::query_as::<_, Workspace>(
            r#"
            SELECT
              id, name, is_default, last_opened_at, created_at, updated_at,
              deleted_at, revision, sync_status, remote_id
            FROM workspaces
            WHERE id = ?1 AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .fetch_optional(self.db.pool())
        .await?;

        workspace.ok_or_else(|| AppError::NotFound("workspace".to_string()))
    }

    async fn active_workspace_id(&self, workspaces: &[Workspace]) -> AppResult<String> {
        let stored = self.read_setting("active_workspace_id").await?;
        if let Some(id) = stored {
            if workspaces.iter().any(|workspace| workspace.id == id) {
                return Ok(id);
            }
        }

        let fallback = workspaces
            .first()
            .ok_or_else(|| AppError::NotFound("workspace".to_string()))?;
        self.write_setting("active_workspace_id", &fallback.id).await?;
        Ok(fallback.id.clone())
    }

    async fn read_setting(&self, key: &str) -> AppResult<Option<String>> {
        let value: Option<(String,)> =
            sqlx::query_as("SELECT value FROM app_settings WHERE key = ?1")
                .bind(key)
                .fetch_optional(self.db.pool())
                .await?;

        Ok(value.map(|item| item.0))
    }

    async fn write_setting(&self, key: &str, value: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(Utc::now().to_rfc3339())
        .execute(self.db.pool())
        .await?;

        Ok(())
    }
}

fn normalize_name(name: String) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "workspace name cannot be empty".to_string(),
        ));
    }
    if trimmed.chars().count() > 80 {
        return Err(AppError::Validation(
            "workspace name must be 80 characters or fewer".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_environment(variables: &[KeyValue]) -> AppResult<()> {
    for variable in variables {
        let key = variable.key.trim();
        if key.is_empty() {
            continue;
        }
        let valid = key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'));
        if !valid {
            return Err(AppError::Validation(format!(
                "invalid environment variable name: {}",
                variable.key
            )));
        }
    }

    Ok(())
}
