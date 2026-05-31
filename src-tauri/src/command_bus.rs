use crate::ai_reserved;
use crate::app_error::AppResult;
use crate::audit_log::AuditLogService;
use crate::local_db::LocalDb;
use crate::models::{
    ApiHistoryItem, ApiRequestInput, ApiResponse, ApiSavedRequest, SystemHealth, Workspace,
    WorkspaceEnvironment, WorkspaceState,
};
use crate::services::api_client::ApiClientService;
use crate::services::database::DatabaseService;
use crate::services::secret_store::SecretStore;
use crate::services::ssh::SshService;
use crate::services::workspace::WorkspaceService;
use crate::sync_reserved;
use tauri::AppHandle;

#[derive(Clone)]
pub struct CommandBus {
    api_client: ApiClientService,
    audit_log: AuditLogService,
    database: DatabaseService,
    secret_store: SecretStore,
    ssh: SshService,
    workspace: WorkspaceService,
}

impl CommandBus {
    pub async fn new(app: AppHandle) -> AppResult<Self> {
        let db = LocalDb::connect(&app).await?;
        db.migrate().await?;

        let audit_log = AuditLogService::new(db.clone());
        let secret_store = SecretStore::new("unfour-workspace");
        let workspace = WorkspaceService::new(db.clone());
        workspace.ensure_default_workspace().await?;

        Ok(Self {
            api_client: ApiClientService::new(db.clone()),
            audit_log,
            database: DatabaseService::new(),
            secret_store,
            ssh: SshService::new(),
            workspace,
        })
    }

    pub async fn system_health(&self) -> AppResult<SystemHealth> {
        Ok(SystemHealth {
            app_name: "Unfour Workspace".to_string(),
            storage_ready: true,
            command_bus_ready: true,
            ai_reserved_capabilities: ai_reserved::capability_ids(),
            sync_strategy: sync_reserved::default_policy().strategy,
        })
    }

    pub async fn list_workspaces(&self) -> AppResult<WorkspaceState> {
        self.workspace.state().await
    }

    pub async fn create_workspace(&self, name: String) -> AppResult<Workspace> {
        let workspace = self.workspace.create(name).await?;
        self.audit_log
            .record(
                Some(&workspace.id),
                "workspace.create",
                Some(&workspace.id),
                serde_json::json!({ "name": workspace.name }),
            )
            .await?;
        Ok(workspace)
    }

    pub async fn rename_workspace(&self, workspace_id: String, name: String) -> AppResult<Workspace> {
        let workspace = self.workspace.rename(workspace_id, name).await?;
        self.audit_log
            .record(
                Some(&workspace.id),
                "workspace.rename",
                Some(&workspace.id),
                serde_json::json!({ "name": workspace.name }),
            )
            .await?;
        Ok(workspace)
    }

    pub async fn delete_workspace(&self, workspace_id: String) -> AppResult<WorkspaceState> {
        let state = self.workspace.delete(workspace_id.clone()).await?;
        self.audit_log
            .record(
                Some(&workspace_id),
                "workspace.delete",
                Some(&workspace_id),
                serde_json::json!({ "softDelete": true }),
            )
            .await?;
        Ok(state)
    }

    pub async fn set_active_workspace(&self, workspace_id: String) -> AppResult<WorkspaceState> {
        self.workspace.set_active(workspace_id).await
    }

    pub async fn workspace_environment(
        &self,
        workspace_id: String,
    ) -> AppResult<WorkspaceEnvironment> {
        self.workspace.environment(workspace_id).await
    }

    pub async fn workspace_environment_update(
        &self,
        workspace_id: String,
        variables: Vec<crate::models::KeyValue>,
    ) -> AppResult<WorkspaceEnvironment> {
        let environment = self
            .workspace
            .update_environment(workspace_id.clone(), variables)
            .await?;
        self.audit_log
            .record(
                Some(&workspace_id),
                "workspace.environment.update",
                Some(&workspace_id),
                serde_json::json!({ "variableCount": environment.variables.len() }),
            )
            .await?;
        Ok(environment)
    }

    pub async fn send_api_request(&self, input: ApiRequestInput) -> AppResult<ApiResponse> {
        let environment = self.workspace.environment(input.workspace_id.clone()).await?;
        let response = self.api_client.send(input.clone(), &environment.variables).await?;
        self.audit_log
            .record(
                Some(&input.workspace_id),
                "api.send_request",
                Some(&response.history_id),
                serde_json::json!({
                    "method": input.method,
                    "url": input.url,
                    "status": response.status
                }),
            )
            .await?;
        Ok(response)
    }

    pub async fn list_api_history(
        &self,
        workspace_id: String,
        limit: Option<i64>,
    ) -> AppResult<Vec<ApiHistoryItem>> {
        self.api_client.list_history(workspace_id, limit).await
    }

    pub async fn save_api_request(&self, input: ApiRequestInput) -> AppResult<ApiSavedRequest> {
        let saved = self.api_client.save_request(input).await?;
        self.audit_log
            .record(
                Some(&saved.workspace_id),
                "api.save_request",
                Some(&saved.id),
                serde_json::json!({ "name": saved.name, "method": saved.method }),
            )
            .await?;
        Ok(saved)
    }

    pub async fn list_saved_api_requests(
        &self,
        workspace_id: String,
    ) -> AppResult<Vec<ApiSavedRequest>> {
        self.api_client.list_saved_requests(workspace_id).await
    }

    pub fn reserved_status(&self) -> serde_json::Value {
        serde_json::json!({
            "ssh": self.ssh.capability_summary(),
            "database": self.database.capability_summary(),
            "secrets": self.secret_store.capability_summary()
        })
    }
}
