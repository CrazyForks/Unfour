import type { SshConnection, SshConnectionInput } from "@unfour/command-client";

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

export function sshConnectionToInput(
  connection: SshConnection,
  workspaceId: string,
): SshConnectionInput {
  return {
    id: connection.id,
    workspaceId,
    name: connection.name,
    host: connection.host,
    port: connection.port,
    username: connection.username,
    authKind: connection.authKind,
    keyPath: connection.keyPath,
    credentialRef: connection.credentialRef,
  };
}

export function sshEndpointLabel(connection: SshConnection | null | undefined) {
  if (!connection) {
    return "No SSH connection";
  }

  return `${connection.username}@${connection.host}`;
}
