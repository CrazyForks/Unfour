import { call } from "./invoke";

export type McpBuildKind = "dev" | "release";

export interface McpBinaryPathResult {
  /** Absolute path the external MCP client should invoke. */
  path: string;
  /** Whether a runnable binary actually exists at `path`. */
  found: boolean;
  /** Build kind, so the UI can tailor its guidance. */
  buildKind: McpBuildKind;
}

export function getMcpBinaryPath() {
  return call<McpBinaryPathResult>("mcp_binary_path");
}
