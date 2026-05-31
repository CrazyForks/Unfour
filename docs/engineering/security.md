# Security

Unfour Workspace handles credentials, remote command execution, database access, and HTTP tokens. The default posture is local-first and least privilege.

## Sensitive Data

Do not store these in SQLite plaintext:

- SSH passwords
- SSH private-key passphrases
- Database passwords
- API tokens
- Proxy credentials

SQLite records should store only `credential_ref`.

## Redaction

Logs and request history redact these header names:

- `authorization`
- `cookie`
- `proxy-authorization`
- `x-api-key`
- `x-auth-token`

Body redaction is not implemented yet. Until it is, users should avoid saving secrets in API bodies.

## Tauri Capabilities

Keep `src-tauri/capabilities/default.json` narrow. Add permissions only when a command or plugin requires them, and record why in this document.

## Dangerous Actions

Future AI/workflow actions must ask for confirmation before:

- Writing to production databases
- Running destructive SSH commands
- Exporting workspaces with sensitive metadata
- Sending secrets to third-party services

## Current Gaps

- OS keychain/Stronghold write/read is reserved but not implemented.
- SSH host-key verification is not implemented.
- Database query cancellation and read-only guardrails are not implemented.
- API request body redaction is not implemented.
- Workspace environment values are not encrypted; do not store long-lived secrets there.
