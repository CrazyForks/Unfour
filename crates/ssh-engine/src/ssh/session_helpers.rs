use super::*;

pub(super) fn session_for_workspace_mut<'a>(
    sessions: &'a mut HashMap<String, SshSessionState>,
    workspace_id: &str,
    session_id: &str,
) -> AppResult<&'a mut SshSessionState> {
    sessions
        .get_mut(session_id)
        .filter(|state| state.summary.workspace_id == workspace_id)
        .ok_or_else(|| AppError::NotFound("ssh session".to_string()))
}

#[cfg(feature = "ssh-native")]
pub(super) fn session_for_workspace<'a>(
    sessions: &'a HashMap<String, SshSessionState>,
    workspace_id: &str,
    session_id: &str,
) -> AppResult<&'a SshSessionState> {
    sessions
        .get(session_id)
        .filter(|state| state.summary.workspace_id == workspace_id)
        .ok_or_else(|| AppError::NotFound("ssh session".to_string()))
}

pub(super) fn ensure_session_active(state: &SshSessionState) -> AppResult<()> {
    if state.summary.status != "connected" {
        return Err(AppError::Validation(
            "ssh session is not connected".to_string(),
        ));
    }
    Ok(())
}

/// Push a session event, dropping the oldest entries once the in-memory cap is
/// exceeded. Session events are only used for `export_log`; trimming them keeps
/// a long-lived session from growing the event vector without bound.
pub(super) fn record_session_event(state: &mut SshSessionState, event: SshSessionEvent) {
    state.events.push(event);
    if state.events.len() > MAX_SESSION_EVENTS {
        let excess = state.events.len() - MAX_SESSION_EVENTS;
        state.events.drain(0..excess);
    }
}

/// Build a redacted session-log export from an event slice. Used for both live
/// sessions (events held in memory) and closed sessions (events hydrated from
/// the terminal-history store after the in-memory entry was dropped to bound
/// memory growth, see issue #4).
pub(super) fn build_ssh_log_export(session_id: &str, events: &[SshSessionEvent]) -> SshLogExport {
    let mut redacted = false;
    let lines = events
        .iter()
        .map(|event| {
            let (data, event_redacted) = redact_ssh_log(&event.data);
            redacted |= event_redacted;
            format!("[{}] {} {}", event.created_at, event.kind, data)
        })
        .collect::<Vec<_>>();
    let content = lines.join("\n");
    // Persisted (closed) sessions already had line-level redaction applied at
    // append time; reflect that in the redacted flag so closed-session exports
    // stay accurate.
    if content.contains("<redacted>") {
        redacted = true;
    }
    SshLogExport {
        session_id: session_id.to_string(),
        filename: format!("ssh-session-{}.log", session_id),
        line_count: lines.len(),
        content,
        redacted,
    }
}

pub(super) fn is_live_status(status: &str) -> bool {
    matches!(status, "connected" | "degraded" | "reconnecting")
}

#[cfg(any(feature = "ssh-native", test))]
pub(super) fn should_reconnect(state: &SshSessionState) -> bool {
    !state.intentional_close && is_live_status(&state.summary.status)
}

#[cfg(feature = "ssh-native")]
pub(super) fn native_client_config() -> russh::client::Config {
    let mut config = russh::client::Config::default();
    config.keepalive_interval = Some(KEEPALIVE_INTERVAL);
    config.keepalive_max = KEEPALIVE_MAX_MISSES;
    config.nodelay = true;
    config
}

pub(super) fn redact_ssh_log(value: &str) -> (String, bool) {
    redact_sensitive_lines(value)
}
