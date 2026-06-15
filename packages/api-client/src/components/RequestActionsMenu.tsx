import { Copy, Download, MoreHorizontal, Trash2, Upload } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from "@unfour/ui";

export function RequestActionsMenu({
  canDelete,
  canDuplicate,
  onDelete,
  onDuplicate,
  onExport,
  onImport,
}: {
  canDelete: boolean;
  canDuplicate: boolean;
  onDelete: () => void;
  onDuplicate: () => void;
  onExport: () => void;
  onImport: () => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <IconButton label="Request actions" tooltip="Request actions">
          <MoreHorizontal size={16} />
        </IconButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem disabled={!canDuplicate} onSelect={onDuplicate}>
          <Copy size={14} /> Duplicate
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={onImport}>
          <Upload size={14} /> Import collection
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={onExport}>
          <Download size={14} /> Export collection
        </DropdownMenuItem>
        <DropdownMenuItem
          className="text-[var(--u-color-danger)]"
          disabled={!canDelete}
          onSelect={onDelete}
        >
          <Trash2 size={14} /> Delete request
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
