use super::*;

impl ApiClientService {
    pub async fn list_collections(&self, workspace_id: String) -> AppResult<Vec<ApiCollection>> {
        validate_workspace_id(&workspace_id)?;
        let rows = sqlx::query_as::<_, CollectionRow>(
            r#"
            SELECT id, workspace_id, name, description, created_at, updated_at
            FROM api_collections
            WHERE workspace_id = ?1 AND deleted_at IS NULL
            ORDER BY name COLLATE NOCASE
            "#,
        )
        .bind(workspace_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows.into_iter().map(ApiCollection::from).collect())
    }

    pub async fn create_collection(
        &self,
        workspace_id: String,
        name: String,
    ) -> AppResult<ApiCollection> {
        validate_workspace_id(&workspace_id)?;
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation(
                "collection name cannot be empty".to_string(),
            ));
        }
        let id = unfour_core::id::new_id();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO api_collections (
              id, workspace_id, name, created_at, updated_at, revision, sync_status
            )
            VALUES (?1, ?2, ?3, ?4, ?4, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&workspace_id)
        .bind(&name)
        .bind(&now)
        .execute(self.db.pool())
        .await?;

        self.get_collection(&workspace_id, &id).await
    }

    pub async fn rename_collection(
        &self,
        workspace_id: String,
        collection_id: String,
        name: String,
    ) -> AppResult<ApiCollection> {
        validate_workspace_id(&workspace_id)?;
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation(
                "collection name cannot be empty".to_string(),
            ));
        }
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE api_collections
            SET name = ?1, updated_at = ?2, revision = revision + 1, sync_status = 'pending'
            WHERE workspace_id = ?3 AND id = ?4 AND deleted_at IS NULL
            "#,
        )
        .bind(&name)
        .bind(&now)
        .bind(&workspace_id)
        .bind(&collection_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("api collection".to_string()));
        }

        self.get_collection(&workspace_id, &collection_id).await
    }

    pub async fn list_collection_folders(
        &self,
        workspace_id: String,
        collection_id: Option<String>,
    ) -> AppResult<Vec<ApiCollectionFolder>> {
        validate_workspace_id(&workspace_id)?;
        let collection_id = normalize_entity_id(collection_id);
        let rows = match collection_id {
            Some(collection_id) => {
                self.get_collection(&workspace_id, &collection_id).await?;
                sqlx::query_as::<_, ApiCollectionFolder>(
                    r#"
                    SELECT id, workspace_id, collection_id, parent_folder_id, name,
                           sort_order, created_at, updated_at, deleted_at,
                           revision, sync_status, remote_id
                    FROM api_collection_folders
                    WHERE workspace_id = ?1 AND collection_id = ?2 AND deleted_at IS NULL
                    ORDER BY COALESCE(parent_folder_id, ''), sort_order, name COLLATE NOCASE
                    "#,
                )
                .bind(&workspace_id)
                .bind(collection_id)
                .fetch_all(self.db.pool())
                .await?
            }
            None => {
                sqlx::query_as::<_, ApiCollectionFolder>(
                    r#"
                    SELECT id, workspace_id, collection_id, parent_folder_id, name,
                           sort_order, created_at, updated_at, deleted_at,
                           revision, sync_status, remote_id
                    FROM api_collection_folders
                    WHERE workspace_id = ?1 AND deleted_at IS NULL
                    ORDER BY collection_id, COALESCE(parent_folder_id, ''), sort_order, name COLLATE NOCASE
                    "#,
                )
                .bind(&workspace_id)
                .fetch_all(self.db.pool())
                .await?
            }
        };

        Ok(rows)
    }

    pub async fn create_collection_folder(
        &self,
        workspace_id: String,
        collection_id: String,
        parent_folder_id: Option<String>,
        name: String,
    ) -> AppResult<ApiCollectionFolder> {
        validate_workspace_id(&workspace_id)?;
        let name = normalize_folder_name(name)?;
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
                    "parent folder must belong to the target collection".to_string(),
                ));
            }
        }
        let now = Utc::now().to_rfc3339();
        let id = unfour_core::id::new_id();
        let sort_order = self
            .next_folder_sort_order_tx(
                &mut tx,
                &workspace_id,
                &collection_id,
                parent_folder_id.as_deref(),
            )
            .await?;

        sqlx::query(
            r#"
            INSERT INTO api_collection_folders (
              id, workspace_id, collection_id, parent_folder_id, name, sort_order,
              created_at, updated_at, revision, sync_status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&workspace_id)
        .bind(&collection_id)
        .bind(&parent_folder_id)
        .bind(&name)
        .bind(sort_order)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        self.get_collection_folder(&workspace_id, &id).await
    }

    pub async fn rename_collection_folder(
        &self,
        workspace_id: String,
        folder_id: String,
        name: String,
    ) -> AppResult<ApiCollectionFolder> {
        validate_workspace_id(&workspace_id)?;
        let name = normalize_folder_name(name)?;
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"
            UPDATE api_collection_folders
            SET name = ?1, updated_at = ?2,
                revision = revision + 1, sync_status = 'pending'
            WHERE workspace_id = ?3 AND id = ?4 AND deleted_at IS NULL
            "#,
        )
        .bind(&name)
        .bind(&now)
        .bind(&workspace_id)
        .bind(&folder_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("api collection folder".to_string()));
        }

        self.get_collection_folder(&workspace_id, &folder_id).await
    }

    pub async fn delete_collection_folder(
        &self,
        workspace_id: String,
        folder_id: String,
    ) -> AppResult<Vec<ApiCollectionFolder>> {
        validate_workspace_id(&workspace_id)?;
        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.pool().begin().await?;
        let folder = self
            .get_collection_folder_tx(&mut tx, &workspace_id, &folder_id)
            .await?;

        sqlx::query(
            r#"
            WITH RECURSIVE folder_tree(id) AS (
              SELECT id
              FROM api_collection_folders
              WHERE workspace_id = ?2 AND id = ?3 AND deleted_at IS NULL
              UNION ALL
              SELECT child.id
              FROM api_collection_folders child
              JOIN folder_tree parent ON child.parent_folder_id = parent.id
              WHERE child.workspace_id = ?2 AND child.deleted_at IS NULL
            )
            UPDATE api_collection_folders
            SET deleted_at = ?1, updated_at = ?1,
                revision = revision + 1, sync_status = 'deleted'
            WHERE id IN (SELECT id FROM folder_tree)
            "#,
        )
        .bind(&now)
        .bind(&workspace_id)
        .bind(&folder_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            WITH RECURSIVE folder_tree(id) AS (
              SELECT id
              FROM api_collection_folders
              WHERE workspace_id = ?2 AND id = ?3
              UNION ALL
              SELECT child.id
              FROM api_collection_folders child
              JOIN folder_tree parent ON child.parent_folder_id = parent.id
              WHERE child.workspace_id = ?2
            )
            UPDATE api_requests
            SET deleted_at = ?1, updated_at = ?1, revision = revision + 1, sync_status = 'deleted'
            WHERE workspace_id = ?2
              AND parent_folder_id IN (SELECT id FROM folder_tree)
              AND deleted_at IS NULL
            "#,
        )
        .bind(&now)
        .bind(&workspace_id)
        .bind(&folder_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        self.list_collection_folders(workspace_id, Some(folder.collection_id))
            .await
    }

    pub async fn move_collection_folder(
        &self,
        workspace_id: String,
        folder_id: String,
        target_parent_folder_id: Option<String>,
    ) -> AppResult<ApiCollectionFolder> {
        validate_workspace_id(&workspace_id)?;
        let target_parent_folder_id = normalize_entity_id(target_parent_folder_id);
        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.pool().begin().await?;
        let folder = self
            .get_collection_folder_tx(&mut tx, &workspace_id, &folder_id)
            .await?;

        if target_parent_folder_id.as_deref() == Some(folder.id.as_str()) {
            return Err(AppError::Validation(
                "moving folder would create a cycle".to_string(),
            ));
        }
        if let Some(parent_id) = &target_parent_folder_id {
            let parent = self
                .get_collection_folder_tx(&mut tx, &workspace_id, parent_id)
                .await?;
            if parent.collection_id != folder.collection_id {
                return Err(AppError::Validation(
                    "target parent folder must belong to the same collection".to_string(),
                ));
            }
            if self
                .folder_contains_descendant_tx(&mut tx, &workspace_id, &folder.id, parent_id)
                .await?
            {
                return Err(AppError::Validation(
                    "moving folder would create a cycle".to_string(),
                ));
            }
        }

        let sort_order = self
            .next_folder_sort_order_tx(
                &mut tx,
                &workspace_id,
                &folder.collection_id,
                target_parent_folder_id.as_deref(),
            )
            .await?;

        let result = sqlx::query(
            r#"
            UPDATE api_collection_folders
            SET parent_folder_id = ?1, sort_order = ?2, updated_at = ?3,
                revision = revision + 1, sync_status = 'pending'
            WHERE workspace_id = ?4 AND id = ?5 AND deleted_at IS NULL
            "#,
        )
        .bind(&target_parent_folder_id)
        .bind(sort_order)
        .bind(&now)
        .bind(&workspace_id)
        .bind(&folder_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("api collection folder".to_string()));
        }

        tx.commit().await?;
        self.get_collection_folder(&workspace_id, &folder_id).await
    }

    pub async fn reorder_collection_folders(
        &self,
        workspace_id: String,
        collection_id: String,
        parent_folder_id: Option<String>,
        folder_ids: Vec<String>,
    ) -> AppResult<Vec<ApiCollectionFolder>> {
        validate_workspace_id(&workspace_id)?;
        let parent_folder_id = normalize_entity_id(parent_folder_id);
        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.pool().begin().await?;
        self.ensure_collection_exists_tx(&mut tx, &workspace_id, &collection_id)
            .await?;
        for (index, folder_id) in folder_ids.iter().enumerate() {
            let folder = self
                .get_collection_folder_tx(&mut tx, &workspace_id, folder_id)
                .await?;
            if folder.collection_id != collection_id || folder.parent_folder_id != parent_folder_id
            {
                return Err(AppError::Validation(
                    "folder reorder ids must be siblings in the target collection".to_string(),
                ));
            }
            sqlx::query(
                r#"
                UPDATE api_collection_folders
                SET sort_order = ?1, updated_at = ?2,
                    revision = revision + 1, sync_status = 'pending'
                WHERE workspace_id = ?3 AND id = ?4 AND deleted_at IS NULL
                "#,
            )
            .bind(i64::try_from(index).unwrap_or(i64::MAX))
            .bind(&now)
            .bind(&workspace_id)
            .bind(folder_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.list_collection_folders(workspace_id, Some(collection_id))
            .await
    }

    pub(super) async fn get_collection_folder(
        &self,
        workspace_id: &str,
        folder_id: &str,
    ) -> AppResult<ApiCollectionFolder> {
        let row = sqlx::query_as::<_, ApiCollectionFolder>(
            r#"
            SELECT id, workspace_id, collection_id, parent_folder_id, name,
                   sort_order, created_at, updated_at, deleted_at,
                   revision, sync_status, remote_id
            FROM api_collection_folders
            WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .bind(folder_id)
        .fetch_optional(self.db.pool())
        .await?;

        row.ok_or_else(|| AppError::NotFound("api collection folder".to_string()))
    }

    pub(super) async fn get_collection_folder_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        workspace_id: &str,
        folder_id: &str,
    ) -> AppResult<ApiCollectionFolder> {
        let row = sqlx::query_as::<_, ApiCollectionFolder>(
            r#"
            SELECT id, workspace_id, collection_id, parent_folder_id, name,
                   sort_order, created_at, updated_at, deleted_at,
                   revision, sync_status, remote_id
            FROM api_collection_folders
            WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .bind(folder_id)
        .fetch_optional(&mut **tx)
        .await?;

        row.ok_or_else(|| AppError::NotFound("api collection folder".to_string()))
    }

    pub(super) async fn ensure_collection_exists_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        workspace_id: &str,
        collection_id: &str,
    ) -> AppResult<()> {
        let exists: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT id
            FROM api_collections
            WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .bind(collection_id)
        .fetch_optional(&mut **tx)
        .await?;

        if exists.is_none() {
            return Err(AppError::NotFound("api collection".to_string()));
        }
        Ok(())
    }

    pub(super) async fn first_or_create_collection_id_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        workspace_id: &str,
        now: &str,
    ) -> AppResult<String> {
        let existing: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT id
            FROM api_collections
            WHERE workspace_id = ?1 AND deleted_at IS NULL
            ORDER BY name COLLATE NOCASE
            LIMIT 1
            "#,
        )
        .bind(workspace_id)
        .fetch_optional(&mut **tx)
        .await?;
        if let Some((id,)) = existing {
            return Ok(id);
        }

        let id = unfour_core::id::new_id();
        sqlx::query(
            r#"
            INSERT INTO api_collections (
              id, workspace_id, name, created_at, updated_at, revision, sync_status
            )
            VALUES (?1, ?2, ?3, ?4, ?4, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(workspace_id)
        .bind(DEFAULT_COLLECTION_NAME)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        Ok(id)
    }

    pub(super) async fn next_folder_sort_order_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        workspace_id: &str,
        collection_id: &str,
        parent_folder_id: Option<&str>,
    ) -> AppResult<i64> {
        let max_order: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT MAX(sort_order)
            FROM api_collection_folders
            WHERE workspace_id = ?1
              AND collection_id = ?2
              AND parent_folder_id IS ?3
              AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .bind(collection_id)
        .bind(parent_folder_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(max_order.unwrap_or(-1) + 1)
    }

    pub(super) async fn next_request_sort_order_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        workspace_id: &str,
        collection_id: &str,
        parent_folder_id: Option<&str>,
    ) -> AppResult<i64> {
        let max_order: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT MAX(sort_order)
            FROM api_requests
            WHERE workspace_id = ?1
              AND collection_id = ?2
              AND parent_folder_id IS ?3
              AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .bind(collection_id)
        .bind(parent_folder_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(max_order.unwrap_or(-1) + 1)
    }

    pub(super) async fn folder_contains_descendant_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        workspace_id: &str,
        folder_id: &str,
        candidate_descendant_id: &str,
    ) -> AppResult<bool> {
        let found: Option<(String,)> = sqlx::query_as(
            r#"
            WITH RECURSIVE folder_tree(id) AS (
              SELECT id
              FROM api_collection_folders
              WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL
              UNION ALL
              SELECT child.id
              FROM api_collection_folders child
              JOIN folder_tree parent ON child.parent_folder_id = parent.id
              WHERE child.workspace_id = ?1 AND child.deleted_at IS NULL
            )
            SELECT id FROM folder_tree WHERE id = ?3 LIMIT 1
            "#,
        )
        .bind(workspace_id)
        .bind(folder_id)
        .bind(candidate_descendant_id)
        .fetch_optional(&mut **tx)
        .await?;

        Ok(found.is_some())
    }

    /// Soft-delete a collection and cascade soft-delete its saved requests in a
    /// single transaction.
    pub async fn delete_collection(
        &self,
        workspace_id: String,
        collection_id: String,
    ) -> AppResult<Vec<ApiCollection>> {
        validate_workspace_id(&workspace_id)?;
        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.pool().begin().await?;

        let result = sqlx::query(
            r#"
            UPDATE api_collections
            SET deleted_at = ?1, updated_at = ?1, revision = revision + 1,
                sync_status = 'deleted'
            WHERE workspace_id = ?2 AND id = ?3 AND deleted_at IS NULL
            "#,
        )
        .bind(&now)
        .bind(&workspace_id)
        .bind(&collection_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            // tx is dropped without commit -> rolled back.
            return Err(AppError::NotFound("api collection".to_string()));
        }

        sqlx::query(
            r#"
            UPDATE api_collection_folders
            SET deleted_at = ?1, updated_at = ?1,
                revision = revision + 1, sync_status = 'deleted'
            WHERE workspace_id = ?2 AND collection_id = ?3 AND deleted_at IS NULL
            "#,
        )
        .bind(&now)
        .bind(&workspace_id)
        .bind(&collection_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE api_requests
            SET deleted_at = ?1, updated_at = ?1, revision = revision + 1, sync_status = 'deleted'
            WHERE workspace_id = ?2 AND collection_id = ?3 AND deleted_at IS NULL
            "#,
        )
        .bind(&now)
        .bind(&workspace_id)
        .bind(&collection_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        self.list_collections(workspace_id).await
    }

    /// Reassign a saved request to a different collection and/or folder.
    pub(super) async fn get_collection(
        &self,
        workspace_id: &str,
        collection_id: &str,
    ) -> AppResult<ApiCollection> {
        let row = sqlx::query_as::<_, CollectionRow>(
            r#"
            SELECT id, workspace_id, name, description, created_at, updated_at
            FROM api_collections
            WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL
            "#,
        )
        .bind(workspace_id)
        .bind(collection_id)
        .fetch_optional(self.db.pool())
        .await?;

        row.map(ApiCollection::from)
            .ok_or_else(|| AppError::NotFound("api collection".to_string()))
    }
}
