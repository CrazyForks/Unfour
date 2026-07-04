use super::super::*;
use super::support::{password_input, service_with_workspaces};

#[tokio::test]
async fn ssh_connection_crud_is_workspace_scoped_and_soft_deletes() {
    let (service, workspace_a, workspace_b) = service_with_workspaces().await;

    let created = service
        .save_connection(password_input(&workspace_a))
        .await
        .expect("save ssh connection");
    assert_eq!(created.host, "example.internal");
    assert_eq!(created.port, 22);
    assert_eq!(created.username, "deploy");
    assert_eq!(created.credential_ref.as_deref(), Some("ssh-password-1"));

    let workspace_a_items = service
        .list_connections(workspace_a.clone())
        .await
        .expect("list workspace a");
    let workspace_b_items = service
        .list_connections(workspace_b)
        .await
        .expect("list workspace b");
    assert_eq!(workspace_a_items.len(), 1);
    assert!(workspace_b_items.is_empty());

    let updated = service
        .save_connection(SshConnectionInput {
            id: Some(created.id.clone()),
            name: "Deploy bastion".to_string(),
            port: Some(2222),
            ..password_input(&workspace_a)
        })
        .await
        .expect("update ssh connection");
    assert_eq!(updated.name, "Deploy bastion");
    assert_eq!(updated.port, 2222);
    assert_eq!(updated.sync_status, "pending");

    let remaining = service
        .delete_connection(workspace_a.clone(), created.id)
        .await
        .expect("delete ssh connection");
    assert!(remaining.is_empty());
    assert!(service
        .list_connections(workspace_a)
        .await
        .expect("list after delete")
        .is_empty());
}

