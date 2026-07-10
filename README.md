<div align="center">

[English](README.md) · [简体中文](README.zh-CN.md)

# Unfour

**A local-first desktop workspace for backend developers that combines API debugging, SSH terminals, and database management — and exposes them to your AI agent through a local MCP server.**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/zyqzyq/Unfour/actions/workflows/ci.yml/badge.svg)](https://github.com/zyqzyq/Unfour/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/zyqzyq/Unfour?include_prereleases&sort=semver)](https://github.com/zyqzyq/Unfour/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB.svg)](https://tauri.app)

</div>

> [!WARNING]
> Unfour `v0.1.0` is an early pre-release. It is not recommended for production
> use. Installers are unsigned and may trigger SmartScreen or other operating-
> system security warnings; validate the release against your own requirements.

## Download

Download the current [`v0.1.0` pre-release from GitHub Releases](https://github.com/zyqzyq/Unfour/releases/tag/v0.1.0).

- Windows: NSIS `.exe` is recommended for ordinary users. MSI `.msi` is also
  available for users who prefer MSI or need software deployment management;
  both install the same Unfour version, so choose one.
- Do not install both Windows formats on the same device: duplicate shortcuts or
  uninstall entries and confusing upgrade paths may result. Cross-format
  detection, automatic uninstall, and cross-upgrade are not implemented yet.
- macOS and Linux packages are experimental and unverified until real-device
  smoke checks are recorded; do not treat them as supported or verified yet.
- Verify downloaded installers with the release `SHA256SUMS.txt` asset.

## What Is Unfour?

Unfour is a local-first desktop workspace for backend and operations work.
It keeps API requests, SSH connections, database connections, local activity,
and workspace layout in one local-first application, and exposes those
capabilities to your AI agent through a local MCP server. AI-assisted
troubleshooting workflows built on this foundation are planned.

The app is built with Tauri 2, React, TypeScript, and Rust. The frontend owns
the workbench UI, while security-sensitive execution such as HTTP, SSH,
database drivers, local storage, and credential references lives behind Rust
capability crates and the command bus.

## Modules

- **API Client** - Compose and send HTTP requests, organize saved requests into
  collections and folders, resolve workspace environments, inspect response
  body/headers/cookies/timing, and keep redacted history.
- **SSH Terminal** - Manage SSH connections, start terminal sessions, use split
  panes and search, handle host-key trust, and export redacted session logs.
- **Database** - Manage database connections, browse schemas, run SQL with
  confirmation-aware safety checks, preview table data, and review query output.
- **Workspace** - Scope saved requests, environments, connections, activity,
  tabs, and layout state to a local workspace.
- **Local MCP server** - Expose safe local diagnostic tools to MCP clients
  (such as Codex, Claude Code, or Cursor) through the same command bus used by
  the desktop app, so your AI agent can work with the same API, SSH, and
  database context.

## Local Development

Requirements:

- Node.js and pnpm.
- A stable Rust toolchain.
- Tauri 2 prerequisites for your operating system.

Install and run:

```bash
pnpm install
pnpm run dev
```

Common commands:

```bash
pnpm run build          # build the desktop frontend
pnpm run check          # frontend build + Rust check + large-file check
pnpm run lint           # ESLint
pnpm run test           # frontend unit tests (Vitest)
pnpm run test:e2e       # Playwright smoke tests
pnpm run check:rust     # cargo check --workspace
pnpm run check:rust:ssh # cargo check with the ssh-native feature
pnpm run test:rust      # cargo test --workspace
pnpm run tauri build    # create Tauri release bundles
```

Run commands from the repository root unless a package document says otherwise.

## Project Layout

| Path | Role |
| --- | --- |
| `apps/desktop` | Tauri/Vite desktop app entry and Tauri adapter layer. |
| `packages/app-shell` | Global shell composition and module mount slots. |
| `packages/api-client` | API Client frontend module. |
| `packages/ssh-terminal` | SSH Terminal frontend module. |
| `packages/database` | Database frontend module. |
| `packages/workspace-core` | Shared frontend workspace state. |
| `packages/workspace-local` | Reserved local workspace lifecycle boundary. |
| `packages/ui` | Shared UI primitives and stateless layout helpers. |
| `packages/command-client` | Typed Tauri command wrappers and frontend command types. |
| `crates/*` | Rust backend capability crates and adapters. |

See `docs/architecture/project-structure.md` for the full package and crate
map.

## Release Status

The current published version is `v0.1.0 pre-release`. This is an early
pre-release, and release readiness is limited by the verification evidence in:

- `docs/testing/release-verification.md`
- `docs/testing/manual-test-cases.md`
- `docs/release/release-checklist.md`
- `docs/release/distribution.md`
- `docs/release/signing.md`

Windows distribution provides NSIS `.exe` and MSI `.msi`. NSIS is recommended
for ordinary users; MSI is for MSI preference or software deployment
management. Choose one format. Installers are unsigned and may trigger
SmartScreen. Do not use this pre-release in production. macOS and Linux remain
experimental/unverified until real-device smoke checks are complete.
Do not claim a release check passes unless it was run successfully for the
target platform or is backed by current repository evidence.

## Documentation

- `AGENTS.md` - repository rules for coding agents.
- `docs/agents/START_HERE.md` - scoped onboarding path for AI agents.
- `docs/architecture/package-boundaries.md` - package ownership and forbidden
  dependency directions.
- `docs/architecture/project-structure.md` - repository, package, crate, and
  call-chain map.
- `docs/architecture/data-storage.md` - workspace data, SQLite, credential
  references, and local activity rules.
- `docs/architecture/diagnostics.md` - local structured logs, redaction,
  retention, diagnostic bundles, and developer logging guidance.
- `docs/architecture/security-model.md` - security posture, redaction, host-key
  policy, and dangerous-action rules.
- `docs/mcp/overview.md` and `docs/mcp/tools.md` - local MCP server behavior.
- `docs/testing/release-verification.md` - release verification matrix.
- `docs/release/release-checklist.md` - public release checklist.
- `docs/user/USER_GUIDE.md` - user-facing workflow guide.

## Contributing

Please read `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and the package boundary
rules in `AGENTS.md` before opening a pull request.

Security issues should be reported through `SECURITY.md`, not a public issue.

## License

Licensed under the [Apache License 2.0](LICENSE).
