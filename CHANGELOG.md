# Changelog

This file is the user-facing change history for Unfour, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html)

## [Unreleased]

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

[0.1.1]: https://github.com/zyqzyq/Unfour/releases/tag/v0.1.1
[0.1.0]: https://github.com/zyqzyq/Unfour/releases/tag/v0.1.0
