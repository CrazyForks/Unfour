use super::*;
use crate::transaction::CommandActivity;
use unfour_core::domain::{
    CommandContext, DomainEntityKey, DomainSnapshot, ExternalApplyPage, ExternalApplyReport,
    ExternalWorkspaceApply, ExternalWorkspaceEnvironmentApply,
    ExternalWorkspaceEnvironmentVariableApply, ExternalWorkspaceVariableApply,
};

impl CommandBus {
    pub async fn read_domain_snapshot(&self, key: &DomainEntityKey) -> AppResult<DomainSnapshot> {
        self.workspace.read_snapshot(key).await
    }

    pub async fn apply_external_page(
        &self,
        page: ExternalApplyPage,
    ) -> AppResult<ExternalApplyReport> {
        let counts = serde_json::json!({
            "workspaceCount": page.workspaces.len(),
            "workspaceVariableCount": page.workspace_variables.len(),
            "workspaceEnvironmentCount": page.workspace_environments.len(),
            "workspaceEnvironmentVariableCount": page.workspace_environment_variables.len(),
        });
        let context = CommandContext::external("workspace.external.apply_page");
        let executor_context = context.clone();
        let service = self.workspace.clone();
        self.execute_domain_command(
            context,
            Some(CommandActivity {
                workspace_id: None,
                action: "workspace.external.apply_page",
                target: None,
                details: counts,
            }),
            move |connection| {
                Box::pin(async move {
                    service
                        .apply_external_page_on(connection, &executor_context, page)
                        .await
                })
            },
        )
        .await
    }

    pub async fn apply_external_workspaces(
        &self,
        changes: Vec<ExternalWorkspaceApply>,
    ) -> AppResult<ExternalApplyReport> {
        self.apply_external_page(ExternalApplyPage {
            workspaces: changes,
            ..ExternalApplyPage::default()
        })
        .await
    }

    pub async fn apply_external_workspace_variables(
        &self,
        changes: Vec<ExternalWorkspaceVariableApply>,
    ) -> AppResult<ExternalApplyReport> {
        self.apply_external_page(ExternalApplyPage {
            workspace_variables: changes,
            ..ExternalApplyPage::default()
        })
        .await
    }

    pub async fn apply_external_workspace_environments(
        &self,
        changes: Vec<ExternalWorkspaceEnvironmentApply>,
    ) -> AppResult<ExternalApplyReport> {
        self.apply_external_page(ExternalApplyPage {
            workspace_environments: changes,
            ..ExternalApplyPage::default()
        })
        .await
    }

    pub async fn apply_external_workspace_environment_variables(
        &self,
        changes: Vec<ExternalWorkspaceEnvironmentVariableApply>,
    ) -> AppResult<ExternalApplyReport> {
        self.apply_external_page(ExternalApplyPage {
            workspace_environment_variables: changes,
            ..ExternalApplyPage::default()
        })
        .await
    }
}
