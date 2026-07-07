-- Guard api_collection_folders against parent cycles (A -> B -> A).
--
-- The base schema only blocks a direct self-reference
-- (CHECK(parent_folder_id IS NULL OR parent_folder_id <> id)). A deeper cycle
-- (folder moved under one of its own descendants) is still possible and would
-- break tree traversal. These triggers walk the ancestor chain of the proposed
-- parent and abort the write if the row being modified appears in it.
--
-- Separate timestamped core migration on purpose: it must not alter the
-- initial table definitions, so it stays independent of any in-flight edits to
-- engine code.

CREATE TRIGGER IF NOT EXISTS trg_api_collection_folders_no_cycle_insert
BEFORE INSERT ON api_collection_folders
WHEN NEW.parent_folder_id IS NOT NULL
BEGIN
  WITH RECURSIVE ancestors(ancestor_id) AS (
    SELECT NEW.parent_folder_id
    UNION ALL
    SELECT f.parent_folder_id
    FROM api_collection_folders f
    JOIN ancestors ON f.id = ancestors.ancestor_id
    WHERE f.parent_folder_id IS NOT NULL
  )
  SELECT CASE
    WHEN EXISTS(SELECT 1 FROM ancestors WHERE ancestor_id = NEW.id)
    THEN RAISE(ABORT, 'api_collection_folders would create a cycle')
  END;
END;

CREATE TRIGGER IF NOT EXISTS trg_api_collection_folders_no_cycle_update
BEFORE UPDATE OF parent_folder_id ON api_collection_folders
WHEN NEW.parent_folder_id IS NOT NULL
BEGIN
  WITH RECURSIVE ancestors(ancestor_id) AS (
    SELECT NEW.parent_folder_id
    UNION ALL
    SELECT f.parent_folder_id
    FROM api_collection_folders f
    JOIN ancestors ON f.id = ancestors.ancestor_id
    WHERE f.parent_folder_id IS NOT NULL
  )
  SELECT CASE
    WHEN EXISTS(SELECT 1 FROM ancestors WHERE ancestor_id = NEW.id)
    THEN RAISE(ABORT, 'api_collection_folders would create a cycle')
  END;
END;
