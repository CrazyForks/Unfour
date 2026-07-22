# Changelog

This file is the user-facing change history for Unfour, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html)

## [Unreleased]

## [0.2.0] - 2026-07-22

Minor release focused on SSH file transfer and task automation, shared workspace
variables, and multi-statement Database execution.

### Added

- **SSH SFTP remote files** — Browse remote directories, transfer files, and manage
  remote paths from a dedicated SFTP panel with context menus, multi-select, and
  drag-and-drop upload.
- **SSH Task automation** — Create and run multi-step SSH tasks (command, upload,
  download) with workspace-scoped sync-safe templates, local bindings, run
  history, streamed transcripts, and Save / Run editor UX.
- **Task run inputs from workspace env** — Prefill task run placeholders from the
  active workspace environment and echo the executed commands in the run output.
- **Shared workspace variables** — Promote API environments to workspace-scoped
  variables with title-bar active-environment switching; API request resolution
  overlays workspace defaults.
- **`@unfour/workspace-environments` package** — Extract environment/variable
  management into a dedicated frontend package mounted from the app shell, with
  module navigation and dirty-leave confirmation while editing.
- **Database multi-statement Run** — Split editor SQL on semicolons and run
  Current / All statements sequentially, showing multiple result sets as
  sub-tabs.

### Fixed

- **SFTP transfer UI** — Keep transfer progress in sync and improve transfer
  throughput.
- **SSH task downloads** — Prevent task downloads from overwriting local
  directories; treat missing SSH exit status as failure so failed steps are not
  marked success.
- **SSH task navigation** — Improve task workspace navigation and keep module
  shortcuts scoped so hidden modules do not steal focus (e.g. API Ctrl+S).
- **Database table preview** — Stabilize table preview loading and remove
  placeholder loading rows that could flash incorrect grid content.

### Changed

- **API environments ownership** — Environment CRUD and storage move from the
  API Client / http-engine path to workspace variables in `workspace-engine`
  and the new environments package; API Client consumes the shared workspace
  active environment.
- **SSH module surface** — SSH Terminal sidebar gains Connections / Files /
  Tasks modes for terminal, SFTP, and task automation workflows.

### Docs

- Updated architecture docs for workspace variables package boundaries, data
  storage, and project structure.
- Updated SSH Terminal and API Client package docs for the new surfaces.

## [0.1.2] - 2026-07-20

Feature and reliability release focused on API interoperability, Database row
editing, and MCP/SSH stability.

### Added

- **OpenAPI collection export** — Export API collections as OpenAPI 3.1 from the
  collection toolbar, with shared dialog and tree actions in the API Client.
- **OpenAPI YAML import** — Import OpenAPI YAML into API collections through the
  http-engine OpenAPI import path.
- **Database table row editing** — Edit table rows with confirmation gating,
  optimistic concurrency checks, and bind-parameter SQL updates.
- **MCP API environment CRUD** — Manage API environments through MCP tools over
  the command bus.
- **Named secret operations** — Secret store supports named secret read/write
  helpers for credential-reference workflows.
- **Data grid UX** — Column resizing and JSON preview improvements in the shared
  data table / Database table grid.

### Fixed

- **SSH failed sessions** — Failed SSH session tabs and connection errors are
  preserved instead of being discarded silently.
- **MCP idle shutdown** — Idle shutdown is disabled by default so long-lived MCP
  clients are not interrupted unexpectedly.

### Changed

- **Database workspace controller** — Split Database page orchestration into
  dedicated hooks and connection/tree helpers while preserving existing query
  and schema flows.

### Refactored

- Split oversized backend modules into focused directories across
  `database-engine`, `http-engine` (api_client), `ssh-engine`,
  `unfour-command-bus`, and `unfour-mcp` tool handlers. Behavior is unchanged
  aside from the features listed above.
- Removed obsolete workspace implementation leftovers from the earlier
  workspace boundary cleanup.

### Docs

- Updated README screenshots and product overview copy for the current desktop
  modules.
- Documented MCP environment tools and idle-shutdown default in MCP docs.

## [0.1.1] - 2026-07-13

Maintenance and polish release following the 0.1.0 public launch.

### Added

- **Desktop extension slots** — The app shell now exposes module mount surfaces
  and extension slots (`packages/app-shell/src/extensions.ts`), enabling future
  pluggable desktop features without touching core layout code.
- **Release `core_commit` identity** — App system info and the About panel now
  surface the built `core_commit`, and the Community release identity config is
  unified across the build pipeline (`release.yml`, `build.rs`, `app.rs`).
- **Generic deep-link runtime support** — Deep links now resolve at runtime
  without hardcoded scheme handling.
- **i18n resource loading** — Extended the shared i18n provider to load
  additional resource bundles and added provider tests.

### Changed

- **Windows distribution** — The build now packages only the NSIS installer and
  drops the MSI requirement, simplifying the upgrade story (see
  `docs/release/distribution.md`).

### Fixed

- **Settings dialog** — Enlarged the settings window and removed the MCP tab
  height flash on open.

### Refactored

- **File-size discipline** — Split oversized source files into module
  directories across `unfour-core` (models), `unfour-mcp` (ssh tools),
  `workspace-engine`, `api-client`, `command-client` (types), and `packages/ui`
  (shell, tree-view). Behavior is unchanged; this improves maintainability and
  keeps the CI large-file gate green.
- **Shared styles** — Moved global styles out of `apps/desktop/src/styles.css`
  into dedicated `packages/app-shell/src/styles` modules (animations, host,
  index) and tightened the shared-token checks.

### Docs

- Marked API, SQLite, SSH, PostgreSQL, MySQL, and MCP release-verification
  checks as PASS.
- Updated README and distribution/release documentation to reflect the NSIS-only
  Windows packaging.

## [0.1.0] - 2026-07-09

First public release.

### Added

- **API Client** — Compose, send, save, and inspect HTTP requests with workspace
  environments and redacted history.
- **SSH Terminal** — Manage SSH connections and terminal sessions with split
  panes, host-key trust, and redacted log export.
- **Database** — Manage connections, browse schemas, run SQL with confirmation
  guardrails, and preview query results.
- **Workspace** — Scope requests, environments, connections, activity, tabs, and
  layout to a local workspace with unique names and per-workspace persistence.
- **Local MCP server** — Expose safe local diagnostic tools (API replay, SSH
  connection) to MCP clients over the command bus.
- **App shell & platform** — Single-instance app, settings window, structured
  local logs, centralized design tokens, and shared i18n.

### Security

- Credentials stored as references only; sensitive headers redacted in history,
  activity, and logs; keychain purged on connection delete; MCP tools reject
  forbidden write/control operations.

### Known limitations

- Signing is not yet complete; unsigned artifacts may trigger OS warnings.
- Windows distributes both NSIS `.exe` and MSI `.msi` for the same version. NSIS
  is recommended for ordinary users; MSI is available for MSI preference or
  software deployment management. Choose one format because installing both
  may create duplicate shortcuts or uninstall entries and confuse upgrades.
- Cross-format detection, automatic uninstall, and NSIS/MSI cross-upgrade are
  not implemented at this stage.
- macOS and Linux artifacts remain experimental/unverified until real-device
  smoke checks are complete.

[0.2.0]: https://github.com/zyqzyq/Unfour/releases/tag/v0.2.0
[0.1.2]: https://github.com/zyqzyq/Unfour/releases/tag/v0.1.2
[0.1.1]: https://github.com/zyqzyq/Unfour/releases/tag/v0.1.1
[0.1.0]: https://github.com/zyqzyq/Unfour/releases/tag/v0.1.0
