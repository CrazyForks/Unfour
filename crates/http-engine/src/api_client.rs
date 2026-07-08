use chrono::Utc;
use reqwest::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::{Client, Method};
use sqlx::{Sqlite, Transaction};
use std::time::{Duration, Instant};
use unfour_core::models::{
    ApiCollection, ApiCollectionFolder, ApiEnvironment, ApiHistoryDetail, ApiHistoryItem,
    ApiRequestInput, ApiResponse, ApiSavedRequest, KeyValue,
};
use unfour_core::{AppError, AppResult};
use unfour_local_storage::LocalDb;

#[path = "helpers.rs"]
mod helpers;
use helpers::{
    build_url, normalize_collection_id, normalize_entity_id, normalize_folder_name, parse_method,
    resolve_input, validate_environment, validate_workspace_id, CollectionRow, EnvironmentRow,
};

const DEFAULT_AUTH_JSON: &str = r#"{"type":"none"}"#;
const DEFAULT_COLLECTION_NAME: &str = "My Collection";

#[derive(Clone)]
pub struct ApiClientService {
    client: Client,
    db: LocalDb,
}

impl ApiClientService {
    pub fn new(db: LocalDb) -> Self {
        Self {
            client: Client::new(),
            db,
        }
    }

    pub async fn send(&self, input: ApiRequestInput) -> AppResult<ApiResponse> {
        validate_workspace_id(&input.workspace_id)?;
        let method = parse_method(&input.method)?;
        let environment = self
            .active_environment_variables(&input.workspace_id)
            .await?;
        let resolved = resolve_input(input.clone(), &environment)?;
        let url = build_url(&resolved.url, &resolved.query)?;
        let timeout =
            Duration::from_millis(input.timeout_ms.unwrap_or(60_000).clamp(1_000, 300_000));
        let request_id = unfour_diag::new_request_id();
        let request_fields = serde_json::json!({
            "request_id": request_id.as_str(),
            "method": method.as_str(),
            "host": url.host_str().unwrap_or(""),
            "path": url.path(),
        });

        let mut builder = self
            .client
            .request(method.clone(), url.clone())
            .timeout(timeout);
        let mut has_content_type = false;

        for header in resolved.headers.iter().filter(|item| item.enabled) {
            if header.key.trim().eq_ignore_ascii_case("content-type") {
                has_content_type = true;
            }
            let name = HeaderName::from_bytes(header.key.trim().as_bytes()).map_err(|_| {
                AppError::Validation(format!("invalid header name: {}", header.key))
            })?;
            let value = HeaderValue::from_str(&header.value).map_err(|_| {
                AppError::Validation(format!("invalid header value for {}", header.key))
            })?;
            builder = builder.header(name, value);
        }

        if let Some(body) = resolved.body.clone().filter(|body| !body.is_empty()) {
            if input.body_kind == "json" && !has_content_type {
                builder = builder.header(CONTENT_TYPE, "application/json");
            }
            if !matches!(method, Method::GET | Method::HEAD) {
                builder = builder.body(body);
            }
        }

        let started = Instant::now();
        unfour_diag::log_operation_event(
            "api_request_started",
            "api_client",
            "send",
            "started",
            None,
            None,
            request_fields.clone(),
        );
        let response = match builder.send().await {
            Ok(response) => response,
            Err(error) => {
                unfour_diag::log_operation_event(
                    "api_request_failed",
                    "api_client",
                    "send",
                    "error",
                    Some(started.elapsed().as_millis()),
                    Some("HTTP_ERROR"),
                    request_fields,
                );
                return Err(error.into());
            }
        };
        let duration_ms = started.elapsed().as_millis();
        let status = response.status();
        let response_headers = response
            .headers()
            .iter()
            .map(|(key, value)| KeyValue {
                key: key.to_string(),
                value: value.to_str().unwrap_or("<binary>").to_string(),
                enabled: true,
            })
            .collect::<Vec<_>>();
        let body = match response.text().await {
            Ok(body) => body,
            Err(error) => {
                unfour_diag::log_operation_event(
                    "api_request_failed",
                    "api_client",
                    "send",
                    "error",
                    Some(started.elapsed().as_millis()),
                    Some("HTTP_ERROR"),
                    request_fields,
                );
                return Err(error.into());
            }
        };
        let history_id = match self
            .insert_history(
                &resolved,
                status.as_u16(),
                duration_ms,
                &response_headers,
                &body,
            )
            .await
        {
            Ok(history_id) => history_id,
            Err(error) => {
                unfour_diag::log_operation_event(
                    "api_request_failed",
                    "api_client",
                    "send",
                    "error",
                    Some(started.elapsed().as_millis()),
                    Some(unfour_diag::app_error_kind(&error)),
                    serde_json::json!({
                        "request_id": request_id.as_str(),
                        "method": method.as_str(),
                        "host": url.host_str().unwrap_or(""),
                        "path": url.path(),
                        "status_code": status.as_u16(),
                    }),
                );
                return Err(error);
            }
        };
        unfour_diag::log_operation_event(
            "api_request_completed",
            "api_client",
            "send",
            "ok",
            Some(duration_ms),
            None,
            serde_json::json!({
                "request_id": request_id.as_str(),
                "method": method.as_str(),
                "host": url.host_str().unwrap_or(""),
                "path": url.path(),
                "status_code": status.as_u16(),
            }),
        );

        Ok(ApiResponse {
            history_id,
            status: status.as_u16(),
            status_text: status.canonical_reason().unwrap_or("").to_string(),
            headers: response_headers,
            body,
            duration_ms,
        })
    }

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

