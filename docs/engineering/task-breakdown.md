# Task Breakdown

## P0: Engineering Base

- DONE TASK-CORE-001: Initialize Tauri 2 + React + TypeScript + Vite.
- DONE TASK-CORE-002: Add shadcn-style UI primitives and workspace shell.
- DONE TASK-CORE-003: Add Rust `AppError` and structured command responses.
- DONE TASK-CORE-004: Add Command Bus and route Tauri commands through it.
- DONE TASK-CORE-005: Add SQLite local store and migrations.
- DONE TASK-CORE-006: Reserve secret store boundary.
- DONE TASK-CORE-007: Add audit log and redaction rules.
- TODO TASK-CORE-008: Add automated tests for WorkspaceService and ApiClientService.

## P0: Workspace

- DONE TASK-WORKSPACE-001: Create default workspace on first launch.
- DONE TASK-WORKSPACE-002: List, create, switch, rename, and soft-delete workspaces.
- PARTIAL TASK-WORKSPACE-003: Persist environment variables. Layout persistence remains.
- TODO TASK-WORKSPACE-004: Restore tabs per workspace.

## P0: API MVP

- DONE TASK-API-001: Send HTTP/HTTPS requests through Rust `reqwest`.
- DONE TASK-API-002: Support method, URL, headers, query, JSON body, and timeout.
- DONE TASK-API-003: Store request history by workspace.
- DONE TASK-API-004: Save request templates by workspace.
- DONE TASK-API-005: Add workspace environments and variable resolution.
- TASK-API-006: Add import/export for collections without secrets.

## P1: SSH MVP

- TASK-SSH-001: Add `russh` dependency compatible with the selected Rust toolchain.
- TASK-SSH-002: Password auth returns `session_id`.
- TASK-SSH-003: Private-key auth using local key path and passphrase ref.
- TASK-SSH-004: PTY allocation, xterm input, resize, and event output.
- TASK-SSH-005: Session close and log export with redaction.

## P1: Database MVP

- TASK-DB-001: Connection metadata CRUD with `credential_ref`.
- TASK-DB-002: SQLite connection test.
- TASK-DB-003: PostgreSQL/MySQL connection tests.
- TASK-DB-004: Schema tree for tables and columns.
- TASK-DB-005: SQL editor execution and paginated results.
- TASK-DB-006: Read-only table data view, then controlled edit support.

## P2: Reserved Extensions

- TASK-AI-001: Expose Command Bus through an AI adapter.
- TASK-SYNC-001: Add cloud account model and workspace sync queue.
- TASK-SYNC-002: Conflict UI that keeps both versions.
- TASK-PLUGIN-001: Define extension points for future tools.
