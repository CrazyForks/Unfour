# Next Steps

## Recommended: SSH Phase 3 — Auth & Reliability

Priority order:

1. **Private-key authentication** — Support SSH key-based auth using russh's `authenticate_publickey` with key files from disk.
2. **Connection health monitoring** — Add keep-alive pings and detect dropped connections.
3. **Reconnection logic** — Auto-reconnect sessions when the SSH connection drops unexpectedly.
4. **Terminal session persistence** — Persist terminal output to SQLite so history survives app restarts.

## Lower Priority

- **Host-key UI** — Allow users to view, trust, or reset host-key fingerprints.
- **known_hosts integration** — Import or export fingerprints from/to the system `known_hosts` file.
- **Terminal multiplexing** — Support tmux/screen-like session management within the app.
- **SCP/SFTP file transfer** — Leverage the existing russh connection for file operations.
