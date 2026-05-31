use crate::app_error::AppResult;
use crate::models::{
    ApiHistoryItem, ApiRequestInput, ApiResponse, ApiSavedRequest, KeyValue, SystemHealth,
    Workspace, WorkspaceEnvironment, WorkspaceState,
};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn system_health(state: State<'_, AppState>) -> AppResult<SystemHealth> {
    state.command_bus.system_health().await
}

#[tauri::command]
pub async fn workspace_list(state: State<'_, AppState>) -> AppResult<WorkspaceState> {
    state.command_bus.list_workspaces().await
}

#[tauri::command]
pub async fn workspace_create(
    name: String,
    state: State<'_, AppState>,
) -> AppResult<Workspace> {
    state.command_bus.create_workspace(name).await
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
pub async fn workspace_environment_get(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<WorkspaceEnvironment> {
    state.command_bus.workspace_environment(workspace_id).await
}

#[tauri::command]
pub async fn workspace_environment_update(
    workspace_id: String,
    variables: Vec<KeyValue>,
    state: State<'_, AppState>,
) -> AppResult<WorkspaceEnvironment> {
    state
        .command_bus
        .workspace_environment_update(workspace_id, variables)
        .await
}

#[tauri::command]
pub async fn api_send_request(
    input: ApiRequestInput,
    state: State<'_, AppState>,
) -> AppResult<ApiResponse> {
    state.command_bus.send_api_request(input).await
}

#[tauri::command]
pub async fn api_history_list(
    workspace_id: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiHistoryItem>> {
    state.command_bus.list_api_history(workspace_id, limit).await
}

#[tauri::command]
pub async fn api_request_save(
    input: ApiRequestInput,
    state: State<'_, AppState>,
) -> AppResult<ApiSavedRequest> {
    state.command_bus.save_api_request(input).await
}

#[tauri::command]
pub async fn api_saved_requests(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiSavedRequest>> {
    state.command_bus.list_saved_api_requests(workspace_id).await
}