#[tokio::test]
async fn ssh_connection_validation_keeps_secrets_out_of_config() {
    let (service, workspace_id, _) = service_with_workspaces().await;

    let missing_credential = service
        .save_connection(SshConnectionInput {
            credential_ref: None,
            ..password_input(&workspace_id)
        })
        .await;
    assert!(matches!(missing_credential, Err(AppError::Validation(_))));

    let private_key = service
        .save_connection(SshConnectionInput {
            auth_kind: "private-key".to_string(),
            key_path: Some("C:/Users/zhang/.ssh/id_ed25519".to_string()),
            credential_ref: Some("ssh-key-passphrase-1".to_string()),
            ..password_input(&workspace_id)
        })
        .await
        .expect("save private key metadata");

    let stored_config: (String, i64, String, String, String, Option<String>) = sqlx::query_as(
        "SELECT c.host, c.port, sub.username, sub.auth_method, sub.config_json, c.credential_ref \
         FROM connections c \
         INNER JOIN ssh_connections sub ON sub.connection_id = c.id \
         WHERE c.id = ?1",
    )
    .bind(&private_key.id)
    .fetch_one(service.db.pool())
    .await
    .expect("load stored config");
    assert_eq!(stored_config.0, "example.internal");
    assert_eq!(stored_config.1, 22);
    assert_eq!(stored_config.2, "deploy");
    assert_eq!(stored_config.3, "private-key");
    assert!(stored_config.4.contains("id_ed25519"));
    assert!(!stored_config.4.contains("ssh-key-passphrase-1"));
    assert_eq!(stored_config.5.as_deref(), Some("ssh-key-passphrase-1"));

    let password_with_secret = service
        .save_connection(SshConnectionInput {
            credential_ref: None,
            secret: Some("plain-text-password".to_string()),
            ..password_input(&workspace_id)
        })
        .await
        .expect("save password credential through secret store");

    let password_config: (String, Option<String>) = sqlx::query_as(
        "SELECT sub.config_json, c.credential_ref \
         FROM connections c \
         INNER JOIN ssh_connections sub ON sub.connection_id = c.id \
         WHERE c.id = ?1",
    )
    .bind(password_with_secret.id)
    .fetch_one(service.db.pool())
    .await
    .expect("load stored password config");
    assert!(!password_config.0.contains("plain-text-password"));
    assert!(password_config.1.is_some());
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn ssh_session_lifecycle_supports_connect_input_resize_close_and_export() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");

    let session = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            cols: Some(100),
            rows: Some(30),
            secret: None,
        })
        .await
        .expect("connect ssh session");
    assert_eq!(session.connection_id, connection.id);
    assert_eq!(session.status, "connected");
    assert_eq!(session.cols, 100);

    let output = service
        .send_input(SshSessionInput {
            workspace_id: workspace_id.clone(),
            session_id: session.session_id.clone(),
            data: "echo ok\npassword=secret\n".to_string(),
        })
        .await
        .expect("send ssh input");
    assert_eq!(output.kind, "output");

    let resize = service
        .resize(SshResizeInput {
            workspace_id: workspace_id.clone(),
            session_id: session.session_id.clone(),
            cols: 120,
            rows: 40,
        })
        .await
        .expect("resize ssh pty");
    assert_eq!(resize.kind, "resize");

    let sessions = service
        .list_sessions(workspace_id.clone())
        .await
        .expect("list sessions");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].cols, 120);

    let closed = service
        .close_session(SshCloseInput {
            workspace_id: workspace_id.clone(),
            session_id: session.session_id.clone(),
        })
        .await
        .expect("close session");
    assert_eq!(closed.status, "disconnected");

    let rejected = service
        .send_input(SshSessionInput {
            workspace_id: workspace_id.clone(),
            session_id: session.session_id.clone(),
            data: "whoami\n".to_string(),
        })
        .await;
    assert!(matches!(rejected, Err(AppError::Validation(_))));

    let export = service
        .export_log(SshLogExportInput {
            workspace_id,
            session_id: session.session_id,
        })
        .expect("export log");
    assert!(export.content.contains("<redacted>"));
    assert!(!export.content.contains("password=secret"));
    assert!(export.redacted);
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn deleting_ssh_connection_closes_active_sessions() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");
    let session = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            cols: None,
            rows: None,
            secret: None,
        })
        .await
        .expect("connect ssh session");

    service
        .delete_connection(workspace_id.clone(), connection.id)
        .await
        .expect("delete connection");
    let sessions = service
        .list_sessions(workspace_id)
        .await
        .expect("list sessions after delete");
    assert_eq!(sessions[0].session_id, session.session_id);
    assert_eq!(sessions[0].status, "disconnected");
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn explicit_close_flushes_buffered_output_and_restore_lists_history() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");
    let session = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id,
            cols: None,
            rows: None,
            secret: None,
        })
        .await
        .expect("connect ssh session");

    {
        let mut sessions = service.sessions.lock().expect("lock sessions");
        sessions
            .get_mut(&session.session_id)
            .expect("session")
            .pending_output
            .push_str("buffered before close\r\n");
    }
    service
        .close_session(SshCloseInput {
            workspace_id: workspace_id.clone(),
            session_id: session.session_id.clone(),
        })
        .await
        .expect("close session");
    service.sessions.lock().expect("lock sessions").clear();

    let restored = service
        .list_sessions(workspace_id.clone())
        .await
        .expect("list persisted sessions");
    let history = service
        .session_history(SshCloseInput {
            workspace_id,
            session_id: session.session_id,
        })
        .await
        .expect("hydrate persisted history");
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].status, "disconnected");
    assert!(history[0].data.contains("buffered before close"));
    assert!(history[0].data.contains("SSH session closed."));
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn repeated_flush_without_new_output_does_not_duplicate_history() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");
    let session = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id,
            cols: None,
            rows: None,
            secret: None,
        })
        .await
        .expect("connect ssh session");

    {
        let mut sessions = service.sessions.lock().expect("lock sessions");
        sessions
            .get_mut(&session.session_id)
            .expect("session")
            .pending_output
            .push_str("persist exactly once\r\n");
    }
    service
        .flush_session_history(&session.session_id)
        .await
        .expect("first flush");
    service
        .flush_session_history(&session.session_id)
        .await
        .expect("second flush");

    let history = service
        .session_history(SshCloseInput {
            workspace_id,
            session_id: session.session_id,
        })
        .await
        .expect("hydrate history");
    assert_eq!(history[0].data.matches("persist exactly once").count(), 1);
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn repeated_close_does_not_panic_and_returns_stable_result() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");
    let session = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            cols: None,
            rows: None,
            secret: None,
        })
        .await
        .expect("connect ssh session");

    let first_close = service
        .close_session(SshCloseInput {
            workspace_id: workspace_id.clone(),
            session_id: session.session_id.clone(),
        })
        .await
        .expect("first close");
    assert_eq!(first_close.status, "disconnected");

    let second_close = service
        .close_session(SshCloseInput {
            workspace_id: workspace_id.clone(),
            session_id: session.session_id.clone(),
        })
        .await
        .expect("second close should not fail");
    assert_eq!(second_close.status, "disconnected");
    assert_eq!(second_close.session_id, first_close.session_id);
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn async_send_input_and_resize_work_in_simulated_path() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");
    let session = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            cols: Some(80),
            rows: Some(24),
            secret: None,
        })
        .await
        .expect("connect ssh session");

    // send_input is now async.
    let event = service
        .send_input(SshSessionInput {
            workspace_id: workspace_id.clone(),
            session_id: session.session_id.clone(),
            data: "ls -la\n".to_string(),
        })
        .await
        .expect("async send input");
    assert_eq!(event.kind, "output");

    // resize is now async.
    let resize_event = service
        .resize(SshResizeInput {
            workspace_id: workspace_id.clone(),
            session_id: session.session_id.clone(),
            cols: 200,
            rows: 50,
        })
        .await
        .expect("async resize");
    assert_eq!(resize_event.kind, "resize");
    assert!(resize_event.data.contains("200x50"));

    // Verify session dimensions were updated.
    let sessions = service
        .list_sessions(workspace_id)
        .await
        .expect("list sessions");
    assert_eq!(sessions[0].cols, 200);
    assert_eq!(sessions[0].rows, 50);
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn multiple_sessions_handle_concurrent_input_and_close() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");

    let session_a = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            cols: None,
            rows: None,
            secret: None,
        })
        .await
        .expect("connect session a");

    let session_b = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            cols: None,
            rows: None,
            secret: None,
        })
        .await
        .expect("connect session b");

    // Send input to both sessions concurrently.
    let (result_a, result_b) = tokio::join!(
        service.send_input(SshSessionInput {
            workspace_id: workspace_id.clone(),
            session_id: session_a.session_id.clone(),
            data: "echo A\n".to_string(),
        }),
        service.send_input(SshSessionInput {
            workspace_id: workspace_id.clone(),
            session_id: session_b.session_id.clone(),
            data: "echo B\n".to_string(),
        }),
    );
    assert!(result_a.is_ok());
    assert!(result_b.is_ok());

    // Close session a, session b remains active.
    service
        .close_session(SshCloseInput {
            workspace_id: workspace_id.clone(),
            session_id: session_a.session_id.clone(),
        })
        .await
        .expect("close session a");

    // Session b should still accept input.
    let event_b = service
        .send_input(SshSessionInput {
            workspace_id: workspace_id.clone(),
            session_id: session_b.session_id.clone(),
            data: "whoami\n".to_string(),
        })
        .await
        .expect("session b still active");
    assert_eq!(event_b.kind, "output");

    let sessions = service
        .list_sessions(workspace_id)
        .await
        .expect("list sessions");
    let closed = sessions
        .iter()
        .find(|s| s.session_id == session_a.session_id)
        .unwrap();
    let active = sessions
        .iter()
        .find(|s| s.session_id == session_b.session_id)
        .unwrap();
    assert_eq!(closed.status, "disconnected");
    assert_eq!(active.status, "connected");
}

