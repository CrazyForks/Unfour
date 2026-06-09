import { describe, expect, it } from "vitest";
import {
  cancelSshReconnect,
  closeSshSession,
  connectSshSession,
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
});
