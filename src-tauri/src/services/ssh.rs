use crate::app_error::{AppError, AppResult};
use crate::local_db::LocalDb;
use crate::models::{SshConnection, SshConnectionConfig, SshConnectionInput, StoredConnection};
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct SshService {
    db: LocalDb,
}

impl SshService {
    pub fn new(db: LocalDb) -> Self {
        Self { db }
    }

    pub async fn list_connections(&self, workspace_id: String) -> AppResult<Vec<SshConnection>> {
        validate_workspace_id(&workspace_id)?;

        let rows = sqlx::query_as::<_, StoredConnection>(
            r#"
            SELECT
              id, workspace_id, kind, name, config_json, credential_ref, created_at,
              updated_at, deleted_at, revision, sync_status, remote_id
            FROM connections
            WHERE workspace_id = ?1 AND kind = 'ssh' AND deleted_at IS NULL
            ORDER BY updated_at DESC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(self.db.pool())
        .await?;

        rows.into_iter().map(stored_to_ssh_connection).collect()
    }

    pub async fn save_connection(&self, input: SshConnectionInput) -> AppResult<SshConnection> {
        validate_workspace_id(&input.workspace_id)?;
        let name = normalize_name(&input.name)?;
        let config = input_to_config(&input)?;
        let credential_ref = empty_to_none(input.credential_ref);
        validate_credential_boundary(&config, credential_ref.as_deref())?;
        let now = Utc::now().to_rfc3339();
        let config_json = serde_json::to_string(&config)?;

        if let Some(id) = input
            .id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
        {
            let result = sqlx::query(
                r#"
                UPDATE connections
                SET name = ?1, config_json = ?2, credential_ref = ?3,
                    updated_at = ?4, revision = revision + 1, sync_status = 'pending'
                WHERE id = ?5 AND workspace_id = ?6 AND kind = 'ssh' AND deleted_at IS NULL
                "#,
            )
            .bind(name)
            .bind(config_json)
            .bind(credential_ref)
            .bind(now)
            .bind(id)
            .bind(&input.workspace_id)
            .execute(self.db.pool())
            .await?;

            if result.rows_affected() == 0 {
                return Err(AppError::NotFound("ssh connection".to_string()));
            }

            return self.get_connection(&input.workspace_id, id).await;
        }

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO connections (
              id, workspace_id, kind, name, config_json, credential_ref,
              created_at, updated_at, revision, sync_status
            )
            VALUES (?1, ?2, 'ssh', ?3, ?4, ?5, ?6, ?6, 1, 'local')
            "#,
        )
        .bind(&id)
        .bind(&input.workspace_id)
        .bind(name)
        .bind(config_json)
        .bind(credential_ref)
        .bind(now)
        .execute(self.db.pool())
        .await?;

        self.get_connection(&input.workspace_id, &id).await
    }

    pub async fn delete_connection(
        &self,
        workspace_id: String,
        connection_id: String,
    ) -> AppResult<Vec<SshConnection>> {
        validate_workspace_id(&workspace_id)?;
        validate_connection_id(&connection_id)?;
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE connections
            SET deleted_at = ?1, updated_at = ?1, revision = revision + 1, sync_status = 'deleted'
            WHERE id = ?2 AND workspace_id = ?3 AND kind = 'ssh' AND deleted_at IS NULL
            "#,
        )
        .bind(now)
        .bind(connection_id)
        .bind(&workspace_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("ssh connection".to_string()));
        }

        self.list_connections(workspace_id).await
    }

    pub fn capability_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "metadata-mvp",
            "plannedBackend": "russh",
            "features": [
                "connection-metadata-crud",
                "credential-ref-boundary",
                "password-auth-reserved",
                "private-key-auth-reserved",
                "terminal-streaming-reserved"
            ]
        })
    }

    async fn get_connection(&self, workspace_id: &str, id: &str) -> AppResult<SshConnection> {
        validate_workspace_id(workspace_id)?;
        validate_connection_id(id)?;

        let row = sqlx::query_as::<_, StoredConnection>(
            r#"
            SELECT
              id, workspace_id, kind, name, config_json, credential_ref, created_at,
              updated_at, deleted_at, revision, sync_status, remote_id
            FROM connections
            WHERE id = ?1 AND workspace_id = ?2 AND kind = 'ssh' AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(workspace_id)
        .fetch_optional(self.db.pool())
        .await?;

        row.map(stored_to_ssh_connection)
            .transpose()?
            .ok_or_else(|| AppError::NotFound("ssh connection".to_string()))
    }
}

fn stored_to_ssh_connection(row: StoredConnection) -> AppResult<SshConnection> {
    let config = serde_json::from_str::<SshConnectionConfig>(&row.config_json)?;
    Ok(SshConnection {
        id: row.id,
        workspace_id: row.workspace_id,
        name: row.name,
        host: config.host,
        port: config.port,
        username: config.username,
        auth_kind: config.auth_kind,
        key_path: config.key_path,
        credential_ref: row.credential_ref,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
        revision: row.revision,
        sync_status: row.sync_status,
        remote_id: row.remote_id,
    })
}

