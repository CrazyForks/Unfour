-- Add indexes for hot query paths that currently scan tables.
--
-- `activity_events.list_recent` orders by (workspace_id, created_at DESC,
--   id DESC) but had no covering index. activity_events only grows, so this
--   is the highest-value addition.
-- `workspaces` recent-workspace listing filters deleted_at IS NULL and
--   orders by last_opened_at DESC.
-- `api_requests` workspace-level listing (e.g. "all saved requests in this
--   workspace not in a folder") has no index that starts at workspace_id;
--   the existing collection-parent index does not cover it.
-- `api_environments` listing filters (workspace_id, deleted_at); today's
--   index only covers workspace_id.
-- `saved_sql` listing now filters deleted_at IS NULL after the soft-delete
--   retrofit; extend the existing index to cover the filter.

CREATE INDEX IF NOT EXISTS idx_activity_events_workspace_created_id
ON activity_events(workspace_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_activity_events_created_id
ON activity_events(created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_workspaces_recent
ON workspaces(deleted_at, last_opened_at DESC);

CREATE INDEX IF NOT EXISTS idx_api_requests_workspace_deleted
ON api_requests(workspace_id, deleted_at);

CREATE INDEX IF NOT EXISTS idx_api_environments_workspace_deleted
ON api_environments(workspace_id, deleted_at);

DROP INDEX IF EXISTS idx_saved_sql_workspace_updated;
CREATE INDEX IF NOT EXISTS idx_saved_sql_workspace_updated
ON saved_sql(workspace_id, deleted_at, updated_at DESC);
