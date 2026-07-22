use std::collections::{HashMap, HashSet};

use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use unfour_core::models::{
    WorkspaceEnvironment, WorkspaceEnvironmentVariable, WorkspaceVariable, WorkspaceVariableInput,
};
use unfour_core::{AppError, AppResult};

use super::WorkspaceService;

#[derive(sqlx::FromRow)]
struct WorkspaceEnvironmentRow {
    id: String,
    workspace_id: String,
    name: String,
    sort_order: i64,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
    revision: i64,
    sync_status: String,
    remote_id: Option<String>,
}

impl WorkspaceService {
    pub async fn list_variables(&self, workspace_id: String) -> AppResult<Vec<WorkspaceVariable>> {
        self.get(&workspace_id).await?;
        let variables = sqlx::query_as::<_, WorkspaceVariable>(
            r#"
            SELECT
              id, workspace_id, key, value, is_secret, is_enabled, description,
              sort_order, created_at, updated_at, deleted_at, revision,
              sync_status, remote_id
            FROM workspace_variables
            WHERE workspace_id = ?1 AND deleted_at IS NULL
            ORDER BY sort_order ASC, created_at ASC, id ASC
            "#,
        )
        .bind(&workspace_id)
        .fetch_all(self.db.pool())
        .await?;
        Ok(variables)
    }

    pub async fn replace_variables(
        &self,
        workspace_id: String,
        variables: Vec<WorkspaceVariableInput>,
    ) -> AppResult<Vec<WorkspaceVariable>> {
        self.get(&workspace_id).await?;
        validate_variables(&variables)?;
        let mut tx = self.db.pool().begin().await?;
        replace_workspace_variables(&mut tx, &workspace_id, variables).await?;
        tx.commit().await?;
        self.list_variables(workspace_id).await
    }

    pub async fn list_environments(
        &self,
        workspace_id: String,
    ) -> AppResult<Vec<WorkspaceEnvironment>> {
        self.get(&workspace_id).await?;
        let active_environment_id = self.active_environment_id(&workspace_id).await?;
        let rows = sqlx::query_as::<_, WorkspaceEnvironmentRow>(
            r#"
            SELECT
              id, workspace_id, name, sort_order, created_at, updated_at,
              deleted_at, revision, sync_status, remote_id
            FROM workspace_environments
            WHERE workspace_id = ?1 AND deleted_at IS NULL
            ORDER BY sort_order ASC, created_at ASC, id ASC
            "#,
        )
        .bind(&workspace_id)
        .fetch_all(self.db.pool())
        .await?;

        let mut environments = Vec::with_capacity(rows.len());
        for row in rows {
            let variables = self
                .list_environment_variables(&workspace_id, &row.id)
                .await?;
            environments.push(WorkspaceEnvironment {
                is_active: active_environment_id.as_deref() == Some(row.id.as_str()),
                id: row.id,
                workspace_id: row.workspace_id,
                name: row.name,
                sort_order: row.sort_order,
                variables,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
                revision: row.revision,
                sync_status: row.sync_status,
                remote_id: row.remote_id,
            });
        }
        Ok(environments)
    }

    pub async fn create_environment(
        &self,
        workspace_id: String,
        name: String,
    ) -> AppResult<WorkspaceEnvironment> {
        self.get(&workspace_id).await?;
        let name = normalize_environment_name(name)?;
        self.assert_environment_name_unique(&workspace_id, &name, None)
            .await?;
        let now = Utc::now().to_rfc3339();
        let id = unfour_core::id::new_id();
        // The first environment in a workspace becomes the active one.
        let existing: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM workspace_environments WHERE workspace_id = ?1 AND deleted_at IS NULL",
        )
        .bind(&workspace_id)
        .fetch_one(self.db.pool())
        .await?;
        let (sort_order,): (i64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(MAX(sort_order), -1) + 1
            FROM workspace_environments
            WHERE workspace_id = ?1 AND deleted_at IS NULL
            "#,
        )
        .bind(&workspace_id)
        .fetch_one(self.db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO workspace_environments (
              id, workspace_id, name, sort_order, created_at, updated_at,
              revision, sync_status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?5, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&workspace_id)
        .bind(name)
        .bind(sort_order)
        .bind(&now)
        .execute(self.db.pool())
        .await?;

        if existing == 0 {
            sqlx::query(
                r#"
                UPDATE workspace_local_state
                SET active_environment_id = ?1, updated_at = ?2
                WHERE workspace_id = ?3
                "#,
            )
            .bind(&id)
            .bind(&now)
            .bind(&workspace_id)
            .execute(self.db.pool())
            .await?;
        }

        self.get_environment(&workspace_id, &id).await
    }

