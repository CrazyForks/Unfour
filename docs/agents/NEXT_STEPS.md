# Next Steps

## Recommended: Polish & Integration

Priority order:

1. **known_hosts integration**
   - Goal: Import or export fingerprints from/to the system `known_hosts` file.
   - Scope: `crates/ssh-engine`, Tauri commands.
   - Forbidden: Do not modify system `known_hosts` file contents without explicit user action.
   - Risk: Medium — file I/O + security-sensitive parsing.
   - Prerequisites: None.
   - Acceptance criteria: User can export stored fingerprints to `known_hosts` format; user can import existing `known_hosts` entries; round-trip preserves fingerprint integrity.
   - Independent commit: Yes.
   - Recommended model: Codex / stronger coding model (file I/O + security).

## Lower Priority

2. **API body redaction**
   - Goal: Redact sensitive values in API request bodies before saving to history.
   - Scope: `crates/http-engine`, `packages/api-debugger`.
   - Forbidden: Do not modify the actual request payload sent to servers; redaction applies only to stored history and logs.
   - Risk: Low — additive filtering on the persistence path.
   - Prerequisites: None.
   - Acceptance criteria: Request bodies containing keys matching the existing sensitive-key list are redacted in history; JSON structure is preserved; non-sensitive fields are unchanged.
   - Independent commit: Yes.
   - Recommended model: weaker cheaper model is sufficient.

3. **Lint warning cleanup**
   - Goal: Reduce `react-hooks/set-state-in-effect`, `react-hooks/exhaustive-deps`, `react-hooks/refs`, and `react-refresh/only-export-components` warnings across the codebase.
   - Scope: `packages/api-debugger`, `packages/database`, `packages/terminal`, `apps/desktop`.
   - Forbidden: Do not change component behavior or refactor hooks beyond what is needed to resolve the warnings.
   - Risk: Low — targeted fixes per warning.
   - Prerequisites: None.
   - Acceptance criteria: `pnpm run lint` produces fewer warnings than the current 64; no new errors introduced; component rendering unchanged.
   - Independent commit: Yes.
   - Recommended model: weaker cheaper model is sufficient.

## Completed

- **Terminal session persistence:** SQLite-backed output history with per-session buffering (16 KB / 500 ms flush interval), secret redaction, UTF-8-safe truncation (256 KB retention limit), hydration on app reopen, browser mock mode compatible. 5 Rust tests in `terminal_history.rs`, 2 integration tests in `ssh.rs`, 1 frontend store test, 1 browser mock test.
- **Connection health monitoring:** Native russh keepalive runs every 3 seconds and detects an unresponsive peer in about 9 seconds.
- **Bounded reconnection:** Unexpected disconnects expose degraded/reconnecting/failed states, retry at 1/2/4 seconds, stop after 3 attempts, and support cancellation.
- **Reconnect cleanup:** Explicit close suppresses reconnect, one supervisor owns each session lifecycle, event listeners are centralized, and failed/cancelled sessions release native handles and cancellation senders.
- **Private-key authentication:** Implemented. Unencrypted keys load from disk via `ssh-key::PrivateKey::read_openssh_file`. Encrypted keys attempt passphrase from SecretStore. Host-key TOFU works for both auth methods.
- **Host-key fingerprint UI:** View trusted fingerprint and reset fingerprint implemented in `SshConnectionDialog`. Mismatch errors surface via the backend.
- **Host-key management Tauri commands:** `ssh_host_key_get` and `ssh_host_key_reset` added.
