use crate::AppState;
use tauri::State;
use unfour_core::{
    models::{Workspace, WorkspaceLayout, WorkspaceState},
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
