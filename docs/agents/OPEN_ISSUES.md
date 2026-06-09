# Open Issues

## P0 — Blocks core usage

None.

## P1 — High priority

### SSH Transport

- **Private-key authentication:** Implemented for unencrypted keys. Encrypted key passphrase support is limited by ssh-key crate format constraints.
- **Connection health monitoring:** No heartbeat or keep-alive mechanism. Disconnected sessions are not detected automatically.
- **Reconnection logic:** If a native SSH connection drops, there is no automatic reconnection. Users must manually close and re-create sessions.

### Security

- **API body redaction:** Request body redaction is not implemented. Users should avoid saving secrets in API bodies.

## P2 — Medium priority

- **Terminal session persistence:** Terminal output events are stored in memory only. Refreshing the app loses terminal history.
- **Host-key UI:** View/reset fingerprint implemented. Mismatch rejection handled by TOFU backend. Confirmation dialog for accepting new fingerprints after reset not yet built.
- **known_hosts integration:** Import or export fingerprints from/to the system `known_hosts` file.
- **Encrypted key passphrase:** The ssh-key crate 0.7.0-rc.10 has limited support for decrypting encrypted OpenSSH keys. Keys without passphrases work. Encrypted key loading returns a clear error guiding users to save a passphrase credential.

## P3 — Low priority / Future

- **Terminal multiplexing:** Support tmux/screen-like session management within the app.
- **SCP/SFTP file transfer:** Leverage the existing russh connection for file operations.
- **Additional database drivers:** PostgreSQL and MySQL support.

## Environment / Tooling

- **OS keychain:** The `keyring` crate is used for production but has not been verified on all target platforms (macOS, Windows, Linux).
- **Windows workspace tests:** `cargo test -p unfour-workspace` fails with `STATUS_ENTRYPOINT_NOT_FOUND` on this Windows environment. Likely a native DLL dependency issue, not a code defect.
- **Real SSH connection verification:** The native transport path with PTY streaming has not been verified against a live SSH server in this environment. Manual verification is recommended.

## Summary

- P0: 0
- P1: 3
- P2: 4
- P3: 3
- Environment: 3
