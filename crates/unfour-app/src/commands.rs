use crate::AppState;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::Instant;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use unfour_core::models::{
    ApiCollection, ApiCollectionFolder, ApiEnvironment, ApiHistoryDetail, ApiHistoryItem,
    ApiRequestInput, ApiResponse, ApiSavedRequest, CredentialCreateInput, CredentialDeleteInput,
    CredentialInspectInput, CredentialMetadata, CredentialRotateInput, DatabaseBrowseInput,
    DatabaseBrowseResult, DatabaseConnection, DatabaseConnectionInput, DatabaseQueryInput,
    DatabaseQueryResult, DatabaseRowMutationInput, DatabaseRowMutationResult, DatabaseSchema,
    DatabaseTableStructure, DatabaseTableStructureInput, DatabaseTestResult, DbQueryHistoryEntry,
    DbQueryHistoryRecordInput, KeyValue, SavedSql, SavedSqlInput, SshCloseInput, SshConnectInput,
    SshConnection, SshConnectionInput, SshHostFingerprintInfo, SshHostKeyInput,
    SshKnownHostsExportInput, SshKnownHostsExportResult, SshKnownHostsImportInput,
    SshKnownHostsImportResult, SshLogExport, SshLogExportInput, SshReconnectCancelInput,
    SshResizeInput, SshSessionEvent, SshSessionInput, SshSessionSummary, SystemHealth, Workspace,
    WorkspaceLayout, WorkspaceState,
};
use unfour_core::AppResult;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticBundleCommandResult {
    pub bundle_dir: String,
    pub manifest_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendLogEntry {
    pub level: String,
    pub event: String,
    pub module: String,
    pub operation: String,
    pub fields: serde_json::Value,
}

async fn trace_command<T>(
    command_name: &'static str,
    future: impl Future<Output = AppResult<T>>,
) -> AppResult<T> {
    let command_id = unfour_diag::new_command_id();
    let started = Instant::now();
    unfour_diag::log_command_started(command_name, &command_id);
    let result = future.await;
    match &result {
        Ok(_) => unfour_diag::log_command_completed(
            command_name,
            &command_id,
            started.elapsed().as_millis(),
        ),
        Err(error) => unfour_diag::log_command_failed(
            command_name,
            &command_id,
            started.elapsed().as_millis(),
            unfour_diag::app_error_kind(error),
        ),
    }
    result
}

#[tauri::command]
pub async fn system_health(state: State<'_, AppState>) -> AppResult<SystemHealth> {
    trace_command("system_health", state.command_bus.system_health()).await
}

#[tauri::command]
pub async fn open_log_dir(app: AppHandle) -> AppResult<()> {
    trace_command("open_log_dir", async move {
        let paths = unfour_paths::initialize_unfour_storage()?;
        app.opener()
            .open_path(paths.logs_dir.to_string_lossy().to_string(), None::<&str>)
            .map_err(|error| {
                unfour_core::AppError::Config(format!("failed to open log directory: {error}"))
            })?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn open_diagnostics_dir(app: AppHandle) -> AppResult<()> {
    trace_command("open_diagnostics_dir", async move {
        let paths = unfour_paths::initialize_unfour_storage()?;
        app.opener()
            .open_path(
                paths.diagnostics_dir.to_string_lossy().to_string(),
                None::<&str>,
            )
            .map_err(|error| {
                unfour_core::AppError::Config(format!(
                    "failed to open diagnostics directory: {error}"
                ))
            })?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn export_diagnostics_bundle() -> AppResult<DiagnosticBundleCommandResult> {
    trace_command("export_diagnostics_bundle", async {
        let paths = unfour_paths::initialize_unfour_storage()?;
        let request = unfour_diag::DiagnosticBundleRequest::oss_dev(
            env!("CARGO_PKG_VERSION").to_string(),
            paths,
        );
        let bundle = unfour_diag::export_diagnostics_bundle(&request)?;

        Ok(DiagnosticBundleCommandResult {
            bundle_dir: bundle.bundle_dir.to_string_lossy().to_string(),
            manifest_path: bundle.manifest_path.to_string_lossy().to_string(),
        })
    })
    .await
}

#[tauri::command]
pub async fn frontend_log(entry: FrontendLogEntry) -> AppResult<()> {
    let status = if entry.level == "error" { "error" } else { "ok" };
    unfour_diag::log_operation_event(
        &entry.event,
        &entry.module,
        &entry.operation,
        status,
        None,
        None,
        entry.fields,
    );
    Ok(())
}

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
pub async fn api_send_request(
    input: ApiRequestInput,
    state: State<'_, AppState>,
) -> AppResult<ApiResponse> {
    trace_command("api_send_request", state.command_bus.send_api_request(input)).await
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

#[tauri::command]
pub async fn credential_create(
    input: CredentialCreateInput,
    state: State<'_, AppState>,
) -> AppResult<CredentialMetadata> {
    trace_command("credential_create", state.command_bus.create_credential(input)).await
}

#[tauri::command]
pub async fn credential_delete(
    input: CredentialDeleteInput,
    state: State<'_, AppState>,
) -> AppResult<()> {
    trace_command("credential_delete", state.command_bus.delete_credential(input)).await
}

#[tauri::command]
pub async fn credential_inspect(
    input: CredentialInspectInput,
    state: State<'_, AppState>,
) -> AppResult<CredentialMetadata> {
    trace_command(
        "credential_inspect",
        state.command_bus.inspect_credential(input),
    )
    .await
}

#[tauri::command]
pub async fn credential_rotate(
    input: CredentialRotateInput,
    state: State<'_, AppState>,
) -> AppResult<CredentialMetadata> {
    trace_command("credential_rotate", state.command_bus.rotate_credential(input)).await
}

#[tauri::command]
pub async fn database_connections_list(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<DatabaseConnection>> {
    state
        .command_bus
        .list_database_connections(workspace_id)
        .await
}

#[tauri::command]
pub async fn database_connection_save(
    input: DatabaseConnectionInput,
    state: State<'_, AppState>,
) -> AppResult<DatabaseConnection> {
    state.command_bus.save_database_connection(input).await
}

#[tauri::command]
pub async fn database_connection_delete(
    workspace_id: String,
    connection_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<DatabaseConnection>> {
    state
        .command_bus
        .delete_database_connection(workspace_id, connection_id)
        .await
}

#[tauri::command]
pub async fn database_connection_test(
    workspace_id: String,
    connection_id: String,
    state: State<'_, AppState>,
) -> AppResult<DatabaseTestResult> {
    trace_command(
        "database_connection_test",
        state
            .command_bus
            .test_database_connection(workspace_id, connection_id),
    )
    .await
}

#[tauri::command]
pub async fn database_schema_get(
    workspace_id: String,
    connection_id: String,
    catalog: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<DatabaseSchema> {
    state
        .command_bus
        .database_schema(workspace_id, connection_id, catalog)
        .await
}

#[tauri::command]
pub async fn database_catalogs_list(
    workspace_id: String,
    connection_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<String>> {
    state
        .command_bus
        .database_catalogs(workspace_id, connection_id)
        .await
}

#[tauri::command]
pub async fn database_query_execute(
    input: DatabaseQueryInput,
    state: State<'_, AppState>,
) -> AppResult<DatabaseQueryResult> {
    trace_command(
        "database_query_execute",
        state.command_bus.execute_database_query(input),
    )
    .await
}

#[tauri::command]
pub async fn database_query_history_record(
    input: DbQueryHistoryRecordInput,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state.command_bus.record_database_query_history(input).await
}

#[tauri::command]
pub async fn database_query_history_list(
    workspace_id: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> AppResult<Vec<DbQueryHistoryEntry>> {
    state
        .command_bus
        .list_database_query_history(workspace_id, limit)
        .await
}

#[tauri::command]
pub async fn database_query_history_clear(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state
        .command_bus
        .clear_database_query_history(workspace_id)
        .await
}

#[tauri::command]
pub async fn database_saved_sql_list(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<SavedSql>> {
    state.command_bus.list_saved_sql(workspace_id).await
}

#[tauri::command]
pub async fn database_saved_sql_save(
    input: SavedSqlInput,
    state: State<'_, AppState>,
) -> AppResult<SavedSql> {
    state.command_bus.save_saved_sql(input).await
}

#[tauri::command]
pub async fn database_saved_sql_delete(
    workspace_id: String,
    id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<SavedSql>> {
    state.command_bus.delete_saved_sql(workspace_id, id).await
}

#[tauri::command]
pub async fn database_table_browse(
    input: DatabaseBrowseInput,
    state: State<'_, AppState>,
) -> AppResult<DatabaseBrowseResult> {
    state.command_bus.browse_database_table(input).await
}

#[tauri::command]
pub async fn database_table_structure(
    input: DatabaseTableStructureInput,
    state: State<'_, AppState>,
) -> AppResult<DatabaseTableStructure> {
    state.command_bus.database_table_structure(input).await
}

#[tauri::command]
pub async fn database_row_mutate(
    input: DatabaseRowMutationInput,
    state: State<'_, AppState>,
) -> AppResult<DatabaseRowMutationResult> {
    state.command_bus.mutate_database_row(input).await
}

#[tauri::command]
pub async fn ssh_connections_list(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<SshConnection>> {
    state.command_bus.list_ssh_connections(workspace_id).await
}

#[tauri::command]
pub async fn ssh_connection_save(
    input: SshConnectionInput,
    state: State<'_, AppState>,
) -> AppResult<SshConnection> {
    state.command_bus.save_ssh_connection(input).await
}

#[tauri::command]
pub async fn ssh_connection_delete(
    workspace_id: String,
    connection_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<SshConnection>> {
    state
        .command_bus
        .delete_ssh_connection(workspace_id, connection_id)
        .await
}

#[tauri::command]
pub async fn ssh_session_connect(
    input: SshConnectInput,
    state: State<'_, AppState>,
) -> AppResult<SshSessionSummary> {
    trace_command("ssh_session_connect", state.command_bus.connect_ssh_session(input)).await
}

#[tauri::command]
pub async fn ssh_sessions_list(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<SshSessionSummary>> {
    state.command_bus.list_ssh_sessions(workspace_id).await
}

#[tauri::command]
pub async fn ssh_session_history(
    input: SshCloseInput,
    state: State<'_, AppState>,
) -> AppResult<Vec<SshSessionEvent>> {
    state.command_bus.ssh_session_history(input).await
}

#[tauri::command]
pub async fn ssh_session_input(
    input: SshSessionInput,
    state: State<'_, AppState>,
) -> AppResult<SshSessionEvent> {
    state.command_bus.send_ssh_input(input).await
}

/// Register the frontend's IPC channel as the live terminal-output sink. Live
/// SSH output then streams over this channel instead of the Tauri event system,
/// which stalls under the high-rate emit burst of a full-screen redraw on
/// WebView2/Windows. The most recently registered channel wins.
#[tauri::command]
pub async fn ssh_register_terminal_channel(
    channel: tauri::ipc::Channel<serde_json::Value>,
    state: State<'_, AppState>,
) -> AppResult<()> {
    if let Ok(mut guard) = state.terminal_channel.lock() {
        *guard = Some(channel);
    }
    Ok(())
}

#[tauri::command]
pub async fn ssh_session_resize(
    input: SshResizeInput,
    state: State<'_, AppState>,
) -> AppResult<SshSessionEvent> {
    trace_command("ssh_session_resize", state.command_bus.resize_ssh_session(input)).await
}

#[tauri::command]
pub async fn ssh_session_close(
    input: SshCloseInput,
    state: State<'_, AppState>,
) -> AppResult<SshSessionSummary> {
    trace_command("ssh_session_close", state.command_bus.close_ssh_session(input)).await
}

#[tauri::command]
pub async fn ssh_session_reconnect_cancel(
    input: SshReconnectCancelInput,
    state: State<'_, AppState>,
) -> AppResult<SshSessionSummary> {
    state.command_bus.cancel_ssh_reconnect(input).await
}

#[tauri::command]
pub async fn ssh_session_log_export(
    input: SshLogExportInput,
    state: State<'_, AppState>,
) -> AppResult<SshLogExport> {
    state.command_bus.export_ssh_log(input).await
}

#[tauri::command]
pub async fn ssh_host_key_get(
    input: SshHostKeyInput,
    state: State<'_, AppState>,
) -> AppResult<Option<SshHostFingerprintInfo>> {
    state.command_bus.get_ssh_host_fingerprint(input).await
}

#[tauri::command]
pub async fn ssh_host_key_reset(
    input: SshHostKeyInput,
    state: State<'_, AppState>,
) -> AppResult<bool> {
    state.command_bus.reset_ssh_host_fingerprint(input).await
}

#[tauri::command]
pub async fn ssh_host_key_list(
    workspace_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<SshHostFingerprintInfo>> {
    state
        .command_bus
        .list_all_ssh_fingerprints(workspace_id)
        .await
}

#[tauri::command]
pub async fn ssh_known_hosts_import(
    input: SshKnownHostsImportInput,
    state: State<'_, AppState>,
) -> AppResult<SshKnownHostsImportResult> {
    state.command_bus.import_ssh_known_hosts(input).await
}

#[tauri::command]
pub async fn ssh_known_hosts_export(
    input: SshKnownHostsExportInput,
    state: State<'_, AppState>,
) -> AppResult<SshKnownHostsExportResult> {
    state.command_bus.export_ssh_known_hosts(input).await
}