    pub async fn update_environment(
        &self,
        workspace_id: String,
        environment_id: String,
        name: String,
        variables: Vec<WorkspaceVariableInput>,
    ) -> AppResult<WorkspaceEnvironment> {
        self.get(&workspace_id).await?;
        let name = normalize_environment_name(name)?;
        validate_variables(&variables)?;
        self.assert_environment_name_unique(&workspace_id, &name, Some(&environment_id))
            .await?;

        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.pool().begin().await?;
        let updated = sqlx::query(
            r#"
            UPDATE workspace_environments
            SET name = ?1, updated_at = ?2, revision = revision + 1,
                sync_status = 'pending'
            WHERE id = ?3 AND workspace_id = ?4 AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .bind(&now)
        .bind(&environment_id)
        .bind(&workspace_id)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(AppError::NotFound("workspace environment".to_string()));
        }
        replace_environment_variables(&mut tx, &workspace_id, &environment_id, variables).await?;
        tx.commit().await?;
        self.get_environment(&workspace_id, &environment_id).await
    }

    pub async fn delete_environment(
        &self,
        workspace_id: String,
        environment_id: String,
    ) -> AppResult<Vec<WorkspaceEnvironment>> {
        self.get(&workspace_id).await?;
        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.pool().begin().await?;
        let deleted = sqlx::query(
            r#"
            UPDATE workspace_environments
            SET deleted_at = ?1, updated_at = ?1, revision = revision + 1,
                sync_status = 'deleted'
            WHERE id = ?2 AND workspace_id = ?3 AND deleted_at IS NULL
            "#,
        )
        .bind(&now)
        .bind(&environment_id)
        .bind(&workspace_id)
        .execute(&mut *tx)
        .await?;
        if deleted.rows_affected() == 0 {
            return Err(AppError::NotFound("workspace environment".to_string()));
        }
        sqlx::query(
            r#"
            UPDATE workspace_environment_variables
            SET deleted_at = ?1, updated_at = ?1, revision = revision + 1,
                sync_status = 'deleted'
            WHERE workspace_id = ?2 AND environment_id = ?3 AND deleted_at IS NULL
            "#,
        )
        .bind(&now)
        .bind(&workspace_id)
        .bind(&environment_id)
        .execute(&mut *tx)
        .await?;

        let active: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT active_environment_id FROM workspace_local_state WHERE workspace_id = ?1",
        )
        .bind(&workspace_id)
        .fetch_optional(&mut *tx)
        .await?;
        if active.and_then(|(id,)| id).as_deref() == Some(environment_id.as_str()) {
            let fallback: Option<(String,)> = sqlx::query_as(
                r#"
                SELECT id
                FROM workspace_environments
                WHERE workspace_id = ?1 AND deleted_at IS NULL
                ORDER BY sort_order ASC, created_at ASC, id ASC
                LIMIT 1
                "#,
            )
            .bind(&workspace_id)
            .fetch_optional(&mut *tx)
            .await?;
            sqlx::query(
                r#"
                UPDATE workspace_local_state
                SET active_environment_id = ?1, updated_at = ?2
                WHERE workspace_id = ?3
                "#,
            )
            .bind(fallback.map(|(id,)| id))
            .bind(&now)
            .bind(&workspace_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.list_environments(workspace_id).await
    }

    pub async fn set_active_environment(
        &self,
        workspace_id: String,
        environment_id: Option<String>,
    ) -> AppResult<Vec<WorkspaceEnvironment>> {
        self.get(&workspace_id).await?;
        let environment_id = non_empty_optional(environment_id);
        if let Some(environment_id) = environment_id.as_deref() {
            self.get_environment(&workspace_id, environment_id).await?;
        }
        sqlx::query(
            r#"
            UPDATE workspace_local_state
            SET active_environment_id = ?1, updated_at = ?2
            WHERE workspace_id = ?3
            "#,
        )
        .bind(environment_id)
        .bind(Utc::now().to_rfc3339())
        .bind(&workspace_id)
        .execute(self.db.pool())
        .await?;
        self.list_environments(workspace_id).await
    }

    pub async fn active_environment_id(&self, workspace_id: &str) -> AppResult<Option<String>> {
        self.get(workspace_id).await?;
        let active: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT environment.id
            FROM workspace_local_state AS settings
            JOIN workspace_environments AS environment
              ON environment.id = settings.active_environment_id
             AND environment.workspace_id = settings.workspace_id
             AND environment.deleted_at IS NULL
            WHERE settings.workspace_id = ?1
            "#,
        )
        .bind(workspace_id)
        .fetch_optional(self.db.pool())
        .await?;
        Ok(active.map(|(id,)| id))
    }

