<div align="center">

# Unfour

**One local-first desktop workspace for API debugging, SSH terminals, and database management.**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/zyqzyq/Unfour/actions/workflows/ci.yml/badge.svg)](https://github.com/zyqzyq/Unfour/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/zyqzyq/Unfour?include_prereleases&sort=semver)](https://github.com/zyqzyq/Unfour/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB.svg)](https://tauri.app)

</div>

> [!WARNING]
> **Unfour is in early alpha (`v0.1`).** Core flows work, but SSH sessions still
> need broad live-server verification and database SQL execution can run
> destructive statements. Do not point it at production systems yet. Expect
> breaking changes between releases.

---

## What is Unfour?

Unfour bundles the day-to-day tools of a backend / ops developer into
a single fast, local-first desktop app — no accounts, no cloud, your data stays
on your machine. It is built on [Tauri 2](https://tauri.app), React, TypeScript,
and Rust, so the UI is lightweight and the security-sensitive work (SSH,
database drivers, credential storage) runs in a Rust backend.

### Modules

- **🌐 API Debugger** — Compose and send HTTP requests, organize them into
  collections and folders, manage environments with `{{variable}}` resolution,
  inspect headers / cookies / timing, and keep a redacted request history.
- **🖥️ SSH Terminal** — Connect over SSH with password or private-key auth, run
  live terminal sessions with split panes and search, and export session logs.
- **🗄️ Database** — Browse connections and schemas, run SQL, preview tables, and
  review query history. SQLite today, with PostgreSQL / MySQL paths in progress.
- **🧰 Workspace** — Everything is scoped to a workspace: environment variables,
  layout, saved requests, and connections persist locally in SQLite.
- **🤖 Local MCP server** — A built-in [MCP](https://modelcontextprotocol.io)
  server exposes safe, read-only workspace / API / database tools to AI clients.

## Screenshots

> _Screenshots coming soon._ Drop images under `docs/assets/` and reference them
> here once the first public build is tagged.

<!--
![API Debugger](docs/assets/api-debugger.png)
![SSH Terminal](docs/assets/ssh-terminal.png)
![Database](docs/assets/database.png)
-->

## Install

### Download a prebuilt binary (recommended)

Grab the latest installer for your platform from the
[**Releases**](https://github.com/zyqzyq/Unfour/releases) page:

| Platform | File |
| -------- | ---- |
| Windows  | `.msi` / `.exe` |
| macOS    | `.dmg` |
| Linux    | `.AppImage` / `.deb` |

> Builds are not yet code-signed, so your OS may warn on first launch. Verify the
> download against the release checksums before running.

### Build from source

Requirements: a modern stable Rust toolchain (verified with `rustc 1.96.0`),
Node.js, and [pnpm](https://pnpm.io). See the Tauri
[prerequisites](https://tauri.app/start/prerequisites/) for OS-specific system
libraries.

```bash
git clone https://github.com/zyqzyq/Unfour.git
cd Unfour
pnpm install
pnpm run tauri dev      # run the app in development
pnpm run tauri build    # produce a release bundle
```

## Project Layout

| Path | Role |
| ---- | ---- |
| `apps/desktop` | Tauri/Vite desktop app entry and Tauri adapter layer. |
| `packages/*` | Frontend package boundaries imported as `@unfour/*`. |
| `crates/*` | Rust backend capability crates composed by the desktop app. |
| `Cargo.toml` | Cargo workspace for `apps/desktop/src-tauri` and the crates. |

Frontend interaction lives in React/TypeScript; execution and security
boundaries (HTTP, SSH, database, secret storage) live in Rust behind a command
bus. Credentials are kept in the OS keychain — never stored as SQLite plaintext.

## Development

```bash
pnpm install
pnpm run build          # build the frontend
pnpm run check          # frontend build + cargo check
pnpm run lint           # ESLint
pnpm run test           # frontend unit tests (Vitest)
pnpm run test:e2e       # Playwright end-to-end tests
pnpm run check:rust     # cargo check --workspace
pnpm run check:rust:ssh # cargo check with the ssh-native feature
pnpm run test:rust      # cargo test --workspace
```

Run Cargo commands from the repository root.

## Contributing

Contributions are welcome! Please read **[CONTRIBUTING.md](CONTRIBUTING.md)**
for the development setup, package boundary rules, and commit conventions, and
**[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)** before opening a pull request.

Found a security issue? Please follow **[SECURITY.md](SECURITY.md)** and do not
open a public issue.

## Documentation

- [`AGENTS.md`](AGENTS.md) — rules for coding agents (start here for AI tools).
- [`docs/architecture/package-boundaries.md`](docs/architecture/package-boundaries.md) — package boundaries.
- [`docs/project/PACKAGE_STATUS.md`](docs/project/PACKAGE_STATUS.md) — per-package / crate status.
- [`docs/user/USER_GUIDE.md`](docs/user/USER_GUIDE.md) — user-facing guide.
- [`docs/ai/mcp.md`](docs/ai/mcp.md) — local MCP server tools and usage.
- [`docs/roadmap.md`](docs/roadmap.md) — roadmap.

## License

Licensed under the [Apache License 2.0](LICENSE).
