import { handleApiMock } from "./api";
import { handleDatabaseMock } from "./database";
import { handleDiagnosticsMock } from "./diagnostics";
import { handleSecretStoreMock } from "./secret-store";
import { handleSshMock } from "./ssh";
import { handleWorkspaceMock } from "./workspace";
import { UNHANDLED, type MockCommandHandler } from "./types";

const mockHandlers: MockCommandHandler[] = [
  handleDiagnosticsMock,
  handleWorkspaceMock,
  handleApiMock,
  handleSecretStoreMock,
  handleDatabaseMock,
  handleSshMock,
];

export async function mockInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  for (const handler of mockHandlers) {
    const result = await handler<T>(command, args);
    if (result !== UNHANDLED) {
      return result;
    }
  }

  throw new Error(`Mock command is not implemented: ${command}`);
}
