import { describe, expect, it } from "vitest";
import {
  cancelSshReconnect,
  closeSshSession,
  connectSshSession,
  getSshSessionHistory,
  saveApiRequest,
  saveSshConnection,
  sendSshInput,
} from "./tauri";

describe("SSH browser mock lifecycle", () => {
  it("supports the health-state contract and reconnect cancellation", async () => {
    const workspaceId = `mock-health-${crypto.randomUUID()}`;
    const connection = await saveSshConnection({
      workspaceId,
      name: "Mock SSH",
      host: "localhost",
      username: "developer",
      authKind: "password",
      credentialRef: "mock-credential-ref",
    });
    const session = await connectSshSession({
      workspaceId,
      connectionId: connection.id,
    });

    expect(session.status).toBe("connected");
    expect(session.reconnectAttempt).toBe(0);
    await expect(
      sendSshInput({ workspaceId, sessionId: session.sessionId, data: "whoami\n" }),
    ).resolves.toMatchObject({ kind: "output" });

    const cancelled = await cancelSshReconnect({
      workspaceId,
      sessionId: session.sessionId,
    });
    expect(cancelled.status).toBe("disconnected");
    await expect(
      sendSshInput({ workspaceId, sessionId: session.sessionId, data: "pwd\n" }),
    ).rejects.toThrow("not connected");

    const closed = await closeSshSession({
      workspaceId,
      sessionId: session.sessionId,
    });
    expect(closed.status).toBe("disconnected");
  });

  it("hydrates only safe output for the requested workspace and session", async () => {
    const workspaceId = `mock-history-${crypto.randomUUID()}`;
    const connection = await saveSshConnection({
      workspaceId,
      name: "Mock SSH history",
      host: "localhost",
      username: "developer",
      authKind: "password",
      credentialRef: "must-not-be-persisted",
    });
    const session = await connectSshSession({
      workspaceId,
      connectionId: connection.id,
    });
    await sendSshInput({
      workspaceId,
      sessionId: session.sessionId,
      data: "password=secret\n",
    });

    const history = await getSshSessionHistory({
      workspaceId,
      sessionId: session.sessionId,
    });
    expect(history.length).toBeGreaterThan(0);
    expect(history.every((event) => event.kind !== "input")).toBe(true);
    expect(history.map((event) => event.data).join("")).not.toContain("secret");
    expect(history.map((event) => event.data).join("")).not.toContain(
      "must-not-be-persisted",
    );
    await expect(
      getSshSessionHistory({
        workspaceId: `other-${workspaceId}`,
        sessionId: session.sessionId,
      }),
    ).resolves.toEqual([]);
  });
});

describe("API body redaction in browser mock", () => {
  it("redacts sensitive fields in saved request body while preserving structure", async () => {
    const workspaceId = `mock-redaction-${crypto.randomUUID()}`;
    const sensitiveBody = JSON.stringify({
      username: "alice",
      authorization: "Bearer secret-token-123",
      nested: {
        xApiKey: "should-not-redact-different-key",
        "x-api-key": "real-secret-key",
        items: [{ cookie: "session=abc123", name: "item1" }],
      },
    });

    const saved = await saveApiRequest({
      workspaceId,
      name: "Redaction Test",
      method: "POST",
      url: "https://api.example.com/login",
      headers: [
        { key: "Content-Type", value: "application/json", enabled: true },
        { key: "Authorization", value: "Bearer secret-token-123", enabled: true },
      ],
      query: [],
      body: sensitiveBody,
      bodyKind: "json",
    });

    expect(saved.body).not.toBeNull();
    const parsed = JSON.parse(saved.body!);
    // Non-sensitive fields preserved
    expect(parsed.username).toBe("alice");
    expect(parsed.nested.items[0].name).toBe("item1");
    // Sensitive fields redacted
    expect(parsed.authorization).toBe("<redacted>");
    expect(parsed.nested["x-api-key"]).toBe("<redacted>");
    expect(parsed.nested.items[0].cookie).toBe("<redacted>");
    // Non-sensitive key with similar name not redacted
    expect(parsed.nested.xApiKey).toBe("should-not-redact-different-key");

    // Headers also redacted
    const headers = JSON.parse(saved.headersJson);
    const authHeader = headers.find((h: { key: string }) => h.key === "Authorization");
    expect(authHeader.value).toBe("<redacted>");
    const ctHeader = headers.find((h: { key: string }) => h.key === "Content-Type");
    expect(ctHeader.value).toBe("application/json");
  });

  it("preserves non-sensitive JSON body unchanged", async () => {
    const workspaceId = `mock-no-redaction-${crypto.randomUUID()}`;
    const cleanBody = JSON.stringify({ name: "test", count: 42, tags: ["a", "b"] });

    const saved = await saveApiRequest({
      workspaceId,
      name: "Clean Body Test",
      method: "POST",
      url: "https://api.example.com/data",
      headers: [],
      query: [],
      body: cleanBody,
      bodyKind: "json",
    });

    // Body string returned verbatim when no sensitive keys exist
    expect(saved.body).toBe(cleanBody);
  });

  it("handles non-JSON and empty bodies gracefully", async () => {
    const workspaceId = `mock-plain-${crypto.randomUUID()}`;

    const plainSaved = await saveApiRequest({
      workspaceId,
      name: "Plain Text",
      method: "POST",
      url: "https://api.example.com/upload",
      headers: [],
      query: [],
      body: "this is plain text, not json",
      bodyKind: "text",
    });
    expect(plainSaved.body).toBe("this is plain text, not json");

    const emptySaved = await saveApiRequest({
      workspaceId,
      name: "Empty Body",
      method: "GET",
      url: "https://api.example.com/items",
      headers: [],
      query: [],
      body: undefined,
      bodyKind: "none",
    });
    expect(emptySaved.body).toBeNull();
  });
});
