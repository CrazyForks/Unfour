import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import {
  ChevronDown,
  Folder,
  Pencil,
  Plus,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import type { Workspace } from "@unfour/command-client";
import { Badge, Button, cn } from "@unfour/ui";
import { WorkspaceDialogs } from "./WorkspaceDialogs";

export function WorkspaceMenu({
  activeWorkspace,
  onActivateWorkspace,
  workspaces,
}: {
  activeWorkspace?: Workspace;
  onActivateWorkspace: (workspaceId: string) => void;
  workspaces: Workspace[];
}) {
  const [createOpen, setCreateOpen] = useState(false);
  const [renameOpen, setRenameOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);

  return (
    <>
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <Button
            className="ml-2 max-w-[240px] justify-start gap-1 border-transparent bg-white px-2 font-semibold shadow-none hover:bg-slate-100"
            size="sm"
            type="button"
            variant="outline"
          >
            <span className="min-w-0 truncate">
              {activeWorkspace?.name ?? "No workspace"}
            </span>
            <ChevronDown className="shrink-0 text-slate-500" size={14} />
          </Button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            align="start"
            className="z-50 w-72 rounded-md border border-slate-200 bg-white p-1 text-sm text-slate-800 shadow-xl"
            sideOffset={6}
          >
            <DropdownMenu.Label className="px-2 py-1.5 text-xs font-semibold uppercase text-slate-500">
              Workspaces
            </DropdownMenu.Label>
            {workspaces.map((workspace) => (
              <DropdownMenu.Item
                className={cn(
                  "flex min-h-8 cursor-pointer items-center gap-2 rounded px-2 py-1.5 outline-none hover:bg-slate-100 focus:bg-slate-100",
                  activeWorkspace?.id === workspace.id && "bg-teal-50 text-teal-900",
                )}
                key={workspace.id}
                onSelect={() => onActivateWorkspace(workspace.id)}
              >
                <Folder size={14} />
                <span className="min-w-0 flex-1 truncate">{workspace.name}</span>
                {workspace.isDefault && <Badge tone="teal">default</Badge>}
              </DropdownMenu.Item>
            ))}
            {workspaces.length === 0 && (
              <div className="px-2 py-4 text-center text-xs text-slate-500">
                No workspaces
              </div>
            )}
            <DropdownMenu.Separator className="my-1 h-px bg-slate-200" />
            <DropdownMenu.Item
              className="flex h-8 cursor-pointer items-center gap-2 rounded px-2 outline-none hover:bg-slate-100 focus:bg-slate-100"
              onSelect={() => setCreateOpen(true)}
            >
              <Plus size={14} />
              New workspace
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="flex h-8 cursor-pointer items-center gap-2 rounded px-2 outline-none hover:bg-slate-100 focus:bg-slate-100 disabled:pointer-events-none disabled:opacity-50"
              disabled={!activeWorkspace}
              onSelect={() => setRenameOpen(true)}
            >
              <Pencil size={14} />
              Rename current
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="flex h-8 cursor-pointer items-center gap-2 rounded px-2 text-rose-700 outline-none hover:bg-rose-50 focus:bg-rose-50 disabled:pointer-events-none disabled:opacity-50"
              disabled={!activeWorkspace || activeWorkspace.isDefault || workspaces.length <= 1}
              onSelect={() => setDeleteOpen(true)}
            >
              <Trash2 size={14} />
              Delete current
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>

      <WorkspaceDialogs
        activeWorkspace={activeWorkspace}
        createOpen={createOpen}
        deleteOpen={deleteOpen}
        onCreateClose={() => setCreateOpen(false)}
        onDeleteClose={() => setDeleteOpen(false)}
        onRenameClose={() => setRenameOpen(false)}
        renameOpen={renameOpen}
        workspaces={workspaces}
      />
    </>
  );
}
