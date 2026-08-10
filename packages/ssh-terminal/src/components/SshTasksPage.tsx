import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  cancelSshTaskRun,
  clearSshTaskRuns,
  deleteSshTask,
  duplicateSshTask,
  getSshTask,
  listSshTaskRuns,
  listSshTasks,
  listWorkspaceEnvironments,
  listWorkspaceVariables,
  readSshTaskRunLog,
  registerSshTaskRunChannel,
  reorderSshTasks,
  runSshTask,
  saveSshTask,
  type SshConnection,
  type SshTask,
  type SshTaskDetail,
  type SshTaskRun,
  type SshTaskRunEvent,
  type SshTaskSaveInput,
  type WorkspaceEnvironment,
  type WorkspaceVariable,
} from "@unfour/command-client";
import {
  ConfirmDialog,
  ErrorState,
  SegmentedControl,
  SplitPane,
  Tabs,
  useFeedbackErrorHandler,
  useI18n,
} from "@unfour/ui";
import { TaskEditor } from "./TaskEditor";
import { TaskHistory } from "./TaskHistory";
import { TaskList } from "./TaskList";
import { TaskRunDialog } from "./TaskRunDialog";
import { TaskRunPanel } from "./TaskRunPanel";
import { TaskWorkspaceEmpty } from "./TaskWorkspaceEmpty";
import {
  detectTaskInputs,
  dockerImageExportTemplate,
  preferredTaskConnectionId,
  taskDetailToDraft,
} from "../model/task-template";
import {
  activeWorkspaceEnvironmentName,
  activeWorkspaceEnvironmentId,
  defaultTaskRunInputs,
  mergeWorkspaceVariables,
  workspaceEnvironmentById,
} from "../model/task-run-inputs";
import {
  closeTaskTab,
  createEmptyTaskEditorState,
  hydrateTaskTab,
  isTaskTabDirty,
  openNewTaskTab,
  openSavedTaskTab,
  persistTaskTab,
  removeTaskTabs,
  updateTaskTabDraft,
  updateTaskTabView,
  type SshTaskDetailView,
} from "../model/task-editor-tabs";

