use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub last_opened_at: Option<String>,
    pub environment_type: String,
    pub mcp_policy: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub revision: i64,
    pub sync_status: String,
    pub remote_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceState {
    pub active_workspace_id: String,
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceVariable {
    pub id: String,
    pub workspace_id: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub is_enabled: bool,
    pub description: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub revision: i64,
    pub sync_status: String,
    pub remote_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEnvironmentVariable {
    pub id: String,
    pub workspace_id: String,
    pub environment_id: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub is_enabled: bool,
    pub description: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub revision: i64,
    pub sync_status: String,
    pub remote_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEnvironment {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub sort_order: i64,
    pub is_active: bool,
    pub variables: Vec<WorkspaceEnvironmentVariable>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub revision: i64,
    pub sync_status: String,
    pub remote_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceVariableInput {
    #[serde(default)]
    pub id: Option<String>,
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub is_secret: bool,
    #[serde(default = "default_enabled")]
    pub is_enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: i64,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceLayout {
    pub workspace_id: String,
    pub sidebar_collapsed: bool,
    pub active_tab_id: String,
    pub tabs: Vec<WorkspaceLayoutTab>,
    pub selected_api_request_id: Option<String>,
    pub selected_database_connection_id: Option<String>,
    pub selected_ssh_connection_id: Option<String>,
    #[serde(default)]
    pub sidebar_width: i32,
    #[serde(default)]
    pub bottom_panel_height: i32,
    #[serde(default)]
    pub right_inspector_width: i32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceLayoutTab {
    pub id: String,
    pub title: String,
    pub kind: String,
}
