# NEXT_STEPS.md

> Read-only checkpoint scan. These are candidate tasks only. None have been executed unless marked completed.

---

## TASK-01: Extract App.tsx into focused components [COMPLETED]

| Field | Value |
|---|---|
| Status | Completed in `refactor(desktop): extract app shell components` |
| Result | App.tsx reduced from 954 to 199 lines. Extracted: `AppTitleBar`, `ModuleSidebar`, `WindowControls`, `WorkspaceMenu`, `WorkspaceDialogs`, `BottomPanelPlaceholder`, `StatusBarPlaceholder`, `RightInspectorPlaceholder`, utilities, and hooks into `apps/desktop/src/components/`. |

---

## TASK-02: Replace hardcoded Tailwind colors with semantic tokens [COMPLETED]

| Field | Value |
|---|---|
| Status | Completed in `chore(frontend): complete hardening cleanup batch` |
| Result | All 23 hardcoded Tailwind color classes in App.tsx and 7 in feature packages migrated to `--u-color-*` / `--u-badge-*` tokens. CSS-level hex values in `styles.css` migrated (`.subpanel`, `.data-table`, scrollbar). 2 new tokens added (`--u-color-scrollbar`, `--u-color-scrollbar-hover`). |

---

## TASK-03: Configure ESLint or Biome for frontend linting

| Field | Value |
|---|---|
| Goal | Add a frontend linter (ESLint flat config or Biome) with a `lint` script in `package.json`. Configure rules for: unused imports, consistent style, no `any`, React hook rules, and TypeScript strict checks. |
| Scope | Root `package.json` (add script + devDependency). New config file at root (`.eslintrc.cjs` or `biome.json`). May require fixes across all `packages/*/src/` and `apps/desktop/src/` for existing violations. |
| Forbidden | Do not modify `crates/*` or Rust code. Do not change component behavior. |
| Risk | Low. Linter setup is additive. Initial run may surface many warnings that need triage (warn vs. error). |
| Prerequisites | None. |
| Acceptance criteria | `pnpm run lint` succeeds with zero errors. Warnings are documented or tracked. Config file is committed. CI can run the lint step. |
| Independent commit | Yes |

---

## TASK-04: Add vitest for frontend unit testing

| Field | Value |
|---|---|
| Goal | Set up vitest as the frontend test runner. Write initial tests for `useWorkspaceStore` (Zustand store), `request-utils.ts` (API debugger utilities), and `result-utils.ts` (database result utilities). |
| Scope | Root `package.json` (add vitest devDependency + test script). New `vitest.config.ts` in `apps/desktop` or root. Initial test files in targeted packages. |
| Forbidden | Do not modify production code to accommodate tests. Do not modify `crates/*`. |
| Risk | Low. Test infrastructure is additive. |
| Prerequisites | TASK-03 (linter) is recommended first but not required. |
| Acceptance criteria | `pnpm run test` runs vitest and all initial tests pass. At least 3 test files covering pure utility functions and Zustand store logic. |
| Independent commit | Yes |

---

## TASK-05: Wire real SSH transport via russh

| Field | Value |
|---|---|
| Goal | Connect the `russh` SSH library (behind `ssh-native` feature) to the existing `SshService` session lifecycle. Replace the in-memory simulation with real SSH channel I/O: TCP connection, authentication (password and private key), PTY allocation, bidirectional data streaming, and graceful disconnection. |
| Scope | `crates/ssh-engine/src/ssh.rs` (primary). May need changes in `apps/desktop/src-tauri/src/command_bus.rs` for async event streaming. Frontend `packages/terminal/` may need Tauri event listeners for real-time terminal output. |
| Forbidden | Do not restructure the `SshService` public API (keep existing command signatures). Do not modify `packages/command-client/src/types.ts` (types are already correct). |
| Risk | High. SSH transport involves async I/O, error handling, host-key verification, key format parsing, and cross-platform considerations. The `russh` API may require significant adaptation. |
| Prerequisites | SSH host-key verification strategy decided (ISSUE from security.md). Windows NASM dependency resolved (already handled via `ring` backend). |
| Acceptance criteria | `cargo check -p unfour-workspace --features ssh-native` passes. Connecting to a real SSH server (e.g., localhost with OpenSSH) succeeds. Terminal output appears in xterm. Session close cleans up resources. Existing tests still pass. |
| Independent commit | Yes (can be split into: transport layer, auth, PTY, event streaming) |

