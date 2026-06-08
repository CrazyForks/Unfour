# Next Steps

## Recommended: SSH Phase 3 — Auth & Reliability

Priority order:

1. **Private-key authentication**
   - Goal: Support SSH key-based auth using russh's `authenticate_publickey` with key files from disk.
   - Scope: `crates/ssh-engine`, `packages/terminal`, Tauri commands.
   - Forbidden: Do not store private keys in SQLite. Persist only a credential reference via SecretStore.
   - Risk: Medium — touches SecretStore, SSH auth flow, and UI file-picker.
   - Prerequisites: None.
   - Acceptance criteria: Users can create SSH connections with private-key auth; keys are loaded from disk and never persisted as plaintext; simulated path has a stub.
   - Independent commit: Yes.
   - Recommended model: Codex / stronger coding model (Rust async + russh + SecretStore).

2. **Connection health monitoring**
   - Goal: Add keep-alive pings and detect dropped connections.
   - Scope: `crates/ssh-engine` background task, `packages/terminal` UI indicator.
   - Forbidden: Do not add busy-wait loops. Use tokio intervals or russh keep-alive.
   - Risk: Low-Medium — async background task, UI state indicator.
   - Prerequisites: None.
   - Acceptance criteria: A disconnected session shows a visual indicator within 10 seconds; no CPU spin.
   - Independent commit: Yes.
   - Recommended model: Codex / stronger coding model (Rust async).

3. **Reconnection logic**
   - Goal: Auto-reconnect sessions when the SSH connection drops unexpectedly.
   - Scope: `crates/ssh-engine` session management, `packages/terminal` UI.
   - Forbidden: Do not silently retry forever. Implement bounded backoff with user-visible state.
   - Risk: Medium — state machine changes, error handling paths.
   - Prerequisites: Connection health monitoring.
   - Acceptance criteria: Dropped connections attempt reconnection up to 3 times with backoff; user can cancel; reconnection state is visible.
   - Independent commit: Yes.
   - Recommended model: Codex / stronger coding model (Rust async + state machine).

4. **Terminal session persistence**
   - Goal: Persist terminal output to SQLite so history survives app restarts.
   - Scope: `crates/local-storage` schema, `crates/ssh-engine` output callback, `packages/terminal` hydration.
   - Forbidden: Do not persist secrets or credential references in terminal history.
   - Risk: Medium — storage schema change, cross-layer data flow.
   - Prerequisites: None.
   - Acceptance criteria: Terminal output is saved per session; output is restored when app restarts; old output can be truncated/rotated.
   - Independent commit: Yes.
   - Recommended model: Codex / stronger coding model (storage schema + cross-layer).

## Lower Priority

5. **Host-key UI**
   - Goal: Allow users to view, trust, or reset host-key fingerprints.
   - Scope: `packages/terminal` UI, Tauri commands.
   - Recommended model: weaker cheaper model is sufficient.

6. **known_hosts integration**
   - Goal: Import or export fingerprints from/to the system `known_hosts` file.
   - Scope: `crates/ssh-engine`, Tauri commands.
   - Recommended model: Codex / stronger coding model (file I/O + security).

7. **API body redaction**
   - Goal: Redact sensitive values in API request bodies before saving to history.
   - Scope: `crates/http-engine`, `packages/api-debugger`.
   - Recommended model: weaker cheaper model is sufficient.

8. **Lint warning cleanup**
   - Goal: Reduce `react-hooks/set-state-in-effect`, `react-hooks/exhaustive-deps`, and `react-hooks/refs` warnings.
   - Scope: `packages/api-debugger`, `packages/database`, `packages/terminal`, `apps/desktop`.
   - Recommended model: weaker cheaper model is sufficient.
