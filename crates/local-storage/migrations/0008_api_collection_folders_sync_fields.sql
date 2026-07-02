-- api_collection_folders was created without the reserved sync fields, while
-- its parent table api_collections has them. This is an inconsistency with
-- the project's own "reserve sync fields from the beginning" rule (see
-- docs/architecture/data-storage.md). Retrofit the three missing columns so
-- future cloud sync can push folder tombstones and revisions.
--
-- ALTER TABLE ADD COLUMN is safe on SQLite for these nullable / defaulted
-- columns and preserves existing rows.

ALTER TABLE api_collection_folders
ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;

ALTER TABLE api_collection_folders
ADD COLUMN sync_status TEXT NOT NULL DEFAULT 'local';

ALTER TABLE api_collection_folders
ADD COLUMN remote_id TEXT;
