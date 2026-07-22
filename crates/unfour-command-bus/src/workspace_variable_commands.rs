use super::*;

impl CommandBus {
    pub async fn workspace_variables_list(
        &self,
        workspace_id: String,
    ) -> AppResult<Vec<WorkspaceVariable>> {
        self.workspace.list_variables(workspace_id).await
    }

    pub async fn workspace_variables_replace(
        &self,
        workspace_id: String,
        variables: Vec<WorkspaceVariableInput>,
    ) -> AppResult<Vec<WorkspaceVariable>> {
        let variables = self
            .workspace
            .replace_variables(workspace_id.clone(), variables)
            .await?;
        self.activity_log
            .record(
                Some(&workspace_id),
                "workspace.variables.update",
                Some(&workspace_id),
                serde_json::json!({ "variableCount": variables.len() }),
            )
            .await?;
        Ok(variables)
    }

    pub async fn workspace_environments_list(
        &self,
        workspace_id: String,
    ) -> AppResult<Vec<WorkspaceEnvironment>> {
        self.workspace.list_environments(workspace_id).await
    }

    pub async fn workspace_environment_create(
        &self,
        workspace_id: String,
        name: String,
    ) -> AppResult<WorkspaceEnvironment> {
        let environment = self
            .workspace
            .create_environment(workspace_id.clone(), name)
            .await?;
        self.activity_log
            .record(
                Some(&workspace_id),
                "workspace.environment.create",
                Some(&environment.id),
                serde_json::json!({ "name": environment.name }),
            )
            .await?;
        Ok(environment)
    }

    pub async fn workspace_environment_update(
        &self,
        workspace_id: String,
        environment_id: String,
        name: String,
        variables: Vec<WorkspaceVariableInput>,
    ) -> AppResult<WorkspaceEnvironment> {
        let environment = self
            .workspace
            .update_environment(workspace_id.clone(), environment_id, name, variables)
            .await?;
        self.activity_log
            .record(
                Some(&workspace_id),
                "workspace.environment.update",
                Some(&environment.id),
                serde_json::json!({
                    "name": environment.name,
                    "variableCount": environment.variables.len(),
                }),
            )
            .await?;
        Ok(environment)
    }

    pub async fn workspace_environment_delete(
        &self,
        workspace_id: String,
        environment_id: String,
    ) -> AppResult<Vec<WorkspaceEnvironment>> {
        let environments = self
            .workspace
            .delete_environment(workspace_id.clone(), environment_id.clone())
            .await?;
        self.activity_log
            .record(
                Some(&workspace_id),
                "workspace.environment.delete",
                Some(&environment_id),
                serde_json::json!({ "softDelete": true }),
            )
            .await?;
        Ok(environments)
    }

    pub async fn workspace_environment_set_active(
        &self,
        workspace_id: String,
        environment_id: Option<String>,
    ) -> AppResult<Vec<WorkspaceEnvironment>> {
        self.workspace
            .set_active_environment(workspace_id, environment_id)
            .await
    }

    pub async fn workspace_variables_resolve(
        &self,
        workspace_id: String,
        active_environment_id: Option<String>,
        input: String,
    ) -> AppResult<String> {
        self.workspace
            .resolve_variables(&workspace_id, active_environment_id.as_deref(), &input)
            .await
    }

    pub(crate) async fn resolve_api_request_input(
        &self,
        mut input: ApiRequestInput,
    ) -> AppResult<ApiRequestInput> {
        let active_environment_id = self
            .workspace
            .active_environment_id(&input.workspace_id)
            .await?;
        input.url = self
            .workspace
            .resolve_variables(
                &input.workspace_id,
                active_environment_id.as_deref(),
                &input.url,
            )
            .await?;
        input.auth_json = match input.auth_json {
            Some(auth_json) => Some(
                self.workspace
                    .resolve_variables(
                        &input.workspace_id,
                        active_environment_id.as_deref(),
                        &auth_json,
                    )
                    .await?,
            ),
            None => None,
        };
        input.body = match input.body {
            Some(body) => Some(
                self.workspace
                    .resolve_variables(&input.workspace_id, active_environment_id.as_deref(), &body)
                    .await?,
            ),
            None => None,
        };
        input.headers = self
            .resolve_api_key_values(
                &input.workspace_id,
                active_environment_id.as_deref(),
                input.headers,
            )
            .await?;
        input.query = self
            .resolve_api_key_values(
                &input.workspace_id,
                active_environment_id.as_deref(),
                input.query,
            )
            .await?;
        Ok(input)
    }

    async fn resolve_api_key_values(
        &self,
        workspace_id: &str,
        active_environment_id: Option<&str>,
        values: Vec<KeyValue>,
    ) -> AppResult<Vec<KeyValue>> {
        let mut resolved = Vec::with_capacity(values.len());
        for value in values {
            resolved.push(KeyValue {
                key: self
                    .workspace
                    .resolve_variables(workspace_id, active_environment_id, &value.key)
                    .await?,
                value: self
                    .workspace
                    .resolve_variables(workspace_id, active_environment_id, &value.value)
                    .await?,
                enabled: value.enabled,
            });
        }
        Ok(resolved)
    }
}

pub(crate) fn legacy_api_environment(environment: WorkspaceEnvironment) -> ApiEnvironment {
    ApiEnvironment {
        id: environment.id,
        workspace_id: environment.workspace_id,
        name: environment.name,
        variables: environment
            .variables
            .into_iter()
            .map(|variable| KeyValue {
                key: variable.key,
                value: variable.value,
                enabled: variable.is_enabled,
            })
            .collect(),
        is_active: environment.is_active,
        created_at: environment.created_at,
        updated_at: environment.updated_at,
    }
}
