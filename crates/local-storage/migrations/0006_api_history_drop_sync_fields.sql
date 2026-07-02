-- api_history is a local-only request log. It does not participate in cloud
-- sync (per architecture decision 2026-07-02), so the reserved sync fields
-- (revision / sync_status / remote_id / deleted_at) are removed to match
-- `db_query_history`, which is the established local-log pattern.
--
-- SQLite's ALTER TABLE DROP COLUMN has build/runtime quirks across versions,
-- so this migration uses the safe table-rebuild pattern: create a new table
-- without the dropped columns, copy data, drop the old table, rename.

CREATE TABLE IF NOT EXISTS api_history_new (
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

INSERT INTO api_history_new (
  id, workspace_id, name, method, url,
  request_headers_json, request_query_json, request_body,
  status, duration_ms,
  response_headers_json, response_body_preview,
  created_at, updated_at
)
SELECT
  id, workspace_id, name, method, url,
  request_headers_json, request_query_json, request_body,
  status, duration_ms,
  response_headers_json, response_body_preview,
  created_at, updated_at
FROM api_history;

DROP TABLE api_history;

ALTER TABLE api_history_new RENAME TO api_history;

CREATE INDEX IF NOT EXISTS idx_api_history_workspace_created
ON api_history(workspace_id, created_at DESC);
