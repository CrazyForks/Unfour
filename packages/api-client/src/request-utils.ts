import type {
  ApiHistoryDetail,
  ApiRequestInput,
  ApiSavedRequest,
  KeyValue,
} from "@unfour/command-client";

export function parseKeyValues(value: string): KeyValue[] {
  try {
    const parsed = JSON.parse(value);
    if (Array.isArray(parsed)) {
      return parsed.filter(
        (item): item is KeyValue =>
          typeof item?.key === "string" &&
          typeof item?.value === "string" &&
          typeof item?.enabled === "boolean",
      );
    }
  } catch {
    return [];
  }

  return [];
}

export function savedRequestToInput(
  saved: ApiSavedRequest,
  workspaceId: string,
): ApiRequestInput {
  return {
    workspaceId,
    name: saved.name,
    folderPath: saved.folderPath,
    method: saved.method,
    url: saved.url,
    headers: parseKeyValues(saved.headersJson),
    query: parseKeyValues(saved.queryJson),
    body: saved.body ?? undefined,
    bodyKind: saved.bodyKind,
    timeoutMs: 60_000,
  };
}

export function historyDetailToInput(history: ApiHistoryDetail): ApiRequestInput {
  return {
    workspaceId: history.workspaceId,
    name: history.name ?? `${history.method} ${history.url}`,
    folderPath: null,
    method: history.method,
    url: history.url,
    headers: parseKeyValues(history.requestHeadersJson),
    query: parseKeyValues(history.requestQueryJson),
    body: history.requestBody ?? undefined,
    bodyKind: "json",
    timeoutMs: 60_000,
  };
}

export function parseCollectionImport(
  value: unknown,
  workspaceId: string,
): ApiRequestInput[] {
  const rawItems = Array.isArray(value)
    ? value
    : typeof value === "object" && value !== null && "savedRequests" in value
      ? (value as { savedRequests?: unknown }).savedRequests
      : [];
  if (!Array.isArray(rawItems)) {
    return [];
  }

  return rawItems
    .map((item) => normalizeImportedRequest(item, workspaceId))
    .filter((item): item is ApiRequestInput => item !== null);
}

function normalizeImportedRequest(
  item: unknown,
  workspaceId: string,
): ApiRequestInput | null {
  if (typeof item !== "object" || item === null) {
    return null;
  }
  const candidate = item as Partial<ApiRequestInput>;
  if (typeof candidate.method !== "string" || typeof candidate.url !== "string") {
    return null;
  }

  return {
    workspaceId,
    name: typeof candidate.name === "string" ? candidate.name : undefined,
    folderPath: typeof candidate.folderPath === "string" ? candidate.folderPath : null,
    method: candidate.method.toUpperCase(),
    url: candidate.url,
    headers: Array.isArray(candidate.headers) ? sanitizeKeyValues(candidate.headers) : [],
    query: Array.isArray(candidate.query) ? sanitizeKeyValues(candidate.query) : [],
    body: typeof candidate.body === "string" ? candidate.body : undefined,
    bodyKind: typeof candidate.bodyKind === "string" ? candidate.bodyKind : "json",
    timeoutMs: typeof candidate.timeoutMs === "number" ? candidate.timeoutMs : 60_000,
  };
}

function sanitizeKeyValues(items: unknown[]): KeyValue[] {
  return items
    .filter(isKeyValueLike)
    .map((item) => ({
      key: item.key ?? "",
      value: item.value ?? "",
      enabled: typeof item.enabled === "boolean" ? item.enabled : true,
    }));
}

function isKeyValueLike(item: unknown): item is Partial<KeyValue> {
  if (typeof item !== "object" || item === null) {
    return false;
  }
  const candidate = item as Record<string, unknown>;
  return typeof candidate.key === "string" && typeof candidate.value === "string";
}

export function groupSavedRequests(items: ApiSavedRequest[]) {
  const groups = new Map<string, ApiSavedRequest[]>();
  for (const item of items) {
    const folder = item.folderPath?.trim() || "Unfiled";
    groups.set(folder, [...(groups.get(folder) ?? []), item]);
  }

  return Array.from(groups.entries())
    .sort(([left], [right]) => {
      if (left === "Unfiled") return -1;
      if (right === "Unfiled") return 1;
      return left.localeCompare(right);
    })
    .map(([folder, groupItems]) => ({
      folder,
      items: groupItems.sort((left, right) => left.name.localeCompare(right.name)),
    }));
}

export function duplicateEnvironmentKeys(variables: KeyValue[]) {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const variable of variables) {
    const key = variable.key.trim().toLowerCase();
    if (!key || !variable.enabled) {
      continue;
    }
    if (seen.has(key)) {
      duplicates.add(variable.key.trim());
    }
    seen.add(key);
  }
  return Array.from(duplicates);
}

export function isSensitiveKey(key: string) {
  return /(token|secret|password|passwd|api[_-]?key|auth|credential)/i.test(key);
}

export function formatByteSize(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const kb = bytes / 1024;
  if (kb < 1024) {
    return `${kb.toFixed(kb >= 10 ? 0 : 1)} KB`;
  }
  const mb = kb / 1024;
  return `${mb.toFixed(mb >= 10 ? 0 : 1)} MB`;
}