---

## TASK-06: Add PostgreSQL/MySQL live connection support

| Field | Value |
|---|---|
| Goal | Implement live `test_connection`, `schema`, `execute_query`, and `browse_table` for PostgreSQL and MySQL drivers in `DatabaseService`. Use connection pooling via `sqlx` with credential references from `SecretStore`. |
| Scope | `crates/database-engine/src/database.rs` (primary). May need changes in `command_bus.rs` to pass `SecretStore` reference to `DatabaseService`. |
| Forbidden | Do not change the SQLite implementation. Do not modify frontend types or components. |
| Risk | High. Requires managing connection pools for multiple database types, handling authentication via credential store, and testing against live PostgreSQL/MySQL instances. |
| Prerequisites | `SecretStore.read_secret()` integration in `CommandBus` for database connections. Test infrastructure for PostgreSQL and MySQL (Docker or CI services). |
| Acceptance criteria | `test_connection` succeeds for a live PostgreSQL instance. Schema browsing returns tables and columns. SQL execution works for read queries. Mutation confirmation policy applies. Existing SQLite tests still pass. |
| Independent commit | Yes (can be split into: PostgreSQL, then MySQL) |

---

## TASK-07: Implement Vite code splitting [COMPLETED]

| Field | Value |
|---|---|
| Status | Completed in `chore(frontend): complete hardening cleanup batch` |
| Result | Configured `manualChunks` in `vite.config.ts`. Production build produces 5 JS chunks: monaco (15 KB), vendor-radix (88 KB), vendor-tanstack (101 KB), xterm (334 KB), index (378 KB). All under 500 KB. Vite chunk size warning eliminated. |

---

## TASK-08: Add Rust tests for local-storage and CommandBus

| Field | Value |
|---|---|
| Goal | Add unit tests for `LocalDb::migrate()` (migration idempotency), `ActivityLogService::record()` (event recording and retrieval), and `CommandBus` orchestration (at least 2-3 end-to-end command flows using in-memory SQLite). |
| Scope | `crates/local-storage/src/` (add test module). `apps/desktop/src-tauri/src/` (add test module or integration test). |
| Forbidden | Do not modify production code unless needed for testability (e.g., making `LocalDb::from_pool` public, which it already is). |
| Risk | Low. Tests are additive. `LocalDb::from_pool` already exists for in-memory SQLite testing. |
| Prerequisites | None. |
| Acceptance criteria | `cargo test --workspace` passes with new tests. `unfour-local-storage` has at least 2 tests. `unfour-workspace` has at least 2 integration tests. |
| Independent commit | Yes |

---

## TASK-09: Remove deprecated module-boundaries.md [COMPLETED]

| Field | Value |
|---|---|
| Status | Completed in `chore(frontend): complete hardening cleanup batch` |
| Result | `docs/architecture/module-boundaries.md` deleted. No remaining references in source code; checkpoint docs updated. |

---

## TASK-10: Wire terminal search with xterm search addon

| Field | Value |
|---|---|
| Goal | Connect `TerminalSearchBar` to the `@xterm/addon-search` addon. Implement forward/backward search, highlight current match, and keyboard navigation. |
| Scope | `packages/terminal/src/components/TerminalSearchBar.tsx`, `packages/terminal/src/components/TerminalPane.tsx` (attach addon). |
| Forbidden | Do not modify `crates/*` or other packages. |
| Risk | Low. The addon is already installed; wiring is straightforward. |
| Prerequisites | xterm instance must be accessible from `TerminalSearchBar` (via ref or store). |
| Acceptance criteria | Typing in the search bar highlights matches in the terminal. Enter/Shift+Enter navigates between matches. "Search integration pending" placeholder is removed. |
| Independent commit | Yes |

---

## Recommended Priority Order

1. **TASK-05** (real SSH) -- highest-value feature gap, core roadmap item
2. **TASK-03** (linter) -- establishes quality gate for all subsequent work
3. **TASK-04** (vitest) -- testing infrastructure for frontend packages
4. **TASK-08** (Rust tests) -- backend test coverage for untested services
5. **TASK-06** (PostgreSQL/MySQL) -- second-highest feature gap
6. **TASK-10** (terminal search) -- small feature completion
