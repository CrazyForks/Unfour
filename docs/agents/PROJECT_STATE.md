# Project State

## Scan Metadata

- **Scanned at:** 2026-06-08
- **Branch:** main
- **Current commit:** aa10bfa428fd044e44cc1d1de5b3dc8ac382f76b
- **Commit message:** feat(terminal): connect ssh streaming to xterm
- **Working tree state:** Clean (1 untracked: docs/agents/CHECKPOINT_REFRESH.md)
- **Last checkpoint:** c9b7c13 docs(checkpoint): refresh repository state

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
| SSH engine (simulated + native) | Complete | 13 pass |
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
- **Frontend bundle chunks:** index (380 kB), xterm (367 kB), vendor-tanstack (101 kB), vendor-radix (88 kB), monaco (15 kB)

## Partially Implemented

- **UI module split:** Terminal and Database packages extracted. Workspace package exists but app-shell still contains some workspace UI.
- **SSH authentication:** Password auth via SecretStore works under `ssh-native`. Private-key auth not implemented.
- **Database drivers:** SQLite driver is functional. PostgreSQL/MySQL drivers are not started.

## Not Started

- SSH private-key authentication
- SSH connection health monitoring / keep-alive
- SSH auto-reconnection logic
- Terminal output persistence to SQLite
- Host-key UI (view/trust/reset fingerprints)
- `known_hosts` integration
- Terminal multiplexing (tmux/screen-like)
- SCP/SFTP file transfer
- Additional database drivers (PostgreSQL, MySQL)

## Verification Results

| Command | Result | Notes |
|---|---|---|
| `git diff --check` | PASS | No trailing whitespace issues |
| `pnpm run lint` | PASS (warnings) | 0 errors; warnings exist in api-debugger, database, terminal, desktop |
| `pnpm run test` | PASS | 53 tests, 4 files |
| `pnpm run build` | PASS | Production build succeeds |
| `cargo fmt --check` | PASS | No formatting issues |
| `cargo test --workspace` | PARTIAL | All crates except `unfour-workspace` pass. `unfour-workspace` fails with Windows `STATUS_ENTRYPOINT_NOT_FOUND` (DLL loading issue) |
| `cargo check --workspace` | PASS | All crates compile |
| `cargo check -p unfour-workspace --features ssh-native` | PASS | SSH feature compiles |

## Known Limitations

- **Windows workspace tests:** `cargo test -p unfour-workspace` fails with `STATUS_ENTRYPOINT_NOT_FOUND`. Likely a native DLL dependency issue (OpenSSL/SQLite) on this Windows environment. Does not indicate code defects.
- **Lint warnings:** Multiple packages have `react-hooks/set-state-in-effect`, `react-hooks/exhaustive-deps`, `react-hooks/refs`, and `react-refresh/only-export-components` warnings. These are pre-existing and do not block builds.
- **Real SSH verification:** Native SSH transport with PTY streaming has not been manually verified against a live SSH server in this environment.
- **API body redaction:** Request bodies are not redacted in logs or history.