    /// Resolve `{{VARIABLE}}` tokens for one workspace and an explicitly
    /// supplied active environment. Environment values overlay workspace
    /// values; an environment from another workspace is rejected before any
    /// of its variables are read.
    pub async fn resolve_variables(
        &self,
        workspace_id: &str,
        active_environment_id: Option<&str>,
        input: &str,
    ) -> AppResult<String> {
        self.get(workspace_id).await?;
        let mut values = HashMap::new();
        let workspace_variables = sqlx::query_as::<_, WorkspaceVariable>(
            r#"
            SELECT
              id, workspace_id, key, value, is_secret, is_enabled, description,
              sort_order, created_at, updated_at, deleted_at, revision,
              sync_status, remote_id
            FROM workspace_variables
            WHERE workspace_id = ?1 AND is_enabled = 1 AND deleted_at IS NULL
            ORDER BY sort_order ASC, created_at ASC, id ASC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(self.db.pool())
        .await?;
        for variable in workspace_variables {
            values.insert(variable.key, variable.value);
        }

        if let Some(environment_id) = active_environment_id.filter(|id| !id.trim().is_empty()) {
            // This lookup intentionally includes workspace_id so a foreign
            // environment ID is indistinguishable from a missing one.
            self.get_environment(workspace_id, environment_id).await?;
            let environment_variables = self
                .list_environment_variables(workspace_id, environment_id)
                .await?;
            for variable in environment_variables
                .into_iter()
                .filter(|variable| variable.is_enabled)
            {
                values.insert(variable.key, variable.value);
            }
        }

        resolve_template(input, &values)
    }

    async fn get_environment(
        &self,
        workspace_id: &str,
        environment_id: &str,
    ) -> AppResult<WorkspaceEnvironment> {
        let row = sqlx::query_as::<_, WorkspaceEnvironmentRow>(
            r#"
            SELECT
              id, workspace_id, name, sort_order, created_at, updated_at,
              deleted_at, revision, sync_status, remote_id
            FROM workspace_environments
            WHERE id = ?1 AND workspace_id = ?2 AND deleted_at IS NULL
            "#,
        )
        .bind(environment_id)
        .bind(workspace_id)
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound("workspace environment".to_string()))?;
        let variables = self
            .list_environment_variables(workspace_id, environment_id)
            .await?;
        let active_environment_id = self.active_environment_id(workspace_id).await?;
        Ok(WorkspaceEnvironment {
            is_active: active_environment_id.as_deref() == Some(environment_id),
            id: row.id,
            workspace_id: row.workspace_id,
            name: row.name,
            sort_order: row.sort_order,
            variables,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
            revision: row.revision,
            sync_status: row.sync_status,
            remote_id: row.remote_id,
        })
    }