    async fn get_environment(
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

    async fn ensure_environment_name_available(
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

    async fn active_environment_variables(&self, workspace_id: &str) -> AppResult<Vec<KeyValue>> {
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

    pub async fn list_history(
        &self,
        workspace_id: String,
        limit: Option<i64>,
    ) -> AppResult<Vec<ApiHistoryItem>> {
        validate_workspace_id(&workspace_id)?;
        let limit = limit.unwrap_or(50).clamp(1, 200);

        let items = sqlx::query_as::<_, ApiHistoryItem>(
            r#"
            SELECT
              id, workspace_id, name, method, url, status, duration_ms, created_at, updated_at
            FROM api_history
            WHERE workspace_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .bind(workspace_id)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await?;

        Ok(items)
    }

    pub async fn history_detail(
        &self,
        workspace_id: String,
        history_id: String,
    ) -> AppResult<ApiHistoryDetail> {
        validate_workspace_id(&workspace_id)?;
        if history_id.trim().is_empty() {
            return Err(AppError::Validation(
                "history id cannot be empty".to_string(),
            ));
        }

        let item = sqlx::query_as::<_, ApiHistoryDetail>(
            r#"
            SELECT
              id, workspace_id, name, method, url, request_headers_json, request_query_json,
              request_body, status, duration_ms, response_headers_json, response_body_preview,
              created_at, updated_at
            FROM api_history
            WHERE workspace_id = ?1 AND id = ?2
            "#,
        )
        .bind(workspace_id)
        .bind(history_id)
        .fetch_optional(self.db.pool())
        .await?;

        item.ok_or_else(|| AppError::NotFound("api history".to_string()))
    }

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

    async fn get_collection_folder(
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

    async fn get_collection_folder_tx(
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

    async fn ensure_collection_exists_tx(
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

    async fn first_or_create_collection_id_tx(
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

    async fn next_folder_sort_order_tx(
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

    async fn next_request_sort_order_tx(
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

    async fn folder_contains_descendant_tx(
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

    async fn get_collection(
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

    async fn saved_request_fields(
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

    async fn resolve_request_location_tx(
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

    async fn get_saved_request_for_workspace(
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

    async fn get_saved_request_for_workspace_tx(
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

    async fn insert_history(
        &self,
        input: &ApiRequestInput,
        status: u16,
        duration_ms: u128,
        response_headers: &[KeyValue],
        response_body: &str,
    ) -> AppResult<String> {
        let now = Utc::now().to_rfc3339();
        let id = unfour_core::id::new_id();
        let body_preview = response_body.chars().take(20_000).collect::<String>();

        sqlx::query(
            r#"
            INSERT INTO api_history (
              id, workspace_id, name, method, url, request_headers_json, request_query_json,
              request_body, status, duration_ms, response_headers_json, response_body_preview,
              created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)
            "#,
        )
        .bind(&id)
        .bind(&input.workspace_id)
        .bind(&input.name)
        .bind(input.method.to_uppercase())
        .bind(&input.url)
        .bind(serde_json::to_string(&input.headers)?)
        .bind(serde_json::to_string(&input.query)?)
        .bind(input.body.clone())
        .bind(i64::from(status))
        .bind(i64::try_from(duration_ms).unwrap_or(i64::MAX))
        .bind(serde_json::to_string(response_headers)?)
        .bind(body_preview)
        .bind(now)
        .execute(self.db.pool())
        .await?;

        Ok(id)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "api_client_tests/mod.rs"]
mod tests;
