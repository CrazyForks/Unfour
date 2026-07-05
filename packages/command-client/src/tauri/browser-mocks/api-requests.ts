import { redactHeaders, redactJsonBody, resolveInput } from "./helpers";
import {
  assertMockCollection,
  assertMockFolder,
  firstOrCreateMockCollectionId,
  mockActiveEnvVariables,
  mockState,
  mockStore,
  nextMockRequestSortOrder,
  normalizeMockId,
} from "./state";
import { UNHANDLED } from "./types";
import type {
  ApiRequestInput,
  ApiResponse,
  ApiSavedRequest,
} from "../../types";

export async function handleApiRequestMock<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T | typeof UNHANDLED> {
  if (command === "api_history_list") {
    return mockStore.history as T;
  }

  if (command === "api_history_detail") {
    const workspaceId = String(args?.workspaceId ?? "");
    const historyId = String(args?.historyId ?? "");
    const detail = mockStore.historyDetails.find(
      (item) => item.workspaceId === workspaceId && item.id === historyId,
    );
    if (!detail) throw new Error("api history not found");
    return detail as T;
  }

  if (command === "api_saved_requests") {
    const workspaceId = String(args?.workspaceId ?? mockState.activeWorkspaceId);
    return mockStore.savedRequests.filter((item) => item.workspaceId === workspaceId) as T;
  }

  if (command === "api_request_save") {
    const input = args?.input as ApiRequestInput;
    const collectionId = input.collectionId ?? firstOrCreateMockCollectionId(input.workspaceId);
    const parentFolderId = normalizeMockId(input.parentFolderId);
    assertMockCollection(input.workspaceId, collectionId);
    assertMockFolder(input.workspaceId, collectionId, parentFolderId);
    const saved: ApiSavedRequest = {
      id: crypto.randomUUID(),
      workspaceId: input.workspaceId,
      name: input.name || `${input.method} ${input.url}`,
      collectionId,
      parentFolderId,
      sortOrder: nextMockRequestSortOrder(input.workspaceId, collectionId, parentFolderId),
      authJson: input.authJson ?? JSON.stringify({ type: "none" }),
      method: input.method,
      url: input.url,
      headersJson: JSON.stringify(redactHeaders(input.headers)),
      queryJson: JSON.stringify(input.query),
      body: redactJsonBody(input.body),
      bodyKind: input.bodyKind,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      deletedAt: null,
      revision: 1,
      syncStatus: "local",
      remoteId: null,
    };
    mockStore.savedRequests = [saved, ...mockStore.savedRequests];
    return saved as T;
  }

  if (command === "api_request_update") {
    const input = args?.input as ApiRequestInput;
    const workspaceId = String(args?.workspaceId ?? input.workspaceId);
    const requestId = String(args?.requestId ?? "");
    if (workspaceId !== input.workspaceId) throw new Error("api request workspace mismatch");
    const collectionId = input.collectionId ?? firstOrCreateMockCollectionId(workspaceId);
    const parentFolderId = normalizeMockId(input.parentFolderId);
    assertMockCollection(workspaceId, collectionId);
    assertMockFolder(workspaceId, collectionId, parentFolderId);
    const index = mockStore.savedRequests.findIndex(
      (item) => item.workspaceId === workspaceId && item.id === requestId,
    );
    if (index === -1) throw new Error("api request not found");
    const current = mockStore.savedRequests[index];
    const saved: ApiSavedRequest = {
      ...current,
      name: input.name || `${input.method} ${input.url}`,
      collectionId,
      parentFolderId,
      sortOrder:
        current.collectionId === collectionId && current.parentFolderId === parentFolderId
          ? current.sortOrder
          : nextMockRequestSortOrder(workspaceId, collectionId, parentFolderId),
      authJson: input.authJson ?? JSON.stringify({ type: "none" }),
      method: input.method,
      url: input.url,
      headersJson: JSON.stringify(redactHeaders(input.headers)),
      queryJson: JSON.stringify(input.query),
      body: redactJsonBody(input.body),
      bodyKind: input.bodyKind,
      updatedAt: new Date().toISOString(),
      revision: current.revision + 1,
      syncStatus: "pending",
    };
    mockStore.savedRequests = [
      ...mockStore.savedRequests.slice(0, index),
      saved,
      ...mockStore.savedRequests.slice(index + 1),
    ];
    return saved as T;
  }

  if (command === "api_request_duplicate") {
    const workspaceId = String(args?.workspaceId ?? "");
    const requestId = String(args?.requestId ?? "");
    const source = mockStore.savedRequests.find(
      (item) => item.workspaceId === workspaceId && item.id === requestId,
    );
    if (!source) throw new Error("api request not found");
    const now = new Date().toISOString();
    const duplicate: ApiSavedRequest = {
      ...source,
      id: crypto.randomUUID(),
      name: `${source.name} Copy`,
      createdAt: now,
      updatedAt: now,
      revision: 1,
      syncStatus: "local",
      remoteId: null,
    };
    mockStore.savedRequests = [duplicate, ...mockStore.savedRequests];
    return duplicate as T;
  }

  if (command === "api_request_delete") {
    const workspaceId = String(args?.workspaceId ?? "");
    const requestId = String(args?.requestId ?? "");
    const initialLength = mockStore.savedRequests.length;
    mockStore.savedRequests = mockStore.savedRequests.filter(
      (item) => !(item.workspaceId === workspaceId && item.id === requestId),
    );
    if (mockStore.savedRequests.length === initialLength) {
      throw new Error("api request not found");
    }
    return mockStore.savedRequests.filter((item) => item.workspaceId === workspaceId) as T;
  }

  if (command === "api_send_request") {
    const input = args?.input as ApiRequestInput;
    const started = performance.now();
    const resolved = resolveInput(input, mockActiveEnvVariables(input.workspaceId));
    const url = new URL(resolved.url);
    resolved.query
      .filter((item) => item.enabled && item.key)
      .forEach((item) => url.searchParams.append(item.key, item.value));
    const headers = Object.fromEntries(
      resolved.headers
        .filter((item) => item.enabled && item.key)
        .map((item) => [item.key, item.value]),
    );
    const response = await fetch(url, {
      method: resolved.method,
      headers,
      body:
        resolved.method === "GET" || resolved.method === "HEAD"
          ? undefined
          : resolved.body || undefined,
    });
    const body = await response.text();
    const result: ApiResponse = {
      historyId: crypto.randomUUID(),
      status: response.status,
      statusText: response.statusText,
      headers: Array.from(response.headers.entries()).map(([key, value]) => ({
        key,
        value,
        enabled: true,
      })),
      body,
      durationMs: Math.round(performance.now() - started),
    };
    mockStore.history = [
      {
        id: result.historyId,
        workspaceId: input.workspaceId,
        name: input.name ?? null,
        method: resolved.method,
        url: resolved.url,
        status: result.status,
        durationMs: result.durationMs,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        deletedAt: null,
        revision: 1,
        syncStatus: "local",
        remoteId: null,
      },
      ...mockStore.history,
    ];
    mockStore.historyDetails = [
      {
        id: result.historyId,
        workspaceId: input.workspaceId,
        name: input.name ?? null,
        method: resolved.method,
        url: resolved.url,
        requestHeadersJson: JSON.stringify(redactHeaders(input.headers)),
        requestQueryJson: JSON.stringify(input.query),
        requestBody: redactJsonBody(input.body),
        status: result.status,
        durationMs: result.durationMs,
        responseHeadersJson: JSON.stringify(result.headers),
        responseBodyPreview: body.slice(0, 20_000),
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        deletedAt: null,
        revision: 1,
        syncStatus: "local",
        remoteId: null,
      },
      ...mockStore.historyDetails,
    ];
    return result as T;
  }

  return UNHANDLED;
}
