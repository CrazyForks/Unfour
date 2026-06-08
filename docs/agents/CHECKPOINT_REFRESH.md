# Checkpoint Refresh

Use this procedure to refresh repository progress documents after several completed task batches or after significant cross-layer changes.

## Read First

```text
AGENTS.md
docs/agents/START_HERE.md
docs/agents/EXECUTION_PROTOCOL.md
```

## Goal

Refresh the repository checkpoint based on the current codebase and real verification results.

Only update:

```text
docs/agents/PROJECT_STATE.md
docs/agents/OPEN_ISSUES.md
docs/agents/NEXT_STEPS.md
```

Do not modify source code, configuration, dependencies, or other documentation.

## Scan

Inspect the current repository state:

```bash
git status --short
git log --oneline -10
```

Review:

```text
apps/*
packages/*
crates/*
docs/agents/*
docs/architecture/*
docs/ui/*
docs/engineering/*
```

Confirm:

```text
- implemented capabilities
- partially implemented capabilities
- not-started capabilities
- resolved issues
- new observed issues
- placeholders and stubs
- frontend lint and test status
- Rust test status
- build output and bundle chunks
- feature flags
- package boundaries
- security gaps
```

Do not reuse stale checkpoint content without verifying it against the repository.

## Verification

Run the commands that currently exist in the repository:

```bash
git status --short
pnpm run lint
pnpm run test
pnpm run build
cargo fmt --check
cargo test --workspace
cargo check --workspace
cargo check -p unfour-workspace --features ssh-native
```

If a command does not exist, record:

```text
N/A
```

If a command cannot run in the current environment, record:

```text
NOT RUN
```

Include the reason. Never report unexecuted checks as passing.

## Update PROJECT_STATE.md

Refresh:

```text
- scan metadata
- branch
- current commit
- working tree state
- tech stack
- current phase
- verified capabilities
- partially implemented items
- not-started items
- verification results
- known limitations
```

Use only current repository facts and real command results.

## Update OPEN_ISSUES.md

Refresh the issue list:

```text
- remove resolved issues
- update changed issues
- add newly confirmed issues
- distinguish Observed from Inferred
- recalculate P0 / P1 / P2 / P3 counts
- ensure summary counts match the issue list
```

Do not add speculative issues.

## Update NEXT_STEPS.md

Refresh the task list:

```text
- mark completed tasks
- remove obsolete tasks
- merge overly small related tasks
- keep high-risk work split into verifiable phases
- reorder priorities
```

Each pending task should include:

```text
- Goal
- Scope
- Forbidden
- Risk
- Prerequisites
- Acceptance criteria
- Independent commit
- Recommended model
```

Use one of:

```text
Codex / stronger coding model
weaker cheaper model is sufficient
```

Recommend a stronger model for:

```text
- Rust async
- russh
- CommandBus
- SecretStore
- storage schema
- database live drivers
- cross-layer event streaming
- security-sensitive changes
- complex multi-file refactors
```

A weaker model is sufficient for:

```text
- documentation
- checkpoint refresh
- README updates
- simple UI text
- semantic token replacement
- simple placeholder wiring
- scan and summary
- lint warning classification
```

## Allowed Scope

```text
docs/agents/PROJECT_STATE.md
docs/agents/OPEN_ISSUES.md
docs/agents/NEXT_STEPS.md
```

## Out of Scope

Do not modify:

```text
AGENTS.md
docs/agents/*
docs/architecture/*
docs/ui/*
docs/engineering/*
apps/*
packages/*
crates/*
package.json
pnpm-lock.yaml
Cargo.toml
Cargo.lock
```

Do not fix issues discovered during the scan. Record them only.

## Commit

Create one independent commit:

```text
docs(checkpoint): refresh repository state
```

## Final Report

Follow `docs/agents/EXECUTION_PROTOCOL.md`.

Also include:

```text
## Checkpoint Summary
- previous state vs current state
- resolved issues
- new issues
- next recommended batch
- recommended model for the next batch
```