fn input_to_config(input: &SshConnectionInput) -> AppResult<SshConnectionConfig> {
    let host = normalize_required(&input.host, "ssh host")?;
    let username = normalize_required(&input.username, "ssh username")?;
    let auth_kind = input.auth_kind.trim().to_ascii_lowercase();
    if !matches!(auth_kind.as_str(), "password" | "private-key") {
        return Err(AppError::Validation(format!(
            "unsupported ssh auth kind: {}",
            input.auth_kind
        )));
    }

    let port = input.port.unwrap_or(22);
    if port == 0 {
        return Err(AppError::Validation("ssh port cannot be 0".to_string()));
    }

    let key_path = empty_to_none(input.key_path.clone());
    if auth_kind == "private-key" && key_path.is_none() {
        return Err(AppError::Validation(
            "private-key ssh auth requires a key path".to_string(),
        ));
    }

    Ok(SshConnectionConfig {
        host,
        port,
        username,
        auth_kind,
        key_path,
    })
}

fn validate_credential_boundary(
    config: &SshConnectionConfig,
    credential_ref: Option<&str>,
) -> AppResult<()> {
    if config.auth_kind == "password" && credential_ref.is_none() {
        return Err(AppError::Validation(
            "password ssh auth requires a credential reference".to_string(),
        ));
    }

    Ok(())
}

fn normalize_name(name: &str) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "ssh connection name cannot be empty".to_string(),
        ));
    }
    if trimmed.chars().count() > 80 {
        return Err(AppError::Validation(
            "ssh connection name must be 80 characters or fewer".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

fn normalize_required(value: &str, label: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{} cannot be empty", label)));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(AppError::Validation(format!(
            "{} cannot contain control characters",
            label
        )));
    }
    Ok(trimmed.to_string())
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn validate_workspace_id(workspace_id: &str) -> AppResult<()> {
    if workspace_id.trim().is_empty() {
        return Err(AppError::Validation(
            "workspace id cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_connection_id(connection_id: &str) -> AppResult<()> {
    if connection_id.trim().is_empty() {
        return Err(AppError::Validation(
            "ssh connection id cannot be empty".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    async fn service_with_workspaces() -> (SshService, String, String) {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect in-memory app db");
        let db = LocalDb::from_pool(pool);
        db.migrate().await.expect("run migrations");

        let workspace_a = Uuid::new_v4().to_string();
        let workspace_b = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        for workspace_id in [&workspace_a, &workspace_b] {
            sqlx::query(
                r#"
                INSERT INTO workspaces (
                  id, name, is_default, last_opened_at, created_at, updated_at,
                  revision, sync_status
                )
                VALUES (?1, 'Test Workspace', 0, ?2, ?2, ?2, 1, 'local')
                "#,
            )
            .bind(workspace_id)
            .bind(&now)
            .execute(db.pool())
            .await
            .expect("insert workspace");
        }

        (SshService::new(db), workspace_a, workspace_b)
    }

    fn password_input(workspace_id: &str) -> SshConnectionInput {
        SshConnectionInput {
            id: None,
            workspace_id: workspace_id.to_string(),
            name: "Deploy host".to_string(),
            host: " example.internal ".to_string(),
            port: None,
            username: " deploy ".to_string(),
            auth_kind: "password".to_string(),
            key_path: None,
            credential_ref: Some("ssh-password-1".to_string()),
        }
    }

    #[tokio::test]
    async fn ssh_connection_crud_is_workspace_scoped_and_soft_deletes() {
        let (service, workspace_a, workspace_b) = service_with_workspaces().await;

        let created = service
            .save_connection(password_input(&workspace_a))
            .await
            .expect("save ssh connection");
        assert_eq!(created.host, "example.internal");
        assert_eq!(created.port, 22);
        assert_eq!(created.username, "deploy");
        assert_eq!(created.credential_ref.as_deref(), Some("ssh-password-1"));

        let workspace_a_items = service
            .list_connections(workspace_a.clone())
            .await
            .expect("list workspace a");
        let workspace_b_items = service
            .list_connections(workspace_b)
            .await
            .expect("list workspace b");
        assert_eq!(workspace_a_items.len(), 1);
        assert!(workspace_b_items.is_empty());

        let updated = service
            .save_connection(SshConnectionInput {
                id: Some(created.id.clone()),
                name: "Deploy bastion".to_string(),
                port: Some(2222),
                ..password_input(&workspace_a)
            })
            .await
            .expect("update ssh connection");
        assert_eq!(updated.name, "Deploy bastion");
        assert_eq!(updated.port, 2222);
        assert_eq!(updated.sync_status, "pending");

        let remaining = service
            .delete_connection(workspace_a.clone(), created.id)
            .await
            .expect("delete ssh connection");
        assert!(remaining.is_empty());
        assert!(service
            .list_connections(workspace_a)
            .await
            .expect("list after delete")
            .is_empty());
    }

    #[tokio::test]
    async fn ssh_connection_validation_keeps_secrets_out_of_config() {
        let (service, workspace_id, _) = service_with_workspaces().await;

        let missing_credential = service
            .save_connection(SshConnectionInput {
                credential_ref: None,
                ..password_input(&workspace_id)
            })
            .await;
        assert!(matches!(missing_credential, Err(AppError::Validation(_))));

        let private_key = service
            .save_connection(SshConnectionInput {
                auth_kind: "private-key".to_string(),
                key_path: Some("C:/Users/zhang/.ssh/id_ed25519".to_string()),
                credential_ref: Some("ssh-key-passphrase-1".to_string()),
                ..password_input(&workspace_id)
            })
            .await
            .expect("save private key metadata");

        let stored_config: (String,) =
            sqlx::query_as("SELECT config_json FROM connections WHERE id = ?1")
                .bind(private_key.id)
                .fetch_one(service.db.pool())
                .await
                .expect("load stored config");
        assert!(stored_config.0.contains("id_ed25519"));
        assert!(!stored_config.0.contains("ssh-key-passphrase-1"));
    }
}