// eslint-disable-next-line max-lines-per-function -- coordinates task tabs, queries, mutations, sidebar content, and task-run dialogs in one page boundary
export function SshTasksPage({
  active = true,
  connections,
  onOpenConnections,
  onShellSidebarChange,
  workspaceId,
}: {
  active?: boolean;
  connections: SshConnection[];
  onOpenConnections: () => void;
  onShellSidebarChange?: (sidebar: ReactNode | null) => void;
  workspaceId: string;
}) {
  const { t } = useI18n();
  const queryClient = useQueryClient();
  const handleError = useFeedbackErrorHandler();
  const [editorState, setEditorState] = useState(createEmptyTaskEditorState);
  const [closeTabId, setCloseTabId] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<SshTask | null>(null);
  const [clearTaskId, setClearTaskId] = useState<string | null>(null);
  const [runDialogTask, setRunDialogTask] = useState<SshTaskDetail | null>(null);
  const [runConnectionId, setRunConnectionId] = useState("");
  const [runInputs, setRunInputs] = useState<Record<string, string>>({});
  const [runSecretInputs, setRunSecretInputs] = useState<string[]>([]);
  const [runFilledFromWorkspace, setRunFilledFromWorkspace] = useState(false);
  const [runEnvironmentId, setRunEnvironmentId] = useState("");
  const [runEnvironments, setRunEnvironments] = useState<WorkspaceEnvironment[]>([]);
  const [runWorkspaceVariables, setRunWorkspaceVariables] = useState<WorkspaceVariable[]>([]);
  const [runEnvironmentLoadFailed, setRunEnvironmentLoadFailed] = useState(false);
  const [runActiveEnvironmentName, setRunActiveEnvironmentName] = useState<string | null>(
    null,
  );
  const [activeRun, setActiveRun] = useState<SshTaskRun | null>(null);
  const [activeRunTask, setActiveRunTask] = useState<SshTaskDetail | null>(null);
  const [eventsByRun, setEventsByRun] = useState<Record<string, SshTaskRunEvent[]>>({});
  const [historyLogByRun, setHistoryLogByRun] = useState<Record<string, string>>({});
  const [historyLogLoading, setHistoryLogLoading] = useState(false);
  const eventsByRunRef = useRef(eventsByRun);
  const historyLogByRunRef = useRef(historyLogByRun);
  const tasksSurfaceActiveRef = useRef(active);
  eventsByRunRef.current = eventsByRun;
  historyLogByRunRef.current = historyLogByRun;
  tasksSurfaceActiveRef.current = active;
  const nextNewTabIdRef = useRef(0);

  const tasksQuery = useQuery({
    queryKey: ["ssh-tasks", workspaceId],
    queryFn: () => listSshTasks(workspaceId),
  });
  const tasks = useMemo(() => tasksQuery.data ?? [], [tasksQuery.data]);
  const activeTab =
    editorState.tabs.find((tab) => tab.id === editorState.activeTabId) ?? null;
  const activeTaskId = activeTab?.taskId ?? null;
  const draft = activeTab?.draft ?? null;
  const showHistory = activeTab?.view === "history";

  const detailQuery = useQuery({
    queryKey: ["ssh-task", workspaceId, activeTaskId],
    queryFn: () => getSshTask(workspaceId, activeTaskId!),
    enabled: Boolean(activeTaskId),
  });
  const runsQuery = useQuery({
    queryKey: ["ssh-task-runs", workspaceId, activeTaskId],
    queryFn: () => listSshTaskRuns(workspaceId, activeTaskId!),
    enabled: Boolean(activeTaskId),
  });

  useEffect(() => {
    const detail = detailQuery.data;
    if (!detail || !activeTab || activeTab.taskId !== detail.task.id || activeTab.draft) {
      return;
    }
    // eslint-disable-next-line react-hooks/set-state-in-effect -- React Query detail hydration seeds the matching editor tab without replacing other drafts
    setEditorState((current) => hydrateTaskTab(current, activeTab.id, taskDetailToDraft(detail)));
  }, [activeTab, detailQuery.data]);

  useEffect(() => {
    let disposed = false;
    let dispose: (() => void) | null = null;
    // Coalesce per-line task output into one React update ~per frame. Without
    // this, a verbose command (or transfer progress) re-renders thousands of
    // transcript spans and can freeze the Tasks surface.
    let pending: SshTaskRunEvent[] = [];
    let flushTimer: ReturnType<typeof setTimeout> | null = null;
    const flushPending = () => {
      flushTimer = null;
      if (!pending.length) return;
      // Keep buffering while Connections is shown so a hidden Tasks tree does
      // not re-render on every remote line; flush when Tasks becomes active.
      if (!tasksSurfaceActiveRef.current) {
        flushTimer = setTimeout(flushPending, 250);
        return;
      }
      const batch = pending;
      pending = [];
      setEventsByRun((current) => {
        const next = { ...current };
        for (const event of batch) {
          next[event.runId] = [...(next[event.runId] ?? []), event].slice(-5_000);
        }
        return next;
      });
      for (const event of batch) {
        if (event.kind === "run" && event.status && event.status !== "running") {
          queryClient.invalidateQueries({
            queryKey: ["ssh-task-runs", workspaceId, event.taskId],
          });
        }
      }
    };
    registerSshTaskRunChannel((event) => {
      pending.push(event);
      if (pending.length > 10_000) {
        pending = pending.slice(-10_000);
      }
      if (flushTimer === null) {
        flushTimer = setTimeout(flushPending, 16);
      }
    }).then((cleanup) => {
      if (disposed) cleanup();
      else dispose = cleanup;
    });
    return () => {
      disposed = true;
      if (flushTimer !== null) clearTimeout(flushTimer);
      flushPending();
      dispose?.();
    };
  }, [queryClient, workspaceId]);

  const saveMutation = useMutation({
    mutationFn: ({ draft: input }: { draft: SshTaskSaveInput; tabId: string }) =>
      saveSshTask(input),
    onSuccess: (detail, { tabId }) => {
      const savedDraft = taskDetailToDraft(detail);
      setEditorState((current) => persistTaskTab(current, tabId, savedDraft));
      queryClient.setQueryData(["ssh-task", workspaceId, detail.task.id], detail);
      queryClient.invalidateQueries({ queryKey: ["ssh-tasks", workspaceId] });
    },
    onError: (error) => handleError(error, { key: "feedback.ssh.taskSaveFailed" }),
  });
  const duplicateMutation = useMutation({
    mutationFn: (taskId: string) => duplicateSshTask(workspaceId, taskId),
    onSuccess: (detail) => {
      const savedDraft = taskDetailToDraft(detail);
      setEditorState((current) => {
        const opened = openSavedTaskTab(current, detail.task.id);
        return hydrateTaskTab(opened, opened.activeTabId!, savedDraft);
      });
      queryClient.setQueryData(["ssh-task", workspaceId, detail.task.id], detail);
      queryClient.invalidateQueries({ queryKey: ["ssh-tasks", workspaceId] });
    },
    onError: (error) => handleError(error, { key: "feedback.ssh.taskDuplicateFailed" }),
  });
  const deleteMutation = useMutation({
    mutationFn: (taskId: string) => deleteSshTask(workspaceId, taskId),
    onSuccess: (_, taskId) => {
      setEditorState((current) => removeTaskTabs(current, taskId));
      setDeleteTarget(null);
      queryClient.invalidateQueries({ queryKey: ["ssh-tasks", workspaceId] });
    },
    onError: (error) => handleError(error, { key: "feedback.ssh.taskDeleteFailed" }),
  });
  const reorderMutation = useMutation({
    mutationFn: (taskIds: string[]) => reorderSshTasks({ workspaceId, taskIds }),
    onMutate: async (taskIds) => {
      await queryClient.cancelQueries({ queryKey: ["ssh-tasks", workspaceId] });
      const previous = queryClient.getQueryData<SshTask[]>(["ssh-tasks", workspaceId]);
      const positions = new Map(taskIds.map((taskId, index) => [taskId, index]));
      queryClient.setQueryData<SshTask[]>(["ssh-tasks", workspaceId], (current) =>
        (current ?? [])
          .map((task) => ({
            ...task,
            sortOrder: positions.get(task.id) ?? task.sortOrder,
          }))
          .sort((left, right) => left.sortOrder - right.sortOrder),
      );
      return { previous };
    },
    onError: (error, _taskIds, context) => {
      if (context?.previous) {
        queryClient.setQueryData(["ssh-tasks", workspaceId], context.previous);
      }
      handleError(error, {
        message: t("ssh.tasks.list.reorderFailed"),
      });
    },
    onSuccess: (nextTasks) => {
      queryClient.setQueryData(["ssh-tasks", workspaceId], nextTasks);
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["ssh-tasks", workspaceId] });
    },
  });
  const runMutation = useMutation({
    mutationFn: () =>
      runSshTask({
        workspaceId,
        taskId: runDialogTask!.task.id,
        connectionId: runConnectionId || null,
        inputs: runInputs,
        secretInputNames: runSecretInputs,
      }),
    onSuccess: (run) => {
      setActiveRun(run);
      setActiveRunTask(runDialogTask);
      setRunDialogTask(null);
      queryClient.invalidateQueries({
        queryKey: ["ssh-task-runs", workspaceId, run.taskId],
      });
    },
  });
  const cancelMutation = useMutation({
    mutationFn: () => cancelSshTaskRun({ workspaceId, runId: activeRun!.id }),
    onError: (error) => handleError(error, { key: "feedback.ssh.taskCancelFailed" }),
  });
  const clearMutation = useMutation({
    mutationFn: (taskId: string) => clearSshTaskRuns({ workspaceId, taskId }),
    onSuccess: (_, taskId) => {
      setClearTaskId(null);
      setHistoryLogByRun({});
      queryClient.invalidateQueries({
        queryKey: ["ssh-task-runs", workspaceId, taskId],
      });
    },
    onError: (error) =>
      handleError(error, { key: "feedback.ssh.taskHistoryClearFailed" }),
  });
  const duplicateTask = duplicateMutation.mutate;
  const reorderTasks = reorderMutation.mutate;
  const resetRunMutation = runMutation.reset;
  const saveTask = saveMutation.mutateAsync;

  const newTask = useCallback(
    (template?: SshTaskSaveInput) => {
      const tabId = `new:${++nextNewTabIdRef.current}`;
      setEditorState((current) =>
        openNewTaskTab(
          current,
          tabId,
          template ?? {
            workspaceId,
            name: "",
            description: "",
            defaultConnectionId: null,
            steps: [],
          },
        ),
      );
    },
    [workspaceId],
  );

  const selectTask = useCallback((taskId: string) => {
    setEditorState((current) => openSavedTaskTab(current, taskId));
  }, []);

  const prepareRun = useCallback(
    async (taskId: string) => {
      try {
        const detail = await queryClient.fetchQuery({
          queryKey: ["ssh-task", workspaceId, taskId],
          queryFn: () => getSshTask(workspaceId, taskId),
        });
        const detectedInputs = detectTaskInputs(detail.steps, true);
        let inputs = Object.fromEntries(detectedInputs.map((name) => [name, ""]));
        let secretNames: string[] = [];
        let filledFromWorkspace = false;
        let activeEnvironmentName: string | null = null;
        let environmentId = "";
        let workspaceVariables: WorkspaceVariable[] = [];
        let environments: WorkspaceEnvironment[] = [];
        let environmentLoadFailed = false;

        try {
          [workspaceVariables, environments] = await Promise.all([
            queryClient.fetchQuery({
              queryKey: ["workspace-variables", workspaceId],
              queryFn: () => listWorkspaceVariables(workspaceId),
            }),
            queryClient.fetchQuery({
              queryKey: ["workspace-environments", workspaceId],
              queryFn: () => listWorkspaceEnvironments(workspaceId),
            }),
          ]);
          environmentId = activeWorkspaceEnvironmentId(environments);
          const defaults = defaultTaskRunInputs(
            detectedInputs,
            mergeWorkspaceVariables(
              workspaceVariables,
              workspaceEnvironmentById(environments, environmentId),
            ),
          );
          inputs = defaults.inputs;
          secretNames = defaults.secretNames;
          filledFromWorkspace = defaults.filledFromWorkspace.length > 0;
          activeEnvironmentName = activeWorkspaceEnvironmentName(environments);
        } catch {
          // Workspace defaults are optional; keep empty inputs if they fail to load.
          environmentLoadFailed = true;
        }

        setRunDialogTask(detail);
        setRunConnectionId(preferredTaskConnectionId(detail.localBinding));
        setRunInputs(inputs);
        setRunSecretInputs(secretNames);
        setRunFilledFromWorkspace(filledFromWorkspace);
        setRunActiveEnvironmentName(activeEnvironmentName);
        setRunEnvironmentId(environmentId);
        setRunEnvironments(environments);
        setRunWorkspaceVariables(workspaceVariables);
        setRunEnvironmentLoadFailed(environmentLoadFailed);
        resetRunMutation();
      } catch (error) {
        handleError(error, { key: "feedback.ssh.taskLoadFailed" });
      }
    },
    [handleError, queryClient, resetRunMutation, workspaceId],
  );

  const saveActiveDraft = useCallback(async () => {
    if (!activeTab?.draft) return null;
    if (!activeTab.draft.name.trim()) {
      handleError(new Error(t("ssh.tasks.editor.nameRequired")), {
        message: t("ssh.tasks.editor.nameRequired"),
      });
      return null;
    }
    return saveTask({ draft: activeTab.draft, tabId: activeTab.id });
  }, [activeTab, handleError, saveTask, t]);

  const runActiveTask = useCallback(async () => {
    if (!activeTab?.draft) return;
    const currentDraft = activeTab.draft;
    if (!currentDraft.name.trim()) {
      handleError(new Error(t("ssh.tasks.editor.nameRequired")), {
        message: t("ssh.tasks.editor.nameRequired"),
      });
      return;
    }
    if (currentDraft.steps.every((step) => !step.enabled)) {
      handleError(new Error(t("ssh.tasks.editor.runNeedsSteps")), {
        message: t("ssh.tasks.editor.runNeedsSteps"),
      });
      return;
    }
    let taskId = currentDraft.id ?? null;
    if (!taskId || isTaskTabDirty(activeTab)) {
      const detail = await saveActiveDraft();
      if (!detail) return;
      taskId = detail.task.id;
    }
    await prepareRun(taskId);
  }, [activeTab, handleError, prepareRun, saveActiveDraft, t]);

  const handleListRun = useCallback(
    (task: SshTask) => void prepareRun(task.id),
    [prepareRun],
  );
  const handleDuplicate = useCallback(
    (task: SshTask) => duplicateTask(task.id),
    [duplicateTask],
  );
  const handleExample = useCallback(
    () => newTask(dockerImageExportTemplate(workspaceId)),
    [newTask, workspaceId],
  );

  const changeRunEnvironment = useCallback(
    (environmentId: string) => {
      if (!runDialogTask) return;
      const defaults = defaultTaskRunInputs(
        detectTaskInputs(runDialogTask.steps, true),
        mergeWorkspaceVariables(
          runWorkspaceVariables,
          workspaceEnvironmentById(runEnvironments, environmentId),
        ),
      );
      const environment = workspaceEnvironmentById(runEnvironments, environmentId);
      setRunEnvironmentId(environmentId);
      setRunInputs(defaults.inputs);
      setRunSecretInputs(defaults.secretNames);
      setRunFilledFromWorkspace(defaults.filledFromWorkspace.length > 0);
      setRunActiveEnvironmentName(environment?.name?.trim() || null);
    },
    [runDialogTask, runEnvironments, runWorkspaceVariables],
  );

  const openHistoryRun = useCallback(
    async (run: SshTaskRun) => {
      try {
        const detail = await queryClient.fetchQuery({
          queryKey: ["ssh-task", workspaceId, run.taskId],
          queryFn: () => getSshTask(workspaceId, run.taskId),
        });
        setActiveRun(run);
        setActiveRunTask(detail);

        const hasLiveEvents = (eventsByRunRef.current[run.id]?.length ?? 0) > 0;
        if (hasLiveEvents || historyLogByRunRef.current[run.id] !== undefined) {
          return;
        }

        setHistoryLogLoading(true);
        try {
          const logText = await readSshTaskRunLog(workspaceId, run.id);
          setHistoryLogByRun((current) =>
            current[run.id] === undefined ? { ...current, [run.id]: logText } : current,
          );
        } catch (error) {
          handleError(error, { key: "feedback.ssh.taskLogLoadFailed" });
          setHistoryLogByRun((current) =>
            current[run.id] === undefined ? { ...current, [run.id]: "" } : current,
          );
        } finally {
          setHistoryLogLoading(false);
        }
      } catch (error) {
        handleError(error, { key: "feedback.ssh.taskLoadFailed" });
      }
    },
    [handleError, queryClient, workspaceId],
  );

  const shellSidebar = useMemo(
    () => (
      <TaskList
        key={workspaceId}
        loading={tasksQuery.isLoading}
        onDelete={setDeleteTarget}
        onDuplicate={handleDuplicate}
        onExample={handleExample}
        onNew={newTask}
        onOpenConnections={onOpenConnections}
        onReorder={reorderTasks}
        onRun={handleListRun}
        onSelect={selectTask}
        reordering={reorderMutation.isPending}
        selectedTaskId={activeTaskId}
        tasks={tasks}
      />
    ),
    [
      activeTaskId,
      handleDuplicate,
      handleExample,
      handleListRun,
      newTask,
      onOpenConnections,
      reorderMutation.isPending,
      reorderTasks,
      selectTask,
      tasks,
      tasksQuery.isLoading,
      workspaceId,
    ],
  );

  useEffect(() => {
    if (!onShellSidebarChange) return;
    onShellSidebarChange(shellSidebar);
    return () => onShellSidebarChange(null);
  }, [onShellSidebarChange, shellSidebar]);

  useEffect(() => {
    if (!active) return;
    function onKeyDown(event: KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
      if (!draft || !activeTab) return;
      if (event.key.toLowerCase() === "s") {
        event.preventDefault();
        if (!draft.name.trim() || saveMutation.isPending) return;
        saveMutation.mutate({ draft, tabId: activeTab.id });
        return;
      }
      if (event.key === "Enter") {
        event.preventDefault();
        void runActiveTask();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [active, activeTab, draft, runActiveTask, saveMutation]);

  function requestCloseTab(tabId: string) {
    const tab = editorState.tabs.find((item) => item.id === tabId);
    if (tab && isTaskTabDirty(tab)) {
      setCloseTabId(tabId);
      return;
    }
    setEditorState((current) => closeTaskTab(current, tabId));
  }

  async function saveAndCloseTab() {
    if (!closeTabId) return;
    const tab = editorState.tabs.find((item) => item.id === closeTabId);
    if (!tab?.draft) {
      setCloseTabId(null);
      return;
    }
    if (!tab.draft.name.trim()) {
      handleError(new Error(t("ssh.tasks.editor.nameRequired")), {
        message: t("ssh.tasks.editor.nameRequired"),
      });
      return;
    }
    try {
      const detail = await saveTask({ draft: tab.draft, tabId: tab.id });
      setEditorState((current) => {
        const savedTabId =
          current.tabs.find((item) => item.taskId === detail.task.id)?.id ?? closeTabId;
        return closeTaskTab(current, savedTabId);
      });
      setCloseTabId(null);
    } catch {
      // Mutation onError already reports feedback.
    }
  }

  function discardAndCloseTab() {
    if (!closeTabId) return;
    setEditorState((current) => closeTaskTab(current, closeTabId));
    setCloseTabId(null);
  }

  function updateDraft(nextDraft: SshTaskSaveInput) {
    if (!activeTab) return;
    setEditorState((current) => updateTaskTabDraft(current, activeTab.id, nextDraft));
  }

  function updateDetailView(view: SshTaskDetailView) {
    if (!activeTab) return;
    setEditorState((current) => updateTaskTabView(current, activeTab.id, view));
  }

  const closeTarget = editorState.tabs.find((tab) => tab.id === closeTabId) ?? null;
  const runDisabledReason = !draft
    ? null
    : !draft.name.trim()
      ? t("ssh.tasks.editor.nameRequired")
      : draft.steps.every((step) => !step.enabled)
        ? t("ssh.tasks.editor.runNeedsSteps")
        : null;
  const taskTabs = editorState.tabs.map((tab) => {
    const task = tab.taskId ? tasks.find((item) => item.id === tab.taskId) : null;
    const title = tab.draft?.name.trim() || task?.name || t("ssh.tasks.editor.untitled");
    const dirty = isTaskTabDirty(tab);
    return {
      id: tab.id,
      title,
      draggable: false,
      meta: dirty ? (
        <span
          aria-label={t("ssh.tasks.tabs.unsaved")}
          className="h-1.5 w-1.5 shrink-0 rounded-full bg-[var(--u-color-primary)]"
          title={t("ssh.tasks.tabs.unsaved")}
        />
      ) : undefined,
    };
  });

  const editorPane =
    detailQuery.isError && !draft ? (
      <ErrorState className="min-h-0 flex-1">{String(detailQuery.error)}</ErrorState>
    ) : draft ? (
      <TaskEditor
        connections={connections}
        draft={draft}
        key={activeTab?.id ?? "draft"}
        onChange={updateDraft}
        onRun={() => void runActiveTask()}
        onSave={() => activeTab && saveMutation.mutate({ draft, tabId: activeTab.id })}
        runDisabledReason={runDisabledReason}
        saving={saveMutation.isPending && saveMutation.variables?.tabId === activeTab?.id}
      />
    ) : (
      <TaskWorkspaceEmpty
        hasTasks={tasks.length > 0}
        loading={Boolean(activeTab)}
        onExample={handleExample}
        onNew={() => newTask()}
      />
    );

  const historyPane =
    activeTaskId && draft ? (
      <TaskHistory
        clearing={clearMutation.isPending && clearMutation.variables === activeTaskId}
        loading={runsQuery.isLoading}
        onClear={() => setClearTaskId(activeTaskId)}
        onSelectRun={(run) => void openHistoryRun(run)}
        runs={runsQuery.data ?? []}
        selectedRunId={activeRun?.id}
      />
    ) : (
      <div className="flex min-h-0 flex-1 items-center justify-center border-t border-[var(--u-color-border)] text-[12px] text-[var(--u-color-text-muted)]">
        {t("ssh.tasks.history.saveFirst")}
      </div>
    );

  return (
    <div className="flex min-h-0 flex-1 bg-[var(--u-color-surface)]">
      <main className="flex min-h-0 min-w-0 flex-1 flex-col">
        <Tabs
          activeId={activeTab?.id ?? ""}
          endControl={
            activeTab ? (
              <SegmentedControl
                className="w-[196px]"
                onChange={updateDetailView}
                options={[
                  { label: t("ssh.tasks.tabs.editor"), value: "editor" },
                  { label: t("ssh.tasks.tabs.history"), value: "history" },
                ]}
                value={activeTab.view}
              />
            ) : undefined
          }
          onClose={requestCloseTab}
          onSelect={(tabId) =>
            setEditorState((current) => ({ ...current, activeTabId: tabId }))
          }
          tabs={taskTabs}
        />
        {draft && showHistory ? (
          <SplitPane
            className="min-h-0 flex-1"
            defaultRatio={62}
            minPaneSize={140}
            orientation="vertical"
            resizable
          >
            {editorPane}
            {historyPane}
          </SplitPane>
        ) : (
          editorPane
        )}
        {activeRun && activeRunTask && (
          <TaskRunPanel
            cancelling={cancelMutation.isPending}
            events={eventsByRun[activeRun.id] ?? []}
            logLoading={historyLogLoading && historyLogByRun[activeRun.id] === undefined}
            logText={historyLogByRun[activeRun.id] ?? null}
            onCancel={() => cancelMutation.mutate()}
            onClose={() => {
              setActiveRun(null);
              setActiveRunTask(null);
            }}
            run={activeRun}
            task={activeRunTask}
          />
        )}
      </main>

      <TaskRunDialog
        activeEnvironmentName={runActiveEnvironmentName}
        connectionId={runConnectionId}
        connections={connections}
        environmentId={runEnvironmentId}
        environmentLoadFailed={runEnvironmentLoadFailed}
        environments={runEnvironments.map(({ id, name }) => ({ id, name }))}
        error={runMutation.error}
        filledFromWorkspace={runFilledFromWorkspace}
        inputValues={runInputs}
        onConnectionChange={setRunConnectionId}
        onEnvironmentChange={changeRunEnvironment}
        onInputChange={(name, value) =>
          setRunInputs((current) => ({ ...current, [name]: value }))
        }
        onOpenChange={(open) => !open && setRunDialogTask(null)}
        onRun={() => runMutation.mutate()}
        open={runDialogTask !== null}
        pending={runMutation.isPending}
        secretInputNames={runSecretInputs}
        task={runDialogTask}
      />
      <ConfirmDialog
        confirmLabel={t("ssh.tasks.tabs.saveAndClose")}
        description={
          closeTarget
            ? t("ssh.tasks.tabs.unsavedDescription", {
                name:
                  closeTarget.draft?.name.trim() || t("ssh.tasks.editor.untitled"),
              })
            : ""
        }
        onConfirm={() => void saveAndCloseTab()}
        onOpenChange={(open) => !open && setCloseTabId(null)}
        onSecondary={discardAndCloseTab}
        open={closeTarget !== null}
        pending={saveMutation.isPending}
        secondaryLabel={t("ssh.tasks.tabs.discard")}
        secondaryTone="danger"
        title={t("ssh.tasks.tabs.unsavedTitle")}
        tone="default"
      />
      <ConfirmDialog
        confirmLabel={t("ssh.tasks.actions.delete")}
        description={
          deleteTarget ? t("ssh.tasks.confirmDelete", { name: deleteTarget.name }) : ""
        }
        onConfirm={() => deleteTarget && deleteMutation.mutate(deleteTarget.id)}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        open={deleteTarget !== null}
        pending={deleteMutation.isPending}
        title={t("ssh.tasks.confirmDeleteTitle")}
      />
      <ConfirmDialog
        confirmLabel={t("ssh.tasks.history.clear")}
        description={t("ssh.tasks.history.clearDescription")}
        onConfirm={() => clearTaskId && clearMutation.mutate(clearTaskId)}
        onOpenChange={(open) => !open && setClearTaskId(null)}
        open={clearTaskId !== null}
        pending={clearMutation.isPending}
        title={t("ssh.tasks.history.clearTitle")}
      />
    </div>
  );
}
