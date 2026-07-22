use crate::AppState;
use tauri::State;
use unfour_core::{
    models::{
        Workspace, WorkspaceEnvironment, WorkspaceLayout, WorkspaceState, WorkspaceVariable,
        WorkspaceVariableInput,
    },
    AppResult,
};

#[tauri::command]
pub async fn workspace_list(state: State<'_, AppState>) -> AppResult<WorkspaceState> {
    state.command_bus.list_workspaces().await
}

#[tauri::command]
pub async fn workspace_create(
    name: String,
    environment_type: Option<String>,
    mcp_policy: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Workspace> {
    state
        .command_bus
        .create_workspace_with_options(name, environment_type, mcp_policy)
        .await
}

#[tauri::command]
pub async fn workspace_update_environment(
    workspace_id: String,
    environment_type: String,
    state: State<'_, AppState>,
) -> AppResult<Workspace> {
    state
        .command_bus
        .update_workspace_environment(workspace_id, environment_type)
        .await
}

#[tauri::command]
pub async fn workspace_rename(
    workspace_id: String,
    name: String,
    state: State<'_, AppState>,
) -> AppResult<Workspace> {
    state.command_bus.rename_workspace(workspace_id, name).await
}

#[tauri::command]
pub async fn workspace_delete(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<WorkspaceState> {
    state.command_bus.delete_workspace(workspace_id).await
}

#[tauri::command]
pub async fn workspace_set_active(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<WorkspaceState> {
    state.command_bus.set_active_workspace(workspace_id).await
}

#[tauri::command]
pub async fn workspace_layout_get(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<WorkspaceLayout> {
    state.command_bus.workspace_layout(workspace_id).await
}

#[tauri::command]
pub async fn workspace_layout_update(
    workspace_id: String,
    layout: WorkspaceLayout,
    state: State<'_, AppState>,
) -> AppResult<WorkspaceLayout> {
    state
        .command_bus
        .workspace_layout_update(workspace_id, layout)
        .await
}

#[tauri::command]
pub async fn workspace_variables_list(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<WorkspaceVariable>> {
    state
        .command_bus
        .workspace_variables_list(workspace_id)
        .await
}

#[tauri::command]
pub async fn workspace_variables_replace(
    workspace_id: String,
    variables: Vec<WorkspaceVariableInput>,
    state: State<'_, AppState>,
) -> AppResult<Vec<WorkspaceVariable>> {
    state
        .command_bus
        .workspace_variables_replace(workspace_id, variables)
        .await
}

#[tauri::command]
pub async fn workspace_environments_list(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<WorkspaceEnvironment>> {
    state
        .command_bus
        .workspace_environments_list(workspace_id)
        .await
}

#[tauri::command]
pub async fn workspace_environment_create(
    workspace_id: String,
    name: String,
    state: State<'_, AppState>,
) -> AppResult<WorkspaceEnvironment> {
    state
        .command_bus
        .workspace_environment_create(workspace_id, name)
        .await
}

#[tauri::command]
pub async fn workspace_environment_update(
    workspace_id: String,
    environment_id: String,
    name: String,
    variables: Vec<WorkspaceVariableInput>,
    state: State<'_, AppState>,
) -> AppResult<WorkspaceEnvironment> {
    state
        .command_bus
        .workspace_environment_update(workspace_id, environment_id, name, variables)
        .await
}

#[tauri::command]
pub async fn workspace_environment_delete(
    workspace_id: String,
    environment_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<WorkspaceEnvironment>> {
    state
        .command_bus
        .workspace_environment_delete(workspace_id, environment_id)
        .await
}

#[tauri::command]
pub async fn workspace_environment_set_active(
    workspace_id: String,
    environment_id: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<WorkspaceEnvironment>> {
    state
        .command_bus
        .workspace_environment_set_active(workspace_id, environment_id)
        .await
}

#[tauri::command]
pub async fn workspace_variables_resolve(
    workspace_id: String,
    active_environment_id: Option<String>,
    input: String,
    state: State<'_, AppState>,
) -> AppResult<String> {
    state
        .command_bus
        .workspace_variables_resolve(workspace_id, active_environment_id, input)
        .await
}
