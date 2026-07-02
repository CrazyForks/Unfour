-- Drop legacy dead columns superseded by migration 0004. After 0004 ran,
-- `api_requests.folder_path` was replaced by `api_requests.parent_folder_id`
-- and `api_collections.folders_json` was replaced by the
-- `api_collection_folders` table. Both legacy columns are no longer read by
-- any Rust code; keeping them invites future code to read stale data.
--
-- SQLite ALTER TABLE DROP COLUMN is supported on 3.35.0+ (2021-03), which is
-- well below the bundled SQLite version Tauri ships on all supported
-- platforms. Use the simple DROP form rather than a full table rebuild.

ALTER TABLE api_requests DROP COLUMN folder_path;

ALTER TABLE api_collections DROP COLUMN folders_json;
