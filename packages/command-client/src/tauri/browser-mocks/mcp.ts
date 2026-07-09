import { UNHANDLED, type MockResult } from "./types";

/**
 * Browser preview mock. We return `found: false` with a `dev` build kind so the
 * Settings → MCP "binary not found" guidance (including the dev-specific hint)
 * is visible without a real Tauri backend.
 */
export function handleMcpMock<T>(command: string): MockResult<T> {
  if (command === "mcp_binary_path") {
    return {
      path: "/mock/unfour-mcp",
      found: false,
      buildKind: "dev",
    } as T;
  }

  return UNHANDLED;
}