    async fn list_environment_variables(
        &self,
        workspace_id: &str,
        environment_id: &str,
    ) -> AppResult<Vec<WorkspaceEnvironmentVariable>> {
        let variables = sqlx::query_as::<_, WorkspaceEnvironmentVariable>(
            r#"
            SELECT
              id, workspace_id, environment_id, key, value, is_secret,
              is_enabled, description, sort_order, created_at, updated_at,
              deleted_at, revision, sync_status, remote_id
            FROM workspace_environment_variables
            WHERE workspace_id = ?1 AND environment_id = ?2 AND deleted_at IS NULL
            ORDER BY sort_order ASC, created_at ASC, id ASC
            "#,
        )
        .bind(workspace_id)
        .bind(environment_id)
        .fetch_all(self.db.pool())
        .await?;
        Ok(variables)
    }

    async fn assert_environment_name_unique(
        &self,
        workspace_id: &str,
        name: &str,
        exclude_id: Option<&str>,
    ) -> AppResult<()> {
        let existing: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT id
            FROM workspace_environments
            WHERE workspace_id = ?1
              AND name COLLATE NOCASE = ?2
              AND deleted_at IS NULL
              AND (?3 IS NULL OR id <> ?3)
            LIMIT 1
            "#,
        )
        .bind(workspace_id)
        .bind(name)
        .bind(exclude_id)
        .fetch_optional(self.db.pool())
        .await?;
        if existing.is_some() {
            return Err(AppError::Validation(format!(
                "environment name already exists in this workspace: {name}"
            )));
        }
        Ok(())
    }
}

