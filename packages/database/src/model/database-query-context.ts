import type { DatabaseTreeModel } from "./database-tree";
import type { DatabaseQueryWorkspaceTab } from "./types";

export function normalizeQueryContext(
  current: Pick<DatabaseQueryWorkspaceTab, "catalog" | "schema">,
  treeModel: DatabaseTreeModel,
) {
  const currentCatalog = treeModel.catalogs.find((catalog) => catalog.key === (current.catalog ?? ""));
  const fallbackCatalog = currentCatalog ?? treeModel.catalogs[0];
  if (!fallbackCatalog) {
    return { catalog: null, schema: null };
  }

  const catalog = fallbackCatalog.key || null;
  if (!fallbackCatalog.hasSchemaLevel) {
    return { catalog, schema: null };
  }

  const currentSchema = fallbackCatalog.schemas.find((schema) => schema.key === (current.schema ?? ""));
  const fallbackSchema = currentSchema ?? fallbackCatalog.schemas[0];
  return {
    catalog,
    schema: fallbackSchema?.key || null,
  };
}

