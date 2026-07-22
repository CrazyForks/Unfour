import type { SshTask } from "@unfour/command-client";
import { useMemo, useState } from "react";
import {
  Button,
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
  IconButton,
  Input,
  LoadingState,
  SidebarSection,
  useI18n,
} from "@unfour/ui";
import { Play, Plus, Workflow } from "lucide-react";
import { SshSidebarModeSwitcher } from "./SshSidebarModeSwitcher";

export function TaskList({
  loading,
  onDelete,
  onDuplicate,
  onExample,
  onNew,
  onOpenConnections,
  onRun,
  onSelect,
  selectedTaskId,
  tasks,
}: {
  loading: boolean;
  onDelete: (task: SshTask) => void;
  onDuplicate: (task: SshTask) => void;
  onExample: () => void;
  onNew: () => void;
  onOpenConnections: () => void;
  onRun: (task: SshTask) => void;
  onSelect: (taskId: string) => void;
  selectedTaskId: string | null;
  tasks: SshTask[];
}) {
  const { t } = useI18n();
  const [filter, setFilter] = useState("");
  const filtered = useMemo(() => {
    const query = filter.trim().toLowerCase();
    if (!query) return tasks;
    return tasks.filter(
      (task) =>
        task.name.toLowerCase().includes(query) ||
        task.description.toLowerCase().includes(query),
    );
  }, [filter, tasks]);

  return (
    <SidebarSection className="flex h-full min-h-0 flex-col">
      <div className="flex h-7 shrink-0 items-center justify-between gap-2 px-1">
        <SshSidebarModeSwitcher
          activeMode="tasks"
          onChange={(mode) => mode === "connections" && onOpenConnections()}
        />
        <div className="flex items-center gap-0.5">
          <IconButton
            label={t("ssh.tasks.actions.new")}
            onClick={() => onNew()}
            size="compact"
          >
            <Plus size={14} />
          </IconButton>
        </div>
      </div>
      {loading ? (
        <LoadingState className="min-h-0 flex-1 rounded-none border-0">
          {t("ssh.tasks.list.loading")}
        </LoadingState>
      ) : tasks.length ? (
        <>
          <div className="px-1 pb-1">
            <Input
              aria-label={t("ssh.tasks.list.filter")}
              className="h-7"
              onChange={(event) => setFilter(event.target.value)}
              placeholder={t("ssh.tasks.list.filterPlaceholder")}
              value={filter}
            />
          </div>
          {filtered.length ? (
            <div className="min-h-0 flex-1 overflow-y-auto py-1">
              {filtered.map((task) => {
                const selected = selectedTaskId === task.id;
                return (
                  <ContextMenu key={task.id}>
                    <ContextMenuTrigger asChild>
                      <div
                        className={`group flex min-h-9 items-center gap-1 rounded-[var(--u-radius-sm)] px-1.5 transition-colors ${
                          selected
                            ? "bg-[var(--u-color-surface-active)] text-[var(--u-color-text)]"
                            : "text-[var(--u-color-text-muted)] hover:bg-[var(--u-color-surface-hover)] hover:text-[var(--u-color-text)]"
                        }`}
                      >
                        <button
                          className="min-w-0 flex-1 cursor-pointer py-0.5 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--u-color-focus)]"
                          onClick={() => onSelect(task.id)}
                          type="button"
                        >
                          <span className="block truncate text-[12px] font-medium leading-4">
                            {task.name}
                          </span>
                          <span className="block truncate text-[10px] leading-3 text-[var(--u-color-text-soft)]">
                            {task.description || t("ssh.tasks.list.noDescription")}
                          </span>
                        </button>
                        <div
                          className={`flex shrink-0 transition-opacity ${
                            selected
                              ? "opacity-100"
                              : "opacity-0 group-hover:opacity-100 group-focus-within:opacity-100"
                          }`}
                        >
                          <IconButton
                            disableTooltip
                            label={t("ssh.tasks.actions.run")}
                            onClick={() => onRun(task)}
                            size="compact"
                          >
                            <Play size={12} />
                          </IconButton>
                        </div>
                      </div>
                    </ContextMenuTrigger>
                    <ContextMenuContent>
                      <ContextMenuItem onSelect={() => onRun(task)}>
                        {t("ssh.tasks.actions.run")}
                      </ContextMenuItem>
                      <ContextMenuItem onSelect={() => onSelect(task.id)}>
                        {t("ssh.tasks.actions.open")}
                      </ContextMenuItem>
                      <ContextMenuItem onSelect={() => onDuplicate(task)}>
                        {t("ssh.tasks.actions.duplicate")}
                      </ContextMenuItem>
                      <ContextMenuSeparator />
                      <ContextMenuItem onSelect={() => onDelete(task)} tone="danger">
                        {t("ssh.tasks.actions.delete")}
                      </ContextMenuItem>
                    </ContextMenuContent>
                  </ContextMenu>
                );
              })}
            </div>
          ) : (
            <div className="flex min-h-0 flex-1 items-center justify-center px-3 text-center text-[12px] text-[var(--u-color-text-muted)]">
              {t("ssh.tasks.list.filterEmpty")}
            </div>
          )}
        </>
      ) : (
        <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 p-4 text-center">
          <Workflow className="text-[var(--u-color-text-soft)]" size={24} />
          <div>
            <p className="text-[13px] font-medium text-[var(--u-color-text)]">
              {t("ssh.tasks.list.emptyTitle")}
            </p>
            <p className="mt-1 text-[12px] text-[var(--u-color-text-muted)]">
              {t("ssh.tasks.list.emptyDescription")}
            </p>
          </div>
          <Button onClick={() => onNew()} size="sm">
            {t("ssh.tasks.actions.new")}
          </Button>
          <Button onClick={onExample} size="sm" variant="secondary">
            {t("ssh.tasks.actions.dockerExample")}
          </Button>
        </div>
      )}
    </SidebarSection>
  );
}
