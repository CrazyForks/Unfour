use super::*;

impl ApiClientService {
    pub async fn list_environments(&self, workspace_id: String) -> AppResult<Vec<ApiEnvironment>> {
        validate_workspace_id(&workspace_id)?;
        let rows = sqlx::query_as::<_, EnvironmentRow>(
            r#"
            SELECT id, workspace_id, name, variables_json, is_active, created_at, updated_at
            FROM api_environments
            WHERE workspace_id = ?1 AND deleted_at IS NULL
            ORDER BY name COLLATE NOCASE
            "#,
        )
        .bind(workspace_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows.into_iter().map(ApiEnvironment::from).collect())
    }

    pub async fn create_environment(
        &self,
        workspace_id: String,
        name: String,
    ) -> AppResult<ApiEnvironment> {
        validate_workspace_id(&workspace_id)?;
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation(
                "environment name cannot be empty".to_string(),
            ));
        }
        self.ensure_environment_name_available(&workspace_id, &name, None)
            .await?;

        // The first environment in a workspace becomes the active one.
        let existing: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM api_environments WHERE workspace_id = ?1 AND deleted_at IS NULL",
        )
        .bind(&workspace_id)
        .fetch_one(self.db.pool())
        .await?;
        let is_active = existing == 0;
        let id = unfour_core::id::new_id();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO api_environments (
              id, workspace_id, name, variables_json, is_active, created_at, updated_at,
              revision, sync_status
            )
            VALUES (?1, ?2, ?3, '[]', ?4, ?5, ?5, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&workspace_id)
        .bind(&name)
        .bind(is_active)
        .bind(&now)
        .execute(self.db.pool())
        .await?;

        self.get_environment(&workspace_id, &id).await
    }

    pub async fn update_environment(
        &self,
        workspace_id: String,
        environment_id: String,
        name: String,
        variables: Vec<KeyValue>,
    ) -> AppResult<ApiEnvironment> {
        validate_workspace_id(&workspace_id)?;
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation(
                "environment name cannot be empty".to_string(),
            ));
        }
        self.get_environment(&workspace_id, &environment_id).await?;
        self.ensure_environment_name_available(&workspace_id, &name, Some(&environment_id))
            .await?;
        validate_environment(&variables)?;
        let now = Utc::now().to_rfc3339();
        let variables_json = serde_json::to_string(&variables)?;

        let result = sqlx::query(
            r#"
            UPDATE api_environments
            SET name = ?1, variables_json = ?2, updated_at = ?3,
                revision = revision + 1, sync_status = 'pending'
            WHERE workspace_id = ?4 AND id = ?5 AND deleted_at IS NULL
            "#,
        )
        .bind(&name)
        .bind(&variables_json)
        .bind(&now)
        .bind(&workspace_id)
        .bind(&environment_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("api environment".to_string()));
        }

        self.get_environment(&workspace_id, &environment_id).await
    }

    pub async fn delete_environment(
        &self,
        workspace_id: String,
        environment_id: String,
    ) -> AppResult<Vec<ApiEnvironment>> {
        validate_workspace_id(&workspace_id)?;
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE api_environments
            SET deleted_at = ?1, updated_at = ?1, is_active = 0,
                revision = revision + 1, sync_status = 'deleted'
            WHERE workspace_id = ?2 AND id = ?3 AND deleted_at IS NULL
            "#,
        )
        .bind(&now)
        .bind(&workspace_id)
        .bind(&environment_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("api environment".to_string()));
        }

        self.list_environments(workspace_id).await
    }

    /// Activate one environment (clearing any other active flag) for the
    /// workspace. Passing `None`/empty deactivates all of them ("No
    /// Environment").
    pub async fn activate_environment(
        &self,
        workspace_id: String,
        environment_id: Option<String>,
    ) -> AppResult<Vec<ApiEnvironment>> {
        validate_workspace_id(&workspace_id)?;
        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.pool().begin().await?;

        let target_id = environment_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty());

        if let Some(id) = target_id {
            // Deactivate other active environments first, then activate the
            // target. Order matters: the partial unique index
            // `uq_api_environments_active_per_workspace` enforces a single
            // active row per workspace at the statement level, so activating
            // the target while another row is still active would raise
            // SQLITE_CONSTRAINT before the second statement could run.
            sqlx::query(
                r#"
                UPDATE api_environments
                SET is_active = 0, updated_at = ?1,
                    revision = revision + 1, sync_status = 'pending'
                WHERE workspace_id = ?2 AND id != ?3 AND deleted_at IS NULL AND is_active = 1
                "#,
            )
            .bind(&now)
            .bind(&workspace_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            let result = sqlx::query(
                r#"
                UPDATE api_environments
                SET is_active = 1, updated_at = ?1,
                    revision = revision + 1, sync_status = 'pending'
                WHERE workspace_id = ?2 AND id = ?3 AND deleted_at IS NULL
                "#,
            )
            .bind(&now)
            .bind(&workspace_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
            if result.rows_affected() == 0 {
                // tx is dropped without commit -> rolled back.
                return Err(AppError::NotFound("api environment".to_string()));
            }
        } else {
            sqlx::query(
                r#"
                UPDATE api_environments
                SET is_active = 0, updated_at = ?1,
                    revision = revision + 1, sync_status = 'pending'
                WHERE workspace_id = ?2 AND deleted_at IS NULL AND is_active = 1
                "#,
            )
            .bind(&now)
            .bind(&workspace_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        self.list_environments(workspace_id).await
    }

    pub(super) async fn get_environment(
        &self,
        workspace_id: &str,
        environment_id: &str,
    ) -> AppResult<ApiEnvironment> {
        let row = sqlx::query_as::<_, EnvironmentRow>(
            r#"
            SELECT id, workspace_id, name, variables_json, is_active, created_at, updated_at
            FROM api_environments
            WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .bind(environment_id)
        .fetch_optional(self.db.pool())
        .await?;

        row.map(ApiEnvironment::from)
            .ok_or_else(|| AppError::NotFound("api environment".to_string()))
    }

    pub(super) async fn ensure_environment_name_available(
        &self,
        workspace_id: &str,
        name: &str,
        exclude_id: Option<&str>,
    ) -> AppResult<()> {
        let existing: Option<(String,)> = match exclude_id {
            Some(environment_id) => {
                sqlx::query_as(
                    r#"
                    SELECT id
                    FROM api_environments
                    WHERE workspace_id = ?1
                      AND id != ?2
                      AND name COLLATE NOCASE = ?3
                      AND deleted_at IS NULL
                    LIMIT 1
                    "#,
                )
                .bind(workspace_id)
                .bind(environment_id)
                .bind(name)
                .fetch_optional(self.db.pool())
                .await?
            }
            None => {
                sqlx::query_as(
                    r#"
                    SELECT id
                    FROM api_environments
                    WHERE workspace_id = ?1
                      AND name COLLATE NOCASE = ?2
                      AND deleted_at IS NULL
                    LIMIT 1
                    "#,
                )
                .bind(workspace_id)
                .bind(name)
                .fetch_optional(self.db.pool())
                .await?
            }
        };

        if existing.is_some() {
            return Err(AppError::Validation(format!(
                "environment name already exists in this workspace: {name}"
            )));
        }

        Ok(())
    }

    pub(super) async fn active_environment_variables(
        &self,
        workspace_id: &str,
    ) -> AppResult<Vec<KeyValue>> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT variables_json
            FROM api_environments
            WHERE workspace_id = ?1 AND is_active = 1 AND deleted_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(workspace_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(row
            .map(|(json,)| serde_json::from_str::<Vec<KeyValue>>(&json).unwrap_or_default())
            .unwrap_or_default())
    }
}
