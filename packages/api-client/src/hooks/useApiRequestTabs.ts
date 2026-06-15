import { useCallback, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  deleteApiRequest,
  duplicateApiRequest,
  getApiHistoryDetail,
  getWorkspaceEnvironment,
  listApiHistory,
  listSavedApiRequests,
  saveApiRequest,
  sendApiRequest,
  updateWorkspaceEnvironment,
  type ApiRequestInput,
  type KeyValue,
} from "@unfour/command-client";
import { formatError } from "../model/api-request-state";
import {
  closeApiTab,
  completeTabSave,
  completeTabSend,
  createNewRequestTab,
  emptyApiTabsState,
  failTabSave,
  failTabSend,
  openHistoryRequest,
  openSavedRequest,
  setActiveApiTab,
  setApiSplitDirection,
  setTabRequestPanel,
  setTabResponsePanel,
  startTabSave,
  startTabSend,
  updateTabDraft,
  type ApiRequestTab,
} from "../model/request-tabs";
import type {
  ApiSplitDirection,
  RequestDraft,
  RequestParamsTab,
  ResponseTab,
} from "../model/types";

export function useApiRequestTabs(workspaceId: string) {
  const queryClient = useQueryClient();
  const nextNewId = useRef(1);
  const importInputRef = useRef<HTMLInputElement>(null);
  const [collectionStatus, setCollectionStatus] = useState("");
  const [state, setState] = useState(() =>
    createNewRequestTab(emptyApiTabsState(workspaceId), "new:1"),
  );

  const savedQuery = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["api-saved", workspaceId],
    queryFn: () => listSavedApiRequests(workspaceId),
  });
  const historyQuery = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["api-history", workspaceId],
    queryFn: () => listApiHistory(workspaceId),
  });
  const environmentQuery = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["workspace-environment", workspaceId],
    queryFn: () => getWorkspaceEnvironment(workspaceId),
  });

  const sendMutation = useMutation({
    mutationFn: ({ input }: { input: ApiRequestInput; tabId: string }) =>
      sendApiRequest(input),
    onSuccess: (response, variables) => {
      setState((current) => completeTabSend(current, variables.tabId, response));
      queryClient.invalidateQueries({ queryKey: ["api-history", workspaceId] });
    },
    onError: (error, variables) =>
      setState((current) =>
        failTabSend(current, variables.tabId, formatError(error)),
      ),
  });

  const saveMutation = useMutation({
    mutationFn: ({ input }: { input: ApiRequestInput; tabId: string }) =>
      saveApiRequest(input),
    onSuccess: (saved, variables) => {
      setState((current) => completeTabSave(current, variables.tabId, saved));
      queryClient.invalidateQueries({ queryKey: ["api-saved", workspaceId] });
    },
    onError: (error, variables) =>
      setState((current) =>
        failTabSave(current, variables.tabId, formatError(error)),
      ),
  });

  const duplicateMutation = useMutation({
    mutationFn: (requestId: string) => duplicateApiRequest(workspaceId, requestId),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["api-saved", workspaceId] }),
  });
  const deleteMutation = useMutation({
    mutationFn: (requestId: string) => deleteApiRequest(workspaceId, requestId),
    onSuccess: (_, requestId) => {
      setState((current) => closeApiTab(current, `saved:${requestId}`));
      queryClient.invalidateQueries({ queryKey: ["api-saved", workspaceId] });
    },
  });
  const saveEnvironmentMutation = useMutation({
    mutationFn: (variables: KeyValue[]) =>
      updateWorkspaceEnvironment(workspaceId, variables),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: ["workspace-environment", workspaceId],
      }),
  });
  const importCollectionMutation = useMutation({
    mutationFn: async (requests: ApiRequestInput[]) => {
      for (const request of requests) {
        await saveApiRequest({ ...request, workspaceId });
      }
      return requests.length;
    },
    onSuccess: (count) => {
      setCollectionStatus(`Imported ${count} request${count === 1 ? "" : "s"}`);
      queryClient.invalidateQueries({ queryKey: ["api-saved", workspaceId] });
    },
    onError: (error) => setCollectionStatus(formatError(error)),
  });
  const sendRequest = sendMutation.mutate;
  const saveRequest = saveMutation.mutateAsync;

  const activeTab =
    state.tabs.find((tab) => tab.id === state.activeTabId) ?? null;
  const envVariables = environmentQuery.data?.variables ?? [];

  const newRequest = useCallback(() => {
    nextNewId.current += 1;
    const id = `new:${nextNewId.current}`;
    setState((current) => createNewRequestTab(current, id));
  }, []);

  const openSaved = useCallback((requestId: string) => {
    const saved = savedQuery.data?.find((item) => item.id === requestId);
    if (saved) {
      setState((current) => openSavedRequest(current, saved));
    }
  }, [savedQuery.data]);

  const openHistory = useCallback(async (historyId: string) => {
    const detail = await getApiHistoryDetail(workspaceId, historyId);
    setState((current) => openHistoryRequest(current, detail));
  }, [workspaceId]);

  function updateDraft(tabId: string, patch: Partial<RequestDraft>) {
    setState((current) => updateTabDraft(current, tabId, patch));
  }

  const sendTab = useCallback((tab: ApiRequestTab) => {
    setState((current) => startTabSend(current, tab.id));
    sendRequest({ input: tabToInput(tab, workspaceId), tabId: tab.id });
  }, [sendRequest, workspaceId]);

  const saveTab = useCallback(async (
    tab: ApiRequestTab,
    identity?: { folderPath: string; name: string },
  ) => {
    const draft = identity ? { ...tab.draft, ...identity } : tab.draft;
    if (identity) {
      setState((current) => updateTabDraft(current, tab.id, identity));
    }
    setState((current) => startTabSave(current, tab.id));
    try {
      const saved = await saveRequest({
        input: tabToInput({ ...tab, draft }, workspaceId),
        tabId: tab.id,
      });
      return saved.id;
    } catch {
      return null;
    }
  }, [saveRequest, workspaceId]);

  return {
    activeTab,
    collectionStatus,
    deleteMutation,
    duplicateMutation,
    envVariables,
    historyItems: historyQuery.data ?? [],
    importInputRef,
    savedRequests: savedQuery.data ?? [],
    state,
    closeTab: (tabId: string) =>
      setState((current) => closeApiTab(current, tabId)),
    importCollectionMutation,
    newRequest,
    openHistory,
    openSaved,
    saveEnvironment: (variables: KeyValue[]) =>
      saveEnvironmentMutation.mutate(variables),
    saveEnvironmentMutation,
    saveTab,
    selectTab: (tabId: string) =>
      setState((current) => setActiveApiTab(current, tabId)),
    sendTab,
    setRequestTab: (tabId: string, requestTab: RequestParamsTab) =>
      setState((current) => setTabRequestPanel(current, tabId, requestTab)),
    setResponseTab: (tabId: string, responseTab: ResponseTab) =>
      setState((current) => setTabResponsePanel(current, tabId, responseTab)),
    setSplitDirection: (direction: ApiSplitDirection) =>
      setState((current) => setApiSplitDirection(current, direction)),
    setCollectionStatus,
    updateDraft,
  };
}

export function tabToInput(
  tab: ApiRequestTab,
  workspaceId: string,
): ApiRequestInput {
  return {
    workspaceId,
    name: tab.draft.name,
    folderPath: tab.draft.folderPath || null,
    method: tab.draft.method,
    url: tab.draft.url,
    headers: tab.draft.headers,
    query: tab.draft.query,
    body:
      tab.draft.method === "GET" || tab.draft.method === "HEAD"
        ? undefined
        : tab.draft.body,
    bodyKind: "json",
    timeoutMs: 60_000,
  };
}