#[test]
fn reconnect_policy_is_bounded_to_three_attempts() {
    assert_eq!(RECONNECT_BACKOFF_SECS, [1, 2, 4]);
    assert_eq!(RECONNECT_BACKOFF_SECS.len(), 3);
    assert_eq!(RECONNECT_BACKOFF_SECS.iter().sum::<u64>(), 7);
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn explicit_close_disables_reconnect() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");
    let session = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id,
            cols: None,
            rows: None,
            secret: None,
        })
        .await
        .expect("connect session");

    service
        .close_session(SshCloseInput {
            workspace_id,
            session_id: session.session_id.clone(),
        })
        .await
        .expect("close session");

    let sessions = service.sessions.lock().expect("session lock");
    let state = sessions.get(&session.session_id).expect("session state");
    assert!(state.intentional_close);
    assert!(!should_reconnect(state));
    assert_eq!(state.summary.status, "disconnected");
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn cancel_reconnect_marks_session_disconnected() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");
    let session = service
        .connect(SshConnectInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id,
            cols: None,
            rows: None,
            secret: None,
        })
        .await
        .expect("connect session");

    let cancelled = service
        .cancel_reconnect(SshReconnectCancelInput {
            workspace_id,
            session_id: session.session_id.clone(),
        })
        .await
        .expect("cancel reconnect");

    assert_eq!(cancelled.status, "disconnected");
    assert_eq!(cancelled.reconnect_attempt, 0);
    let sessions = service.sessions.lock().expect("session lock");
    assert!(!should_reconnect(
        sessions.get(&session.session_id).expect("session state")
    ));
}

#[cfg(not(feature = "ssh-native"))]
#[tokio::test]
async fn dropped_and_failed_states_stop_after_cleanup() {
    let (service, workspace_id, _) = service_with_workspaces().await;
    let connection = service
        .save_connection(password_input(&workspace_id))
        .await
        .expect("save ssh connection");
    let session = service
        .connect(SshConnectInput {
            workspace_id,
            connection_id: connection.id,
            cols: None,
            rows: None,
            secret: None,
        })
        .await
        .expect("connect session");

    let mut sessions = service.sessions.lock().expect("session lock");
    let state = sessions
        .get_mut(&session.session_id)
        .expect("session state");
    state.summary.status = "degraded".to_string();
    assert!(should_reconnect(state));
    state.summary.status = "reconnecting".to_string();
    state.summary.reconnect_attempt = 3;
    assert!(should_reconnect(state));
    state.summary.status = "failed".to_string();
    assert!(!should_reconnect(state));
}
