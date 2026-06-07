import { useMutation } from "@tanstack/react-query";
import { browseDatabaseTable } from "@unfour/command-client";
import type { DatabaseBrowseResult } from "@unfour/command-client";

export function useTableData({
  connectionId,
  onBrowseStart,
  onSuccess,
  workspaceId,
}: {
  connectionId: string | null;
  onBrowseStart: () => void;
  onSuccess: (result: DatabaseBrowseResult) => void;
  workspaceId: string;
}) {
  return useMutation({
    onMutate: onBrowseStart,
    mutationFn: ({
      pageIndex,
      pageSize,
      tableName,
    }: {
      pageIndex: number;
      pageSize: number;
      tableName: string;
    }) =>
      browseDatabaseTable({
        workspaceId,
        connectionId: connectionId ?? "",
        tableName,
        limit: pageSize,
        offset: pageIndex * pageSize,
      }),
    onSuccess,
  });
}
