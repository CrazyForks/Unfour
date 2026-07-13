use super::*;

impl ApiClientService {
    pub async fn save_request(&self, input: ApiRequestInput) -> AppResult<ApiSavedRequest> {
        validate_workspace_id(&input.workspace_id)?;
        let now = Utc::now().to_rfc3339();
        let id = unfour_core::id::new_id();
        let mut tx = self.db.pool().begin().await?;
        let (name, collection_id, parent_folder_id, sort_order) =
            self.saved_request_fields(&mut tx, &input, &now).await?;

        sqlx::query(
            r#"
            INSERT INTO api_requests (
              id, workspace_id, name, collection_id, parent_folder_id, sort_order,
              auth_json, method, url, headers_json, query_json, body, body_kind,
              created_at, updated_at, revision, sync_status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&input.workspace_id)
        .bind(name)
        .bind(collection_id)
        .bind(parent_folder_id)
        .bind(sort_order)
        .bind(input.auth_json.as_deref().unwrap_or(DEFAULT_AUTH_JSON))
        .bind(input.method.to_uppercase())
        .bind(input.url)
        .bind(serde_json::to_string(&input.headers)?)
        .bind(serde_json::to_string(&input.query)?)
        .bind(input.body.clone())
        .bind(input.body_kind)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        self.get_saved_request(&id).await
    }

    pub async fn update_request(
        &self,
        workspace_id: String,
        request_id: String,
        input: ApiRequestInput,
    ) -> AppResult<ApiSavedRequest> {
        validate_workspace_id(&workspace_id)?;
        validate_workspace_id(&input.workspace_id)?;
        if workspace_id != input.workspace_id {
            return Err(AppError::Validation(
                "api request workspace mismatch".to_string(),
            ));
        }
        if request_id.trim().is_empty() {
            return Err(AppError::Validation(
                "api request id cannot be empty".to_string(),
            ));
        }

        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.pool().begin().await?;
        let (name, collection_id, parent_folder_id, _sort_order) =
            self.saved_request_fields(&mut tx, &input, &now).await?;

        let result = sqlx::query(
            r#"
            UPDATE api_requests
            SET name = ?1, collection_id = ?2, parent_folder_id = ?3, auth_json = ?4,
                method = ?5, url = ?6, headers_json = ?7, query_json = ?8,
                body = ?9, body_kind = ?10, updated_at = ?11,
                revision = revision + 1, sync_status = 'pending'
            WHERE workspace_id = ?12 AND id = ?13 AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .bind(collection_id)
        .bind(parent_folder_id)
        .bind(input.auth_json.as_deref().unwrap_or(DEFAULT_AUTH_JSON))
        .bind(input.method.to_uppercase())
        .bind(input.url)
        .bind(serde_json::to_string(&input.headers)?)
        .bind(serde_json::to_string(&input.query)?)
        .bind(input.body.clone())
        .bind(input.body_kind)
        .bind(now)
        .bind(&workspace_id)
        .bind(&request_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("api request".to_string()));
        }

        tx.commit().await?;
        self.get_saved_request_for_workspace(&workspace_id, &request_id)
            .await
    }

    pub async fn list_saved_requests(
        &self,
        workspace_id: String,
    ) -> AppResult<Vec<ApiSavedRequest>> {
        validate_workspace_id(&workspace_id)?;

        let items = sqlx::query_as::<_, ApiSavedRequest>(
            r#"
            SELECT
              id, workspace_id, name, collection_id, parent_folder_id, sort_order,
              auth_json, method, url, headers_json, query_json, body, body_kind,
              created_at, updated_at, deleted_at, revision, sync_status, remote_id
            FROM api_requests
            WHERE workspace_id = ?1 AND deleted_at IS NULL
            ORDER BY collection_id, COALESCE(parent_folder_id, ''), sort_order, updated_at DESC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(items)
    }

    pub async fn duplicate_request(
        &self,
        workspace_id: String,
        request_id: String,
    ) -> AppResult<ApiSavedRequest> {
        validate_workspace_id(&workspace_id)?;
        if request_id.trim().is_empty() {
            return Err(AppError::Validation(
                "api request id cannot be empty".to_string(),
            ));
        }

        let source = self
            .get_saved_request_for_workspace(&workspace_id, &request_id)
            .await?;
        let now = Utc::now().to_rfc3339();
        let id = unfour_core::id::new_id();
        let name = format!("{} Copy", source.name);

        sqlx::query(
            r#"
            INSERT INTO api_requests (
              id, workspace_id, name, collection_id, parent_folder_id, sort_order,
              auth_json, method, url, headers_json, query_json, body, body_kind,
              created_at, updated_at, revision, sync_status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&workspace_id)
        .bind(name)
        .bind(source.collection_id)
        .bind(source.parent_folder_id)
        .bind(source.sort_order)
        .bind(source.auth_json)
        .bind(source.method)
        .bind(source.url)
        .bind(source.headers_json)
        .bind(source.query_json)
        .bind(source.body)
        .bind(source.body_kind)
        .bind(now)
        .execute(self.db.pool())
        .await?;

        self.get_saved_request_for_workspace(&workspace_id, &id)
            .await
    }

    pub async fn delete_request(
        &self,
        workspace_id: String,
        request_id: String,
    ) -> AppResult<Vec<ApiSavedRequest>> {
        validate_workspace_id(&workspace_id)?;
        if request_id.trim().is_empty() {
            return Err(AppError::Validation(
                "api request id cannot be empty".to_string(),
            ));
        }

        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"
            UPDATE api_requests
            SET deleted_at = ?1, updated_at = ?1, revision = revision + 1, sync_status = 'deleted'
            WHERE workspace_id = ?2 AND id = ?3 AND deleted_at IS NULL
            "#,
        )
        .bind(now)
        .bind(&workspace_id)
        .bind(&request_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("api request".to_string()));
        }

        self.list_saved_requests(workspace_id).await
    }

    pub async fn move_request(
        &self,
        workspace_id: String,
        request_id: String,
        collection_id: Option<String>,
        parent_folder_id: Option<String>,
    ) -> AppResult<ApiSavedRequest> {
        validate_workspace_id(&workspace_id)?;
        if request_id.trim().is_empty() {
            return Err(AppError::Validation(
                "api request id cannot be empty".to_string(),
            ));
        }
        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.pool().begin().await?;
        let (collection_id, parent_folder_id) = self
            .resolve_request_location_tx(
                &mut tx,
                &workspace_id,
                collection_id,
                parent_folder_id,
                &now,
            )
            .await?;
        let sort_order = self
            .next_request_sort_order_tx(
                &mut tx,
                &workspace_id,
                &collection_id,
                parent_folder_id.as_deref(),
            )
            .await?;

        let result = sqlx::query(
            r#"
            UPDATE api_requests
            SET collection_id = ?1, parent_folder_id = ?2, sort_order = ?3, updated_at = ?4,
                revision = revision + 1, sync_status = 'pending'
            WHERE workspace_id = ?5 AND id = ?6 AND deleted_at IS NULL
            "#,
        )
        .bind(&collection_id)
        .bind(&parent_folder_id)
        .bind(sort_order)
        .bind(&now)
        .bind(&workspace_id)
        .bind(&request_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("api request".to_string()));
        }

        tx.commit().await?;
        self.get_saved_request_for_workspace(&workspace_id, &request_id)
            .await
    }

