import { Copy, Download, MoreHorizontal, Trash2, Upload } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  useI18n,
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
  const { t } = useI18n();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <IconButton label={t("api.actions.requestActions")} tooltip={t("api.actions.requestActions")}>
          <MoreHorizontal size={16} />
        </IconButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem disabled={!canDuplicate} onSelect={onDuplicate}>
          <Copy size={14} /> {t("api.actions.duplicate")}
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={onImport}>
          <Upload size={14} /> {t("api.actions.importCollection")}
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={onExport}>
          <Download size={14} /> {t("api.actions.exportCollection")}
        </DropdownMenuItem>
        <DropdownMenuItem
          className="text-[var(--u-color-danger)]"
          disabled={!canDelete}
          onSelect={onDelete}
        >
          <Trash2 size={14} /> {t("api.actions.deleteRequest")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
