## Result
- Status: Completed
- Batch: Terminal streaming integration
- Commit: pending

## Modified Files

### Backend (Rust)
- `crates/ssh-engine/src/ssh.rs` — PTY channel lifecycle, async send_input/resize, terminal output callback, channel close
- `crates/ssh-engine/src/lib.rs` — exported `TerminalOutputCallback`
- `crates/ssh-engine/Cargo.toml` — (no new deps, `futures-lite` was added then removed)
- `apps/desktop/src-tauri/src/command_bus.rs` — terminal output callback wired to Tauri events, async send_input/resize, stored AppHandle
- `apps/desktop/src-tauri/capabilities/default.json` — added `core:event:default` permission

### Frontend (TypeScript)
- `packages/terminal/package.json` — added `@tauri-apps/api`, `@xterm/addon-search`
- `packages/terminal/src/components/TerminalPane.tsx` — Tauri event listener, xterm onData input capture, resize propagation, search addon init
- `packages/terminal/src/components/TerminalSearchBar.tsx` — wired search addon (findNext/findPrevious/clearDecorations)
- `packages/terminal/src/components/TerminalSplitView.tsx` — simplified props (removed input box)
- `packages/terminal/src/components/TerminalWorkspace.tsx` — simplified props
- `packages/terminal/src/components/TerminalModuleToolbar.tsx` — onResize made optional
- `packages/terminal/src/TerminalPage.tsx` — removed input/resize mutations (now in TerminalPane)
- `packages/terminal/src/model/terminal-state.ts` — added `terminalSearchAddon` + `setTerminalSearchAddon`
- `packages/terminal/src/model/terminal-state.test.ts` — new: 5 tests for store behavior

## Verification
- `cargo fmt --check` — PASS
- `cargo test -p unfour-ssh-engine` — PASS (13 tests, 2 new)
- `cargo test -p unfour-ssh-engine --features ssh-native` — PASS (9 tests, 1 new callback test)
- `cargo check --workspace` — PASS
- `cargo check -p unfour-workspace --features ssh-native` — PASS
- `pnpm run lint` — PASS (0 errors)
- `pnpm run test` — PASS (53 tests, 5 new)
- `pnpm run build` — PASS

## Terminal Streaming Architecture

### Backend (ssh-engine)
- **PTY lifecycle:** `connect_native` opens a session channel, requests PTY (`xterm-256color`), starts shell via `request_shell`. Channel stored in `NativeSshHandle` alongside the connection handle.
- **Output streaming:** Background `tokio::spawn` task reads `ChannelMsg::Data` from the channel and invokes the `TerminalOutputCallback` with JSON payloads (`{sessionId, data}`).
- **Input:** `send_input` (now async) writes bytes to the native channel via `data_bytes`. Simulated path remains synchronous.
- **Resize:** `resize` (now async) calls `window_change` on the native channel. Simulated path updates internal state.
- **Close:** `close_session` closes the channel first (terminating the reader task), then disconnects the connection.
- **Output callback:** `set_terminal_output_callback` registers an `Arc<dyn Fn(String)>` that the reader task invokes. The `CommandBus` wires this to `app.emit("ssh://terminal-data", payload)`.

### Frontend (terminal package)
- **Event listener:** `TerminalPane` registers a Tauri `listen("ssh://terminal-data")` handler that writes data directly to xterm and appends events to the store.
- **Input capture:** `terminal.onData` captures keyboard input and sends it via `sendSshInput` (dynamic import to avoid circular deps).
- **Resize propagation:** `terminal.onResize` detects dimension changes from FitAddon and calls `resizeSshSession` with actual cols/rows.
- **Search:** `@xterm/addon-search` initialized in TerminalPane, stored in zustand store, consumed by TerminalSearchBar for findNext/findPrevious/clearDecorations.
- **Polling fallback:** Non-Tauri (mock) mode still works via the existing event-rendering path.

## Tests
- **New backend tests:**
  - `async_send_input_and_resize_work_in_simulated_path` — verifies async send_input and resize
  - `multiple_sessions_handle_concurrent_input_and_close` — concurrent operations, isolation
  - `terminal_output_callback_can_be_registered` — callback registration and invocation (ssh-native only)
- **New frontend tests:**
  - `stores and clears the search addon reference`
  - `preserves search addon across workspace activation`
  - `appends terminal events from streaming`
  - `clears events for a specific session`
  - `toggles search open state`
- **Real localhost SSH verification:** NOT VERIFIED (no SSH server available)
- **Unverified areas:** End-to-end streaming with live SSH server, private-key authentication

## Checkpoint Refresh
- **Resolved issues:** PTY allocation, stdin/stdout streaming, Tauri event streaming, frontend terminal input, terminal search, resize propagation — all implemented.
- **Remaining issues:** Private-key auth, end-to-end verification with live SSH, reconnection on disconnect, terminal session persistence.
- **Next recommended batch:** Private-key authentication support, connection health monitoring, reconnection logic.

## Scope Confirmation
- Unrelated files changed: No
- Dependencies added: `@tauri-apps/api` (terminal), `@xterm/addon-search` (terminal)
- Public contracts changed: `send_input` and `resize` are now async; `SshService::set_terminal_output_callback` added
- Backend call chain changed: CommandBus now sets terminal output callback in `new()` to emit Tauri events
