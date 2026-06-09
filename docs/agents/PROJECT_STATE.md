# Project State

## Scan Metadata

- **Scanned at:** 2026-06-09 (checkpoint refresh)
- **Branch:** main
- **Current batch:** SSH reliability
- **Commit message:** feat(ssh): add connection health and reconnect handling
- **Batch state:** Implementation and verification complete
- **Last checkpoint:** SSH reliability implementation and verification complete

## Tech Stack

- **Desktop shell:** Tauri 2 (Rust + WebView)
- **Frontend:** React 19, TypeScript, Vite, Tailwind CSS, Radix UI, TanStack Query
- **Backend:** Rust, Tokio, SQLite (rusqlite), russh (SSH native)
- **Build:** pnpm workspace, Cargo workspace
- **Test:** Vitest (frontend), Cargo test (Rust)

## Current Phase

Terminal streaming and SSH reliability integration are **complete**. PTY lifecycle, stdin/stdout streaming, Tauri event streaming, frontend terminal input capture, resize propagation, search, keepalive monitoring, bounded reconnection, cancellation, and cleanup are wired end-to-end.

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
| SSH engine (simulated + native) | Complete | 20 default + 14 native-feature pass |
| Workspace engine | Complete | Tests blocked on Windows DLL issue |
| CommandBus (Tauri adapter) | Complete | Compile-verified |

### Frontend (TypeScript)

| Capability | Status | Tests |
|---|---|---|
| Workspace store | Complete | 12 pass |
| API Debugger | Complete | 20 pass |
| Database (connections + query) | Complete | 16 pass |
| Terminal state and command-client mock | Complete | 6 pass |

### Build

- **Frontend production build:** PASS
- **Frontend bundle chunks:** index (384 kB), xterm (367 kB), vendor-tanstack (101 kB), vendor-radix (88 kB), monaco (15 kB)
- **Total Rust tests:** 44 passing before unfour-workspace is blocked by the Windows DLL issue
- **Total frontend tests:** 54 passing (5 files)

## Partially Implemented

- **UI module split:** Terminal and Database packages extracted. Workspace package exists but app-shell still contains some workspace UI.
- **SSH authentication:** Password auth and private-key auth both work under `ssh-native`. Encrypted key passphrase loading has limited support (ssh-key crate format constraints).
- **Host-key UI:** View trusted fingerprint and reset fingerprint implemented. Mismatch error display is handled by the TOFU backend.
- **SSH live reliability verification:** Keepalive and reconnect policy are automated-test covered, but a live localhost SSH stop/start cycle was not available in this environment.
- **Database drivers:** SQLite driver is functional. PostgreSQL/MySQL drivers are not started.

## Not Started

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
| `pnpm run test` | PASS | 54 tests, 5 files |
| `pnpm run build` | PASS | Production build succeeds |
| `cargo fmt --check` | PASS | No formatting issues |
| `cargo test --workspace` | PARTIAL | 44 tests pass across 6 crates. `unfour-workspace` fails with Windows `STATUS_ENTRYPOINT_NOT_FOUND` (DLL loading issue) |
| `cargo check --workspace` | PASS | All crates compile |
| `cargo check -p unfour-workspace --features ssh-native` | PASS | SSH feature compiles |
| `cargo test -p unfour-ssh-engine --features ssh-native` | PASS | 14 native-feature tests |
| Browser mock first viewport | PASS | SSH empty state and status bar render correctly |

## Known Limitations

- **Windows workspace tests:** `cargo test -p unfour-workspace` fails with `STATUS_ENTRYPOINT_NOT_FOUND`. Likely a native DLL dependency issue (OpenSSL/SQLite) on this Windows environment. Does not indicate code defects.
- **Lint warnings:** Multiple packages have `react-hooks/set-state-in-effect`, `react-hooks/exhaustive-deps`, `react-hooks/refs`, and `react-refresh/only-export-components` warnings. These are pre-existing and do not block builds.
- **Real SSH verification:** Native SSH transport drop detection, reconnect cancellation, retry exhaustion, and recovery after server return are `NOT VERIFIED` against a live SSH server in this environment.
- **API body redaction:** Request bodies are not redacted in logs or history.
