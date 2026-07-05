use crate::AppState;
use tauri::State;
use unfour_core::{
    models::{
        ApiCollection, ApiCollectionFolder, ApiEnvironment, ApiHistoryDetail, ApiHistoryItem,
        ApiRequestInput, ApiResponse, ApiSavedRequest, KeyValue,
    },
    AppResult,
};

use super::trace_command;

#[tauri::command]
pub async fn api_environments_list(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiEnvironment>> {
    state.command_bus.api_environments_list(workspace_id).await
}

#[tauri::command]
pub async fn api_environment_create(
    workspace_id: String,
    name: String,
    state: State<'_, AppState>,
) -> AppResult<ApiEnvironment> {
    state
        .command_bus
        .api_environment_create(workspace_id, name)
        .await
}

#[tauri::command]
pub async fn api_environment_update(
    workspace_id: String,
    environment_id: String,
    name: String,
    variables: Vec<KeyValue>,
    state: State<'_, AppState>,
) -> AppResult<ApiEnvironment> {
    state
        .command_bus
        .api_environment_update(workspace_id, environment_id, name, variables)
        .await
}

#[tauri::command]
pub async fn api_environment_delete(
    workspace_id: String,
    environment_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiEnvironment>> {
    state
        .command_bus
        .api_environment_delete(workspace_id, environment_id)
        .await
}

#[tauri::command]
pub async fn api_environment_activate(
    workspace_id: String,
    environment_id: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiEnvironment>> {
    state
        .command_bus
        .api_environment_activate(workspace_id, environment_id)
        .await
}

#[tauri::command]
pub async fn api_collection_list(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiCollection>> {
    state.command_bus.api_collection_list(workspace_id).await
}

#[tauri::command]
pub async fn api_collection_create(
    workspace_id: String,
    name: String,
    state: State<'_, AppState>,
) -> AppResult<ApiCollection> {
    state
        .command_bus
        .api_collection_create(workspace_id, name)
        .await
}

#[tauri::command]
pub async fn api_collection_rename(
    workspace_id: String,
    collection_id: String,
    name: String,
    state: State<'_, AppState>,
) -> AppResult<ApiCollection> {
    state
        .command_bus
        .api_collection_rename(workspace_id, collection_id, name)
        .await
}

#[tauri::command]
pub async fn api_collection_delete(
    workspace_id: String,
    collection_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiCollection>> {
    state
        .command_bus
        .api_collection_delete(workspace_id, collection_id)
        .await
}

#[tauri::command]
pub async fn api_collection_folders_list(
    workspace_id: String,
    collection_id: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiCollectionFolder>> {
    state
        .command_bus
        .api_collection_folders_list(workspace_id, collection_id)
        .await
}

#[tauri::command]
pub async fn api_collection_folder_create(
    workspace_id: String,
    collection_id: String,
    parent_folder_id: Option<String>,
    name: String,
    state: State<'_, AppState>,
) -> AppResult<ApiCollectionFolder> {
    state
        .command_bus
        .api_collection_folder_create(workspace_id, collection_id, parent_folder_id, name)
        .await
}

#[tauri::command]
pub async fn api_collection_folder_rename(
    workspace_id: String,
    folder_id: String,
    name: String,
    state: State<'_, AppState>,
) -> AppResult<ApiCollectionFolder> {
    state
        .command_bus
        .api_collection_folder_rename(workspace_id, folder_id, name)
        .await
}

#[tauri::command]
pub async fn api_collection_folder_delete(
    workspace_id: String,
    folder_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiCollectionFolder>> {
    state
        .command_bus
        .api_collection_folder_delete(workspace_id, folder_id)
        .await
}

#[tauri::command]
pub async fn api_collection_folder_move(
    workspace_id: String,
    folder_id: String,
    target_parent_folder_id: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<ApiCollectionFolder> {
    state
        .command_bus
        .api_collection_folder_move(workspace_id, folder_id, target_parent_folder_id)
        .await
}

#[tauri::command]
pub async fn api_collection_folders_reorder(
    workspace_id: String,
    collection_id: String,
    parent_folder_id: Option<String>,
    folder_ids: Vec<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiCollectionFolder>> {
    state
        .command_bus
        .api_collection_folders_reorder(workspace_id, collection_id, parent_folder_id, folder_ids)
        .await
}

#[tauri::command]
pub async fn api_request_move(
    workspace_id: String,
    request_id: String,
    collection_id: Option<String>,
    parent_folder_id: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<ApiSavedRequest> {
    state
        .command_bus
        .api_request_move(workspace_id, request_id, collection_id, parent_folder_id)
        .await
}

#[tauri::command]
pub async fn api_requests_reorder(
    workspace_id: String,
    collection_id: String,
    parent_folder_id: Option<String>,
    request_ids: Vec<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiSavedRequest>> {
    state
        .command_bus
        .api_requests_reorder(workspace_id, collection_id, parent_folder_id, request_ids)
        .await
}

#[tauri::command]
pub async fn api_send_request(
    input: ApiRequestInput,
    state: State<'_, AppState>,
) -> AppResult<ApiResponse> {
    trace_command(
        "api_send_request",
        state.command_bus.send_api_request(input),
    )
    .await
}

#[tauri::command]
pub async fn api_history_list(
    workspace_id: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiHistoryItem>> {
    state
        .command_bus
        .list_api_history(workspace_id, limit)
        .await
}

#[tauri::command]
pub async fn api_history_detail(
    workspace_id: String,
    history_id: String,
    state: State<'_, AppState>,
) -> AppResult<ApiHistoryDetail> {
    state
        .command_bus
        .api_history_detail(workspace_id, history_id)
        .await
}

#[tauri::command]
pub async fn api_request_save(
    input: ApiRequestInput,
    state: State<'_, AppState>,
) -> AppResult<ApiSavedRequest> {
    state.command_bus.save_api_request(input).await
}

#[tauri::command]
pub async fn api_request_update(
    workspace_id: String,
    request_id: String,
    input: ApiRequestInput,
    state: State<'_, AppState>,
) -> AppResult<ApiSavedRequest> {
    state
        .command_bus
        .update_api_request(workspace_id, request_id, input)
        .await
}

#[tauri::command]
pub async fn api_saved_requests(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiSavedRequest>> {
    state
        .command_bus
        .list_saved_api_requests(workspace_id)
        .await
}

#[tauri::command]
pub async fn api_request_duplicate(
    workspace_id: String,
    request_id: String,
    state: State<'_, AppState>,
) -> AppResult<ApiSavedRequest> {
    state
        .command_bus
        .duplicate_api_request(workspace_id, request_id)
        .await
}

#[tauri::command]
pub async fn api_request_delete(
    workspace_id: String,
    request_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<ApiSavedRequest>> {
    state
        .command_bus
        .delete_api_request(workspace_id, request_id)
        .await
}
