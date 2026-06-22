CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  is_default INTEGER NOT NULL DEFAULT 0,
  last_opened_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT
);

CREATE TABLE IF NOT EXISTS workspace_settings (
  workspace_id TEXT PRIMARY KEY,
  layout_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE IF NOT EXISTS api_requests (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT NOT NULL,
  method TEXT NOT NULL,
  url TEXT NOT NULL,
  headers_json TEXT NOT NULL DEFAULT '[]',
  query_json TEXT NOT NULL DEFAULT '[]',
  body TEXT,
  body_kind TEXT NOT NULL DEFAULT 'json',
  folder_path TEXT,
  collection_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE IF NOT EXISTS api_history (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT,
  method TEXT NOT NULL,
  url TEXT NOT NULL,
  request_headers_json TEXT NOT NULL DEFAULT '[]',
  request_query_json TEXT NOT NULL DEFAULT '[]',
  request_body TEXT,
  status INTEGER,
  duration_ms INTEGER,
  response_headers_json TEXT NOT NULL DEFAULT '[]',
  response_body_preview TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE IF NOT EXISTS db_query_history (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  connection_id TEXT,
  connection_name TEXT NOT NULL,
  sql TEXT NOT NULL,
  status TEXT NOT NULL,
  classification TEXT,
  row_count INTEGER,
  affected_rows INTEGER,
  duration_ms INTEGER,
  error TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE IF NOT EXISTS connections (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  config_json TEXT NOT NULL DEFAULT '{}',
  credential_ref TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE IF NOT EXISTS activity_events (
  id TEXT PRIMARY KEY,
  workspace_id TEXT,
  action TEXT NOT NULL,
  target TEXT,
  details_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_api_history_workspace_created ON api_history(workspace_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_db_query_history_workspace_created ON db_query_history(workspace_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_connections_workspace_kind ON connections(workspace_id, kind);

CREATE TABLE IF NOT EXISTS ssh_host_keys (
  host TEXT NOT NULL,
  port INTEGER NOT NULL,
  fingerprint TEXT NOT NULL,
  key_type TEXT,
  public_key_data TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY (host, port)
);

CREATE TABLE IF NOT EXISTS ssh_terminal_history (
  workspace_id TEXT NOT NULL,
  session_id TEXT NOT NULL,
  connection_id TEXT NOT NULL,
  status TEXT NOT NULL,
  reconnect_attempt INTEGER NOT NULL DEFAULT 0,
  auth_kind TEXT NOT NULL,
  host TEXT NOT NULL,
  username TEXT NOT NULL,
  cols INTEGER NOT NULL,
  rows INTEGER NOT NULL,
  content TEXT NOT NULL DEFAULT '',
  byte_len INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (workspace_id, session_id),
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE INDEX IF NOT EXISTS idx_ssh_terminal_history_workspace_updated ON ssh_terminal_history(workspace_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_ssh_terminal_history_connection ON ssh_terminal_history(workspace_id, connection_id);

CREATE TABLE IF NOT EXISTS api_environments (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT NOT NULL,
  variables_json TEXT NOT NULL DEFAULT '[]',
  is_active INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE INDEX IF NOT EXISTS idx_api_environments_workspace ON api_environments(workspace_id);

CREATE TABLE IF NOT EXISTS api_collections (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  folders_json TEXT NOT NULL DEFAULT '[]',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE INDEX IF NOT EXISTS idx_api_collections_workspace ON api_collections(workspace_id);
