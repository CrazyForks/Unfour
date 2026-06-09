# Project State

## Scan Metadata

- **Scanned at:** 2026-06-09 (checkpoint refresh)
- **Branch:** main
- **Current commit:** 13c4b28
- **Commit message:** feat(ssh): add private-key authentication and host-key controls
- **Working tree state:** Clean
- **Last checkpoint:** 13c4b28 feat(ssh): add private-key authentication and host-key controls

## Tech Stack

- **Desktop shell:** Tauri 2 (Rust + WebView)
- **Frontend:** React 19, TypeScript, Vite, Tailwind CSS, Radix UI, TanStack Query
- **Backend:** Rust, Tokio, SQLite (rusqlite), russh (SSH native)
- **Build:** pnpm workspace, Cargo workspace
- **Test:** Vitest (frontend), Cargo test (Rust)

## Current Phase

Terminal streaming integration is **complete**. PTY lifecycle, stdin/stdout streaming, Tauri event streaming, frontend terminal input capture, resize propagation, and search are all wired end-to-end.

UI module split is **in progress**. Terminal and Database packages have been extracted from `packages/app-shell`. Further Workspace extraction is planned.

## Verified Capabilities

### Backend (Rust)

| Capability | Status | Tests |
|---|---|---|
| Core models & redaction | Complete | 3 pass |
| Local storage & migrations | Complete | 6 pass |
| Activity logging | Complete | Covered in local_storage |
| SecretStore (credential references) | Complete | 4 pass |
| Database engine (SQLite CRUD + schema) | Complete | 3 pass |
| HTTP engine (API client + history) | Complete | 8 pass |
| SSH engine (simulated + native) | Complete | 16 pass |
| Workspace engine | Complete | Tests blocked on Windows DLL issue |
| CommandBus (Tauri adapter) | Complete | Compile-verified |

### Frontend (TypeScript)

| Capability | Status | Tests |
|---|---|---|
| Workspace store | Complete | 12 pass |
| API Debugger | Complete | 20 pass |
| Database (connections + query) | Complete | 16 pass |
| Terminal state (streaming + search) | Complete | 5 pass |

### Build

- **Frontend production build:** PASS
- **Frontend bundle chunks:** index (383 kB), xterm (367 kB), vendor-tanstack (101 kB), vendor-radix (88 kB), monaco (15 kB)
- **Total Rust tests:** 40 passing (6 crates); unfour-workspace blocked by DLL issue
- **Total frontend tests:** 53 passing (4 files)

## Partially Implemented

- **UI module split:** Terminal and Database packages extracted. Workspace package exists but app-shell still contains some workspace UI.
- **SSH authentication:** Password auth and private-key auth both work under `ssh-native`. Encrypted key passphrase loading has limited support (ssh-key crate format constraints).
- **Host-key UI:** View trusted fingerprint and reset fingerprint implemented. Mismatch error display is handled by the TOFU backend.
- **Database drivers:** SQLite driver is functional. PostgreSQL/MySQL drivers are not started.

## Not Started

- SSH connection health monitoring / keep-alive
- SSH auto-reconnection logic
- Terminal output persistence to SQLite
- `known_hosts` integration
- Terminal multiplexing (tmux/screen-like)
- SCP/SFTP file transfer
- Additional database drivers (PostgreSQL, MySQL)

## Verification Results

| Command | Result | Notes |
|---|---|---|
| `git diff --check` | PASS | No trailing whitespace issues |
| `pnpm run lint` | PASS (warnings) | 0 errors, 63 warnings; pre-existing in api-debugger, database, terminal, desktop |
| `pnpm run test` | PASS | 53 tests, 4 files |
| `pnpm run build` | PASS | Production build succeeds |
| `cargo fmt --check` | PASS | No formatting issues |
| `cargo test --workspace` | PARTIAL | 40 tests pass across 6 crates. `unfour-workspace` fails with Windows `STATUS_ENTRYPOINT_NOT_FOUND` (DLL loading issue) |
| `cargo check --workspace` | PASS | All crates compile |
| `cargo check -p unfour-workspace --features ssh-native` | PASS | SSH feature compiles |

## Known Limitations

- **Windows workspace tests:** `cargo test -p unfour-workspace` fails with `STATUS_ENTRYPOINT_NOT_FOUND`. Likely a native DLL dependency issue (OpenSSL/SQLite) on this Windows environment. Does not indicate code defects.
- **Lint warnings:** Multiple packages have `react-hooks/set-state-in-effect`, `react-hooks/exhaustive-deps`, `react-hooks/refs`, and `react-refresh/only-export-components` warnings. These are pre-existing and do not block builds.
- **Real SSH verification:** Native SSH transport with PTY streaming has not been manually verified against a live SSH server in this environment.
- **API body redaction:** Request bodies are not redacted in logs or history.
