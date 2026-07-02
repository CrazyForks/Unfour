-- Split the polymorphic `connections` table into typed subtype tables
-- (architecture decision 2026-07-02). The current single-table design uses
-- `kind` + `config_json` to host SSH and database connections together,
-- which loses column-level constraints and forces every read to branch in
-- SQL. Each engine service already hardcodes `kind = 'ssh'` or
-- `kind = 'database'` in its queries, so the split has no behavioral impact
-- on callers; it only makes the per-kind schema explicit.
--
-- Layout after this migration:
--   connections                  parent row: identity + sync + credential_ref
--     ssh_connections            subtype, 1:1 with connections.id, holds SSH config_json
--     database_connections       subtype, 1:1 with connections.id, holds DB config_json
--
-- The parent keeps `credential_ref` because it is shared identity metadata;
-- the kind-specific config_json moves into the subtype row. The legacy
-- `idx_connections_workspace_kind` index is replaced by a workspace-only
-- index since `kind` no longer exists.
--
-- Safe table-rebuild pattern is used because SQLite ALTER TABLE cannot drop
-- columns portably across supported versions.

-- 1. Create subtype tables.

CREATE TABLE IF NOT EXISTS ssh_connections (
  connection_id TEXT PRIMARY KEY,
  config_json TEXT NOT NULL,
  FOREIGN KEY(connection_id) REFERENCES connections(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS database_connections (
  connection_id TEXT PRIMARY KEY,
  config_json TEXT NOT NULL,
  FOREIGN KEY(connection_id) REFERENCES connections(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ssh_connections_connection
ON ssh_connections(connection_id);

CREATE INDEX IF NOT EXISTS idx_database_connections_connection
ON database_connections(connection_id);

-- 2. Backfill subtype rows from the existing polymorphic parent.

INSERT INTO ssh_connections (connection_id, config_json)
SELECT id, config_json
FROM connections
WHERE kind = 'ssh';

INSERT INTO database_connections (connection_id, config_json)
SELECT id, config_json
FROM connections
WHERE kind = 'database';

-- 3. Rebuild the parent table without kind / config_json.

CREATE TABLE IF NOT EXISTS connections_new (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT NOT NULL,
  credential_ref TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  sync_status TEXT NOT NULL DEFAULT 'local',
  remote_id TEXT,
  FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
);

INSERT INTO connections_new (
  id, workspace_id, name, credential_ref,
  created_at, updated_at, deleted_at, revision, sync_status, remote_id
)
SELECT
  id, workspace_id, name, credential_ref,
  created_at, updated_at, deleted_at, revision, sync_status, remote_id
FROM connections;

DROP TABLE connections;

ALTER TABLE connections_new RENAME TO connections;

-- 4. Replace the legacy (workspace_id, kind) index with a workspace-only index.

CREATE INDEX IF NOT EXISTS idx_connections_workspace
ON connections(workspace_id, deleted_at);
