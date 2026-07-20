import type { TFunction } from "@unfour/ui";

export function formatTerminalError(error: unknown, t: TFunction) {
  const rawMessage = rawTerminalError(error);
  const normalized = rawMessage.toLowerCase();

  if (
    normalized.includes("password ssh session requires a stored password") ||
    normalized.includes("password auth requires a credential reference") ||
    normalized.includes("password ssh auth requires a password")
  ) {
    return t("ssh.errors.credentialMissing");
  }

  if (normalized.includes("host key verification failed")) {
    return t("ssh.errors.hostKeyMismatch");
  }
  if (
    normalized.includes("fingerprint does not match") ||
    normalized.includes("host key")
  ) {
    return t("ssh.errors.hostKeyFailed");
  }
  if (
    normalized.includes("authentication failed") ||
    normalized.includes("invalid credentials") ||
    normalized.includes("permission denied") ||
    normalized.includes("key rejected")
  ) {
    return t("ssh.errors.authenticationFailed");
  }
  if (normalized.includes("timed out") || normalized.includes("timeout")) {
    return t("ssh.errors.timeout");
  }
  if (
    normalized.includes("connection refused") ||
    normalized.includes("actively refused")
  ) {
    return t("ssh.errors.connectionRefused");
  }
  if (
    normalized.includes("could not resolve") ||
    normalized.includes("dns") ||
    normalized.includes("nodename") ||
    normalized.includes("name or service not known")
  ) {
    return t("ssh.errors.hostNotResolved");
  }
  if (
    normalized.includes("network unreachable") ||
    normalized.includes("host unreachable") ||
    normalized.includes("no route to host")
  ) {
    return t("ssh.errors.hostUnreachable");
  }
  if (normalized.includes("private key file not found")) {
    return t("ssh.errors.keyFileMissing");
  }
  if (
    normalized.includes("failed to decrypt ssh private key") ||
    normalized.includes("failed to read ssh private key") ||
    normalized.includes("passphrase may be incorrect")
  ) {
    return t("ssh.errors.keyUnreadable");
  }
  if (normalized.includes("session is not connected")) {
    return t("ssh.errors.sessionDisconnected");
  }
  if (normalized.includes("pty size")) {
    return t("ssh.errors.ptySize");
  }

  return redactTerminalError(rawMessage);
}

function rawTerminalError(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "object" && error && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}

function redactTerminalError(value: string) {
  return value
    .split(/\r?\n/)
    .map((line) => {
      if (
        /(^|\b)(authorization|cookie|proxy-authorization|x-api-key|x-auth-token|password|passphrase|private[-_ ]?key)(\b|:|=)/i.test(
          line,
        )
      ) {
        return "<redacted>";
      }
      return line;
    })
    .join("\n")
    .trim();
}
