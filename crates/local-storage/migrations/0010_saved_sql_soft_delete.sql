-- saved_sql currently uses hard DELETE. Convert it to the soft-delete pattern
-- used by api_requests / api_collections / connections so future cloud sync
-- can push tombstones. Also add the reserved sync fields (revision /
-- sync_status / remote_id) so saved_sql follows the same local-record
-- convention as other workspace-scoped business data.
--
-- ALTER TABLE ADD COLUMN is safe here; existing rows pick up defaults
-- (deleted_at NULL, revision 1, sync_status 'local', remote_id NULL).

ALTER TABLE saved_sql
ADD COLUMN deleted_at TEXT;

ALTER TABLE saved_sql
ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;

ALTER TABLE saved_sql
ADD COLUMN sync_status TEXT NOT NULL DEFAULT 'local';

ALTER TABLE saved_sql
ADD COLUMN remote_id TEXT;
