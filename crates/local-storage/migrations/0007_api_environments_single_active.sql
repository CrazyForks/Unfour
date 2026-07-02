-- Enforce a single active environment per workspace at the storage layer
-- (architecture decision 2026-07-02, Option B). The application layer in
-- http-engine already wraps the activate/deactivate updates in one
-- transaction; this partial unique index is defense-in-depth so that a
-- future bug that bypasses that transaction cannot leave two active rows.
--
-- Partial index: only one row per workspace may carry is_active = 1 while
-- not soft-deleted. Soft-deleted rows are excluded so re-activating after a
-- delete does not collide with a tombstoned predecessor.

CREATE UNIQUE INDEX IF NOT EXISTS uq_api_environments_active_per_workspace
ON api_environments(workspace_id)
WHERE is_active = 1 AND deleted_at IS NULL;
