# Open Issues

## SSH Transport

- **Private-key authentication:** Not implemented. Only password auth via SecretStore is supported under `ssh-native`.
- **Connection health monitoring:** No heartbeat or keep-alive mechanism. Disconnected sessions are not detected automatically.
- **Reconnection logic:** If a native SSH connection drops, there is no automatic reconnection. Users must manually close and re-create sessions.
- **Terminal session persistence:** Terminal output events are stored in memory only. Refreshing the app loses terminal history.

## Security

- **OS keychain:** The `keyring` crate is used for production but has not been verified on all target platforms (macOS, Windows, Linux).
- **API body redaction:** Request body redaction is not implemented. Users should avoid saving secrets in API bodies.

## General

- **Real SSH connection verification:** The native transport path with PTY streaming has not been verified against a live SSH server in this environment. Manual verification is recommended.
