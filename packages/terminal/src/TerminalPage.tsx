import { FormEvent, useEffect, useMemo, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  closeSshSession,
  connectSshSession,
  deleteSshConnection,
  exportSshLog,
  saveSshConnection,
  type SshConnectionInput,
  type SshSessionSummary,
} from "@unfour/command-client";
import { useWorkspaceStore } from "@unfour/workspace";
import { LoadingState } from "@unfour/ui";
import { TerminalModuleToolbar } from "./components/TerminalModuleToolbar";
import { TerminalWorkspace } from "./components/TerminalWorkspace";
import { SshConnectionDialog } from "./components/SshConnectionDialog";
import { useSshConnections } from "./hooks/useSshConnections";
import { useTerminalSessions } from "./hooks/useTerminalSessions";
import { useTerminalSplit } from "./hooks/useTerminalSplit";
import {
  useTerminalStore,
} from "./model/terminal-state";
import {
  defaultSshConnectionInput,
  sshConnectionToInput,
} from "./model/ssh-connection-state";
import { buildTerminalSessionTabs } from "./model/terminal-tabs";

export function TerminalPage({ workspaceId }: { workspaceId: string }) {
  const queryClient = useQueryClient();
  const {
    selectedSshConnectionId: selectedConnectionId,
    setSelectedSshConnection,
  } = useWorkspaceStore();
  const split = useTerminalSplit();
  const activeSessionId = useTerminalStore((state) => state.activeSessionId);
  const activateWorkspace = useTerminalStore((state) => state.activateWorkspace);
  const appendTerminalEvents = useTerminalStore((state) => state.appendTerminalEvents);
  const clearTerminalSessionEvents = useTerminalStore(
    (state) => state.clearTerminalSessionEvents,
  );
  const resetTerminalEvents = useTerminalStore((state) => state.resetTerminalEvents);
  const setActiveSessionId = useTerminalStore((state) => state.setActiveSessionId);
  const setExportedLog = useTerminalStore((state) => state.setExportedLog);
  const setSearchOpen = useTerminalStore((state) => state.setSearchOpen);
  const startTerminalSession = useTerminalStore((state) => state.startTerminalSession);
  const terminalEvents = useTerminalStore((state) => state.terminalEvents);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [form, setForm] = useState<SshConnectionInput>(() =>
    defaultSshConnectionInput(workspaceId),
  );

  const connectionsQuery = useSshConnections(workspaceId);
  const sessionsQuery = useTerminalSessions(workspaceId);
  const connections = useMemo(() => connectionsQuery.data ?? [], [connectionsQuery.data]);
  const sessions = useMemo(() => sessionsQuery.data ?? [], [sessionsQuery.data]);
  const selectedConnection = useMemo(
    () => connections.find((item) => item.id === selectedConnectionId) ?? null,
    [connections, selectedConnectionId],
  );
  const activeSession = useMemo(
    () => sessions.find((item) => item.sessionId === activeSessionId) ?? null,
    [activeSessionId, sessions],
  );
  const sessionTabs = useMemo(
    () => buildTerminalSessionTabs({ connections, sessions }),
    [connections, sessions],
  );

  useEffect(() => {
    activateWorkspace(workspaceId);
  }, [activateWorkspace, workspaceId]);

  useEffect(() => {
    setForm((current) => ({ ...current, workspaceId }));
  }, [workspaceId]);

  useEffect(() => {
    if (!connections.length) {
      if (selectedConnectionId) {
        setSelectedSshConnection(null);
      }
      return;
    }

    if (!selectedConnectionId || !connections.some((connection) => connection.id === selectedConnectionId)) {
      setSelectedSshConnection(connections[0].id);
    }
  }, [connections, selectedConnectionId, setSelectedSshConnection]);

  useEffect(() => {
    if (!selectedConnection) {
      return;
    }

    setForm(sshConnectionToInput(selectedConnection, workspaceId));
  }, [selectedConnection, workspaceId]);

  useEffect(() => {
    if (!sessions.length) {
      if (activeSessionId) {
        setActiveSessionId(null);
      }
      return;
    }

    if (!activeSessionId || !sessions.some((session) => session.sessionId === activeSessionId)) {
      setActiveSessionId(sessions[0].sessionId);
    }
  }, [activeSessionId, sessions, setActiveSessionId]);

  const saveMutation = useMutation({
    mutationFn: saveSshConnection,
    onSuccess: (connection) => {
      setSelectedSshConnection(connection.id);
      setDialogOpen(false);
      queryClient.invalidateQueries({ queryKey: ["ssh-connections", workspaceId] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (connectionId: string) => deleteSshConnection(workspaceId, connectionId),
    onSuccess: () => {
      setSelectedSshConnection(null);
      setDialogOpen(false);
      resetTerminalEvents();
      queryClient.invalidateQueries({ queryKey: ["ssh-connections", workspaceId] });
      queryClient.invalidateQueries({ queryKey: ["ssh-sessions", workspaceId] });
    },
  });

  const connectMutation = useMutation({
    mutationFn: (connectionId: string) =>
      connectSshSession({ workspaceId, connectionId, cols: 120, rows: 32 }),
    onSuccess: (session) => {
      startTerminalSession(session.sessionId, [
        {
          sessionId: session.sessionId,
          kind: "output",
          data: `Connected to ${session.username}@${session.host}. PTY ${session.cols}x${session.rows} allocated.\r\n`,
          createdAt: session.createdAt,
        },
      ]);
      queryClient.setQueryData<SshSessionSummary[]>(
        ["ssh-sessions", workspaceId],
        (current = []) => [
          ...current.filter((item) => item.sessionId !== session.sessionId),
          session,
        ],
      );
      queryClient.invalidateQueries({ queryKey: ["ssh-sessions", workspaceId] });
    },
  });

  const closeMutation = useMutation({
    mutationFn: (sessionId: string) => closeSshSession({ workspaceId, sessionId }),
    onSuccess: (session) => {
      appendTerminalEvents([
        {
          sessionId: session.sessionId,
          kind: "close",
          data: "SSH session closed.\r\n",
          createdAt: session.updatedAt,
        },
      ]);
      queryClient.invalidateQueries({ queryKey: ["ssh-sessions", workspaceId] });
    },
  });

  const exportMutation = useMutation({
    mutationFn: () => exportSshLog({ workspaceId, sessionId: activeSessionId ?? "" }),
    onSuccess: (log) => setExportedLog(log.content),
  });

  function updateForm(patch: Partial<SshConnectionInput>) {
    setForm((current) => ({ ...current, ...patch, workspaceId }));
  }

  function newConnection() {
    setSelectedSshConnection(null);
    setForm(defaultSshConnectionInput(workspaceId));
    setDialogOpen(true);
  }

  function openConnectionSettings() {
    if (selectedConnection) {
      setForm(sshConnectionToInput(selectedConnection, workspaceId));
    } else {
      setForm(defaultSshConnectionInput(workspaceId));
    }
    setDialogOpen(true);
  }

  function submitConnection(event: FormEvent) {
    event.preventDefault();
    saveMutation.mutate({
      ...form,
      credentialRef: form.credentialRef?.trim() || null,
      keyPath: form.keyPath?.trim() || null,
    });
  }

  function connectSelectedConnection() {
    if (!selectedConnectionId) {
      newConnection();
      return;
    }

    connectMutation.mutate(selectedConnectionId);
  }

  const activeError =
    connectionsQuery.error ??
    sessionsQuery.error ??
    connectMutation.error ??
    closeMutation.error ??
    exportMutation.error;

  return (
    <div className="flex h-full min-h-0 min-w-0 flex-col bg-[var(--u-color-surface)]">
      <TerminalModuleToolbar
        activeSessionCount={sessions.length}
        canConnect={Boolean(selectedConnectionId)}
        canSplit={sessions.filter((session) => session.status === "active").length > 1}
        canUseSessionActions={Boolean(activeSessionId)}
        connecting={connectMutation.isPending}
        onClear={() => clearTerminalSessionEvents(activeSessionId)}
        onCloseSession={() => activeSessionId && closeMutation.mutate(activeSessionId)}
        onExportLog={() => exportMutation.mutate()}
        onNewConnection={newConnection}
        onNewSession={connectSelectedConnection}
        onOpenPreferences={openConnectionSettings}
        onSearch={() => setSearchOpen(true)}
        onSplit={split.setMode}
        selectedConnectionName={selectedConnection?.name}
        splitMode={split.mode}
      />
      {connectionsQuery.isLoading || sessionsQuery.isLoading ? (
        <LoadingState className="min-h-0 flex-1 rounded-none border-0">
          Loading terminal workspace...
        </LoadingState>
      ) : (
        <TerminalWorkspace
          activeSession={activeSession}
          activeSessionId={activeSessionId}
          error={activeError}
          events={terminalEvents}
          emptyMessage={
            connections.length
              ? "Select an SSH connection and start a session."
              : "No SSH connections are configured for this workspace."
          }
          onCloseSession={(sessionId) => closeMutation.mutate(sessionId)}
          onSelectSession={setActiveSessionId}
          sessions={sessionTabs}
          splitMode={split.mode}
        />
      )}
      <SshConnectionDialog
        canDelete={Boolean(form.id)}
        error={saveMutation.error ?? deleteMutation.error}
        form={form}
        onDelete={() => form.id && deleteMutation.mutate(form.id)}
        onOpenChange={setDialogOpen}
        onSubmit={submitConnection}
        onUpdate={updateForm}
        open={dialogOpen}
        pending={saveMutation.isPending || deleteMutation.isPending}
        workspaceId={workspaceId}
      />
    </div>
  );
}
