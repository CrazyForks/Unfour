import type { SshConnectionInput } from "@unfour/command-client";

export function defaultTerminalInput() {
  return "whoami\n";
}

export function defaultSshConnectionInput(workspaceId: string): SshConnectionInput {
  return {
    workspaceId,
    name: "Deploy host",
    host: "example.internal",
    port: 22,
    username: "deploy",
    authKind: "password",
    credentialRef: null,
  };
}
