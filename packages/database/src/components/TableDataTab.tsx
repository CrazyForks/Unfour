import type { DatabaseQueryResult } from "@unfour/command-client";
import { EmptyState } from "@unfour/ui";
import { TableDataGrid } from "./TableDataGrid";

export function TableDataTab({ result }: { result: DatabaseQueryResult | null }) {
  if (!result) {
    return <EmptyState className="m-2 min-h-0 flex-1">Open a table from the schema tree to view rows.</EmptyState>;
  }

  return <TableDataGrid result={result} />;
}