    pub async fn reorder_requests(
        &self,
        workspace_id: String,
        collection_id: String,
        parent_folder_id: Option<String>,
        request_ids: Vec<String>,
    ) -> AppResult<Vec<ApiSavedRequest>> {
        validate_workspace_id(&workspace_id)?;
        let parent_folder_id = normalize_entity_id(parent_folder_id);
        let mut tx = self.db.pool().begin().await?;
        self.ensure_collection_exists_tx(&mut tx, &workspace_id, &collection_id)
            .await?;
        if let Some(parent_id) = &parent_folder_id {
            let parent = self
                .get_collection_folder_tx(&mut tx, &workspace_id, parent_id)
                .await?;
            if parent.collection_id != collection_id {
                return Err(AppError::Validation(
                    "request reorder parent must belong to the target collection".to_string(),
                ));
            }
        }

        for (index, request_id) in request_ids.iter().enumerate() {
            let request = self
                .get_saved_request_for_workspace_tx(&mut tx, &workspace_id, request_id)
                .await?;
            if request.collection_id != collection_id
                || request.parent_folder_id != parent_folder_id
            {
                return Err(AppError::Validation(
                    "request reorder ids must be siblings in the target collection".to_string(),
                ));
            }
            sqlx::query(
                r#"
                UPDATE api_requests
                SET sort_order = ?1, revision = revision + 1, sync_status = 'pending'
                WHERE workspace_id = ?2 AND id = ?3 AND deleted_at IS NULL
                "#,
            )
            .bind(i64::try_from(index).unwrap_or(i64::MAX))
            .bind(&workspace_id)
            .bind(request_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.list_saved_requests(workspace_id).await
    }

    pub(super) async fn saved_request_fields(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        input: &ApiRequestInput,
        now: &str,
    ) -> AppResult<(String, String, Option<String>, i64)> {
        let (collection_id, parent_folder_id) = self
            .resolve_request_location_tx(
                tx,
                &input.workspace_id,
                input.collection_id.clone(),
                input.parent_folder_id.clone(),
                now,
            )
            .await?;
        let sort_order = self
            .next_request_sort_order_tx(
                tx,
                &input.workspace_id,
                &collection_id,
                parent_folder_id.as_deref(),
            )
            .await?;
        let name = input
            .name
            .clone()
            .unwrap_or_else(|| format!("{} {}", input.method.to_uppercase(), input.url));

        Ok((name, collection_id, parent_folder_id, sort_order))
    }

    pub(super) async fn resolve_request_location_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        workspace_id: &str,
        collection_id: Option<String>,
        parent_folder_id: Option<String>,
        now: &str,
    ) -> AppResult<(String, Option<String>)> {
        let parent_folder_id = normalize_entity_id(parent_folder_id);
        let collection_id = normalize_collection_id(collection_id);

        if let Some(parent_id) = &parent_folder_id {
            let parent = self
                .get_collection_folder_tx(tx, workspace_id, parent_id)
                .await?;
            if let Some(collection_id) = collection_id {
                if collection_id != parent.collection_id {
                    return Err(AppError::Validation(
                        "parent folder must belong to the target collection".to_string(),
                    ));
                }
                return Ok((collection_id, Some(parent_id.clone())));
            }
            return Ok((parent.collection_id, Some(parent_id.clone())));
        }

        let collection_id = match collection_id {
            Some(collection_id) => {
                self.ensure_collection_exists_tx(tx, workspace_id, &collection_id)
                    .await?;
                collection_id
            }
            None => {
                self.first_or_create_collection_id_tx(tx, workspace_id, now)
                    .await?
            }
        };

        Ok((collection_id, None))
    }

    pub async fn get_saved_request(&self, id: &str) -> AppResult<ApiSavedRequest> {
        let saved = sqlx::query_as::<_, ApiSavedRequest>(
            r#"
            SELECT
              id, workspace_id, name, collection_id, parent_folder_id, sort_order,
              auth_json, method, url, headers_json, query_json, body, body_kind,
              created_at, updated_at, deleted_at, revision, sync_status, remote_id
            FROM api_requests
            WHERE id = ?1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(self.db.pool())
        .await?;

        saved.ok_or_else(|| AppError::NotFound("api request".to_string()))
    }

    pub(super) async fn get_saved_request_for_workspace(
        &self,
        workspace_id: &str,
        id: &str,
    ) -> AppResult<ApiSavedRequest> {
        let saved = sqlx::query_as::<_, ApiSavedRequest>(
            r#"
            SELECT
              id, workspace_id, name, collection_id, parent_folder_id, sort_order,
              auth_json, method, url, headers_json, query_json, body, body_kind,
              created_at, updated_at, deleted_at, revision, sync_status, remote_id
            FROM api_requests
            WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .bind(id)
        .fetch_optional(self.db.pool())
        .await?;

        saved.ok_or_else(|| AppError::NotFound("api request".to_string()))
    }

    pub(super) async fn get_saved_request_for_workspace_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        workspace_id: &str,
        id: &str,
    ) -> AppResult<ApiSavedRequest> {
        let saved = sqlx::query_as::<_, ApiSavedRequest>(
            r#"
            SELECT
              id, workspace_id, name, collection_id, parent_folder_id, sort_order,
              auth_json, method, url, headers_json, query_json, body, body_kind,
              created_at, updated_at, deleted_at, revision, sync_status, remote_id
            FROM api_requests
            WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?;

        saved.ok_or_else(|| AppError::NotFound("api request".to_string()))
    }
}
