# ADR 0001: Tech Stack

## Status

Accepted.

## Decision

Use Tauri 2, React, TypeScript, Vite, shadcn-style components, Tailwind CSS, Zustand, TanStack Query, TanStack Table, Monaco Editor, and xterm.

Use Rust services behind a Command Bus. Use SQLite for local-first metadata, `reqwest` for API execution, `sqlx` for database work, and reserve `russh` for SSH. `russh` should use `default-features = false` with `ring` to keep Windows builds independent of NASM.

## Context

The product is a dense desktop operations tool. It needs terminal rendering, SQL/API editors, table-heavy views, local persistence, credentials, and future automation. React has the strongest ecosystem for these surfaces.

## Consequences

- The first bundle is larger because Monaco and xterm are included. Later work should dynamically load heavy panels.
- Tauri commands stay thin. Domain behavior belongs in Rust services.
- Local-first persistence is available before cloud sync.
- The repo expects modern stable Rust. It was verified after upgrading to `rustc 1.96.0`.
