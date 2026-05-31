# Unfour Workspace User Guide

This document is for people using the app. It avoids implementation details that belong in `docs/engineering`.

## Current Preview

Unfour Workspace opens into a single workspace surface:

- Workspaces on the left
- Tool tabs across the top
- API client, SSH terminal, and database panels in the center
- Local-first storage by default

## API Client

1. Select a workspace.
2. Open `API Client`.
3. Add workspace environment variables, such as `base_url` and `source`.
4. Use variables in requests with `{{base_url}}` syntax.
5. Choose an HTTP method.
6. Enter the URL.
7. Add query parameters and headers.
8. Edit the request body for non-GET requests.
9. Click `Send`.
10. Review status, duration, response body, and history.

Saved requests are stored inside the active workspace.

## SSH Terminal

The SSH screen is present as a preview. Real connection support is planned for the SSH MVP.

## Database

The database screen is present as a preview. PostgreSQL, MySQL/MariaDB, and SQLite are planned for the database MVP.

## Data And Privacy

The app is local-first. Workspace metadata is stored locally. Secret storage is reserved for OS keychain/Stronghold integration; until that work lands, do not place long-lived secrets into saved request bodies.

## Documentation Split

- This guide explains how to use the app.
- `docs/engineering` explains how the app is built.
- `AGENTS.md` explains how coding agents should work in this repository.
