# Unfour UI Layouts

Unfour uses one desktop application shell. The shell is structural; modules
provide content, trees, inspectors, and bottom-panel output through slots.

## Application Shell

```text
GlobalToolbar 38px
├─ Sidebar 240-320px
├─ MainWorkspace
│  ├─ TabBar 34px
│  └─ Active module surface
├─ RightInspector optional 260-420px
├─ BottomPanel optional 180-360px
└─ StatusBar 24px
```

## Global Toolbar

Contains only global operations:

- workspace switcher
- back, forward, home
- global search and command palette
- layout toggles
- sync/storage/status indicators
- window controls in desktop runtime

It must not contain module-specific actions such as Send, Run SQL, Connect,
Save request, or Add table.

## Sidebar

The sidebar contains navigation and trees:

- module switcher
- API collections
- database connections and schema tree
- SSH hosts and sessions

Forms and editors do not belong in the sidebar. Use dialogs, inspectors, or
main tabs for edit-heavy workflows.

## Main Workspace

The main workspace owns a single shared `TabBar`. Feature modules render below
the tab bar and should not create their own outer page shell.

## Right Inspector

The inspector is optional and collapsible. It is for secondary details such as
metadata, request history, connection properties, and environment variables.

Width must be state-driven so it can later be persisted per workspace.

## Bottom Panel

The bottom panel is optional and collapsible. It is for logs, traces,
diagnostics, and task output.

Height must be state-driven so it can later be persisted per workspace.

## Status Bar

The status bar is compact and always visible. It shows workspace, storage,
sync, connection, and background-task status.