async fn replace_workspace_variables(
    tx: &mut Transaction<'_, Sqlite>,
    workspace_id: &str,
    variables: Vec<WorkspaceVariableInput>,
) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE workspace_variables
        SET deleted_at = ?1, updated_at = ?1, revision = revision + 1,
            sync_status = 'deleted'
        WHERE workspace_id = ?2 AND deleted_at IS NULL
        "#,
    )
    .bind(&now)
    .bind(workspace_id)
    .execute(&mut **tx)
    .await?;

    for (index, variable) in variables.into_iter().enumerate() {
        let description = normalize_description(variable.description);
        let sort_order = i64::try_from(index).unwrap_or(i64::MAX);
        if let Some(id) = non_empty_optional(variable.id) {
            let updated = sqlx::query(
                r#"
                UPDATE workspace_variables
                SET key = ?1, value = ?2, is_secret = ?3, is_enabled = ?4,
                    description = ?5, sort_order = ?6, updated_at = ?7,
                    deleted_at = NULL, revision = revision + 1,
                    sync_status = 'pending'
                WHERE id = ?8 AND workspace_id = ?9
                "#,
            )
            .bind(variable.key.trim())
            .bind(variable.value)
            .bind(variable.is_secret)
            .bind(variable.is_enabled)
            .bind(description)
            .bind(sort_order)
            .bind(&now)
            .bind(id)
            .bind(workspace_id)
            .execute(&mut **tx)
            .await?;
            if updated.rows_affected() == 0 {
                return Err(AppError::NotFound("workspace variable".to_string()));
            }
        } else {
            sqlx::query(
                r#"
                INSERT INTO workspace_variables (
                  id, workspace_id, key, value, is_secret, is_enabled,
                  description, sort_order, created_at, updated_at,
                  revision, sync_status
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, 1, 'local')
                "#,
            )
            .bind(unfour_core::id::new_id())
            .bind(workspace_id)
            .bind(variable.key.trim())
            .bind(variable.value)
            .bind(variable.is_secret)
            .bind(variable.is_enabled)
            .bind(description)
            .bind(sort_order)
            .bind(&now)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

async fn replace_environment_variables(
    tx: &mut Transaction<'_, Sqlite>,
    workspace_id: &str,
    environment_id: &str,
    variables: Vec<WorkspaceVariableInput>,
) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE workspace_environment_variables
        SET deleted_at = ?1, updated_at = ?1, revision = revision + 1,
            sync_status = 'deleted'
        WHERE workspace_id = ?2 AND environment_id = ?3 AND deleted_at IS NULL
        "#,
    )
    .bind(&now)
    .bind(workspace_id)
    .bind(environment_id)
    .execute(&mut **tx)
    .await?;

    for (index, variable) in variables.into_iter().enumerate() {
        let description = normalize_description(variable.description);
        let sort_order = i64::try_from(index).unwrap_or(i64::MAX);
        if let Some(id) = non_empty_optional(variable.id) {
            let updated = sqlx::query(
                r#"
                UPDATE workspace_environment_variables
                SET key = ?1, value = ?2, is_secret = ?3, is_enabled = ?4,
                    description = ?5, sort_order = ?6, updated_at = ?7,
                    deleted_at = NULL, revision = revision + 1,
                    sync_status = 'pending'
                WHERE id = ?8 AND workspace_id = ?9 AND environment_id = ?10
                "#,
            )
            .bind(variable.key.trim())
            .bind(variable.value)
            .bind(variable.is_secret)
            .bind(variable.is_enabled)
            .bind(description)
            .bind(sort_order)
            .bind(&now)
            .bind(id)
            .bind(workspace_id)
            .bind(environment_id)
            .execute(&mut **tx)
            .await?;
            if updated.rows_affected() == 0 {
                return Err(AppError::NotFound(
                    "workspace environment variable".to_string(),
                ));
            }
        } else {
            sqlx::query(
                r#"
                INSERT INTO workspace_environment_variables (
                  id, workspace_id, environment_id, key, value, is_secret,
                  is_enabled, description, sort_order, created_at, updated_at,
                  revision, sync_status
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, 1, 'local')
                "#,
            )
            .bind(unfour_core::id::new_id())
            .bind(workspace_id)
            .bind(environment_id)
            .bind(variable.key.trim())
            .bind(variable.value)
            .bind(variable.is_secret)
            .bind(variable.is_enabled)
            .bind(description)
            .bind(sort_order)
            .bind(&now)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

fn validate_variables(variables: &[WorkspaceVariableInput]) -> AppResult<()> {
    let mut keys = HashSet::new();
    for variable in variables {
        let key = variable.key.trim();
        if key.is_empty() {
            return Err(AppError::Validation(
                "workspace variable key cannot be empty".to_string(),
            ));
        }
        if !key.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        }) {
            return Err(AppError::Validation(format!(
                "invalid workspace variable key: {}",
                variable.key
            )));
        }
        // Keys are case-insensitive so BASE_URL and base_url cannot both win
        // at resolve time.
        if !keys.insert(key.to_ascii_lowercase()) {
            return Err(AppError::Validation(format!(
                "duplicate workspace variable key: {key}"
            )));
        }
    }
    Ok(())
}

fn normalize_environment_name(name: String) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation(
            "environment name cannot be empty".to_string(),
        ));
    }
    if name.chars().count() > 80 {
        return Err(AppError::Validation(
            "environment name must be 80 characters or fewer".to_string(),
        ));
    }
    Ok(name.to_string())
}

fn normalize_description(description: Option<String>) -> Option<String> {
    description
        .map(|description| description.trim().to_string())
        .filter(|description| !description.is_empty())
}

fn non_empty_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn resolve_template(input: &str, values: &HashMap<String, String>) -> AppResult<String> {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;
    while let Some(start) = remaining.find("{{") {
        output.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];
        let Some(end) = after_start.find("}}") else {
            output.push_str(&remaining[start..]);
            return Ok(output);
        };
        let key = after_start[..end].trim();
        let value = values
            .get(key)
            .ok_or_else(|| AppError::Validation(format!("unresolved variable: {key}")))?;
        output.push_str(value);
        remaining = &after_start[end + 2..];
    }
    output.push_str(remaining);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::resolve_template;
    use std::collections::HashMap;

    #[test]
    fn resolver_reports_missing_variable() {
        let error = resolve_template("https://{{HOST}}/{{PATH}}", &HashMap::new())
            .expect_err("missing variable should fail");
        assert!(error.to_string().contains("unresolved variable: HOST"));
    }
}
