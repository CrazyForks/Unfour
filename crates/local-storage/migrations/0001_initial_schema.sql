CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
  last_opened_at TEXT,
  environment_type TEXT NOT NULL DEFAULT 'dev'
    CHECK (environment_type IN ('dev', 'test', 'prod')),
  mcp_policy TEXT NOT NULL DEFAULT 'auto'
    CHECK (mcp_policy IN ('auto', 'disabled', 'read_only', 'guarded', 'full_access')),
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
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS connections (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  connection_type TEXT NOT NULL CHECK (connection_type IN ('database', 'ssh')),
  name TEXT NOT NULL,
  host TEXT,
  port INTEGER CHECK (port IS NULL OR (port BETWEEN 1 AND 65535)),
  credential_ref TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_connected_at TEXT,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  UNIQUE(workspace_id, id),
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE IF NOT EXISTS ssh_connections (
  connection_id TEXT PRIMARY KEY,
  username TEXT NOT NULL,
  auth_method TEXT NOT NULL CHECK (auth_method IN ('password', 'private-key', 'none')),
  config_json TEXT NOT NULL DEFAULT '{}',
  FOREIGN KEY(connection_id) REFERENCES connections(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS database_connections (
  connection_id TEXT PRIMARY KEY,
  driver TEXT NOT NULL CHECK (driver IN ('sqlite', 'postgres', 'mysql')),
  database_name TEXT,
  username TEXT,
  ssl_mode TEXT CHECK (
    ssl_mode IS NULL OR ssl_mode IN ('disable', 'prefer', 'require', 'verify-ca', 'verify-full')
  ),
  read_only INTEGER NOT NULL DEFAULT 0 CHECK (read_only IN (0, 1)),
  config_json TEXT NOT NULL DEFAULT '{}',
  FOREIGN KEY(connection_id) REFERENCES connections(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ssh_host_keys (
  workspace_id TEXT NOT NULL,
  host TEXT NOT NULL,
  port INTEGER NOT NULL,
  fingerprint TEXT NOT NULL,
  key_type TEXT,
  public_key_data TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY (workspace_id, host, port),
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
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
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
  FOREIGN KEY(connection_id) REFERENCES connections(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS api_collections (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  UNIQUE(workspace_id, id),
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE IF NOT EXISTS api_collection_folders (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  collection_id TEXT NOT NULL,
  parent_folder_id TEXT,
  name TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  UNIQUE(workspace_id, id),
  UNIQUE(workspace_id, collection_id, id),
  CHECK(parent_folder_id IS NULL OR parent_folder_id <> id),
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id),
  FOREIGN KEY(workspace_id, collection_id) REFERENCES api_collections(workspace_id, id),
  FOREIGN KEY(workspace_id, collection_id, parent_folder_id)
    REFERENCES api_collection_folders(workspace_id, collection_id, id)
);

CREATE TABLE IF NOT EXISTS api_requests (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT NOT NULL,
  collection_id TEXT NOT NULL,
  parent_folder_id TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  auth_json TEXT NOT NULL DEFAULT '{"type":"none"}',
  method TEXT NOT NULL,
  url TEXT NOT NULL,
  headers_json TEXT NOT NULL DEFAULT '[]',
  query_json TEXT NOT NULL DEFAULT '[]',
  body TEXT,
  body_kind TEXT NOT NULL DEFAULT 'json',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id),
  FOREIGN KEY(workspace_id, collection_id) REFERENCES api_collections(workspace_id, id),
  FOREIGN KEY(workspace_id, collection_id, parent_folder_id)
    REFERENCES api_collection_folders(workspace_id, collection_id, id)
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
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id),
  FOREIGN KEY(connection_id) REFERENCES connections(id) ON DELETE SET NULL
);

CREATE TRIGGER IF NOT EXISTS trg_db_query_history_connection_workspace_insert
BEFORE INSERT ON db_query_history
WHEN NEW.connection_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM connections
    WHERE id = NEW.connection_id
      AND workspace_id = NEW.workspace_id
  )
BEGIN
  SELECT RAISE(ABORT, 'db_query_history connection must belong to the same workspace');
END;

CREATE TRIGGER IF NOT EXISTS trg_db_query_history_connection_workspace_update
BEFORE UPDATE OF workspace_id, connection_id ON db_query_history
WHEN NEW.connection_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM connections
    WHERE id = NEW.connection_id
      AND workspace_id = NEW.workspace_id
  )
BEGIN
  SELECT RAISE(ABORT, 'db_query_history connection must belong to the same workspace');
END;

CREATE TABLE IF NOT EXISTS api_environments (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT NOT NULL,
  variables_json TEXT NOT NULL DEFAULT '[]',
  is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE IF NOT EXISTS saved_sql (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  connection_id TEXT,
  name TEXT NOT NULL,
  sql TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id),
  FOREIGN KEY(connection_id) REFERENCES connections(id) ON DELETE SET NULL
);

CREATE TRIGGER IF NOT EXISTS trg_saved_sql_connection_workspace_insert
BEFORE INSERT ON saved_sql
WHEN NEW.connection_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM connections
    WHERE id = NEW.connection_id
      AND workspace_id = NEW.workspace_id
      AND deleted_at IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'saved_sql connection must belong to the same workspace');
END;

CREATE TRIGGER IF NOT EXISTS trg_saved_sql_connection_workspace_update
BEFORE UPDATE OF workspace_id, connection_id ON saved_sql
WHEN NEW.connection_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM connections
    WHERE id = NEW.connection_id
      AND workspace_id = NEW.workspace_id
      AND deleted_at IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'saved_sql connection must belong to the same workspace');
END;

CREATE TRIGGER IF NOT EXISTS trg_ssh_terminal_history_connection_workspace_insert
BEFORE INSERT ON ssh_terminal_history
WHEN NOT EXISTS (
  SELECT 1
  FROM connections
  WHERE id = NEW.connection_id
    AND workspace_id = NEW.workspace_id
)
BEGIN
  SELECT RAISE(ABORT, 'ssh terminal history connection must belong to the same workspace');
END;

CREATE TRIGGER IF NOT EXISTS trg_ssh_terminal_history_connection_workspace_update
BEFORE UPDATE OF workspace_id, connection_id ON ssh_terminal_history
WHEN NOT EXISTS (
  SELECT 1
  FROM connections
  WHERE id = NEW.connection_id
    AND workspace_id = NEW.workspace_id
)
BEGIN
  SELECT RAISE(ABORT, 'ssh terminal history connection must belong to the same workspace');
END;

CREATE TABLE IF NOT EXISTS activity_events (
  id TEXT PRIMARY KEY,
  workspace_id TEXT,
  action TEXT NOT NULL,
  target TEXT,
  details_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

CREATE INDEX IF NOT EXISTS idx_activity_events_workspace_created_id
ON activity_events(workspace_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_activity_events_created_id
ON activity_events(created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_api_collection_folders_collection
ON api_collection_folders(workspace_id, collection_id, deleted_at, sort_order);

CREATE INDEX IF NOT EXISTS idx_api_collection_folders_parent
ON api_collection_folders(workspace_id, collection_id, parent_folder_id, deleted_at, sort_order);

CREATE INDEX IF NOT EXISTS idx_api_environments_workspace_deleted
ON api_environments(workspace_id, deleted_at);

CREATE UNIQUE INDEX IF NOT EXISTS uq_api_environments_active_per_workspace
ON api_environments(workspace_id)
WHERE is_active = 1 AND deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uq_api_environments_name_per_workspace
ON api_environments(workspace_id, name COLLATE NOCASE)
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_api_history_workspace_created
ON api_history(workspace_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_api_requests_workspace_tree
ON api_requests(workspace_id, deleted_at, collection_id, parent_folder_id, sort_order, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_api_requests_collection_parent
ON api_requests(workspace_id, collection_id, parent_folder_id, deleted_at, sort_order);

CREATE INDEX IF NOT EXISTS idx_api_collections_workspace
ON api_collections(workspace_id, deleted_at);

CREATE INDEX IF NOT EXISTS idx_connections_workspace
ON connections(workspace_id, deleted_at);

CREATE INDEX IF NOT EXISTS idx_connections_workspace_type
ON connections(workspace_id, connection_type, deleted_at);

CREATE INDEX IF NOT EXISTS idx_database_connections_connection
ON database_connections(connection_id);

CREATE INDEX IF NOT EXISTS idx_db_query_history_workspace_created
ON db_query_history(workspace_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_saved_sql_workspace_updated
ON saved_sql(workspace_id, deleted_at, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_ssh_connections_connection
ON ssh_connections(connection_id);

CREATE INDEX IF NOT EXISTS idx_ssh_terminal_history_workspace_updated
ON ssh_terminal_history(workspace_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_ssh_terminal_history_connection
ON ssh_terminal_history(workspace_id, connection_id);

CREATE INDEX IF NOT EXISTS idx_workspaces_recent
ON workspaces(deleted_at, is_default DESC, last_opened_at DESC, created_at ASC);
