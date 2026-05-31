# Architecture

Unfour Workspace is a local-first Tauri 2 app built around a shared Workspace model. SSH, database, and API workflows reuse the same resource tree, tab system, credential boundary, audit log, and future sync model.

## Runtime Split

- React frontend owns layout, tabs, forms, editor surfaces, terminal rendering, table rendering, and user feedback.
- Rust backend owns filesystem access, SQLite, credential references, HTTP execution, future SSH/database connections, logs, and policy checks.
- Tauri IPC is an adapter layer. It should remain thin.

## Command Flow

```text
React UI
  -> Tauri command
  -> CommandBus
  -> Service
  -> Adapter / Driver
```

The same `CommandBus` shape is reserved for later AI, MCP, CLI, workflow runner, and cloud automation adapters.

## Current Modules

- `command_bus`: single backend entrypoint for app actions.
- `workspace`: workspace CRUD, active workspace state, and workspace-scoped environment variables.
- `api_client`: HTTP/HTTPS execution, workspace variable resolution, saved requests, request history.
- `local_db`: SQLite connection and migrations.
- `audit_log`: append-only action trail with redacted details.
- `ssh`: reserved boundary for `russh` sessions and event streaming. The dependency is available behind the `ssh-native` feature and uses the `ring` backend to avoid NASM on Windows.
- `database`: reserved boundary for `sqlx` connection pools and SQL execution.
- `secret_store`: reserved boundary for OS keychain/Stronghold credential refs.
- `ai_reserved`: command/capability types for future AI invocation.
- `sync_reserved`: local-first sync metadata policy.

## Frontend Shape

- `src/App.tsx`: workspace shell and current MVP panels.
- `src/lib/tauri.ts`: typed Tauri command adapter with browser-dev mocks.
- `src/store/workspace-store.ts`: UI state for active workspace/tab/sidebar.
- `src/components/ui/*`: shadcn-style local primitives.

## Design Constraint

Do not create independent "API app", "SSH app", and "Database app" islands. New features should plug into Workspace resources, tabs, history, and credential references.
