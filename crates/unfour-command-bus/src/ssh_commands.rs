use super::*;

impl CommandBus {
    pub async fn list_ssh_connections(
        &self,
        workspace_id: String,
    ) -> AppResult<Vec<SshConnection>> {
        self.ssh.list_connections(workspace_id).await
    }

    /// Run a read-only SSH diagnostic command against a saved connection. The
    /// command is allowlist-validated and output line-redacted by the SSH
    /// engine; this records an activity event (command + exit status only, never
    /// the captured output).
    pub async fn run_ssh_diagnostic(
        &self,
        input: SshDiagnosticInput,
    ) -> AppResult<SshDiagnosticResult> {
        let workspace_id = input.workspace_id.clone();
        let result = self.ssh.run_diagnostic(input).await?;
        self.activity_log
            .record(
                Some(&workspace_id),
                "ssh.diagnostic",
                Some(&result.connection_id),
                serde_json::json!({
                    "command": result.command,
                    "exitStatus": result.exit_status,
                    "truncated": result.truncated,
                }),
            )
            .await?;
        Ok(result)
    }

    /// Run a one-shot SSH command through the SSH engine. Callers must perform
    /// environment policy and high-risk confirmation before invoking this
    /// method; this boundary records only a redacted/truncated command preview.
    pub async fn run_ssh_command(
        &self,
        input: SshDiagnosticInput,
    ) -> AppResult<SshDiagnosticResult> {
        let workspace_id = input.workspace_id.clone();
        let connection_id = input.connection_id.clone();
        let (command_kind, command_redacted) = command_activity_kind(&input.command);
        let result = self.ssh.run_command(input).await?;
        self.activity_log
            .record(
                Some(&workspace_id),
                "ssh.command",
                Some(&connection_id),
                serde_json::json!({
                    "commandKind": command_kind,
                    "commandRedacted": command_redacted,
                    "exitStatus": result.exit_status,
                    "truncated": result.truncated,
                }),
            )
            .await?;
        Ok(result)
    }

    pub async fn save_ssh_connection(&self, input: SshConnectionInput) -> AppResult<SshConnection> {
        let connection = self.ssh.save_connection(input).await?;
        self.activity_log
            .record(
                Some(&connection.workspace_id),
                "ssh.connection.save",
                Some(&connection.id),
                serde_json::json!({
                    "name": connection.name,
                    "host": connection.host,
                    "authKind": connection.auth_kind,
                    "credentialRef": connection.credential_ref.is_some()
                }),
            )
            .await?;
        Ok(connection)
    }

    pub async fn test_ssh_connection(&self, input: SshConnectionInput) -> AppResult<SshTestResult> {
        let result = self.ssh.test_connection(input.clone()).await?;
        self.activity_log
            .record(
                Some(&input.workspace_id),
                "ssh.connection.test",
                None,
                serde_json::json!({
                    "host": input.host,
                    "authKind": input.auth_kind,
                    "ok": result.ok
                }),
            )
            .await?;
        Ok(result)
    }

    pub async fn delete_ssh_connection(
        &self,
        workspace_id: String,
        connection_id: String,
    ) -> AppResult<Vec<SshConnection>> {
        let connections = self
            .ssh
            .delete_connection(workspace_id.clone(), connection_id.clone())
            .await?;
        self.activity_log
            .record(
                Some(&workspace_id),
                "ssh.connection.delete",
                Some(&connection_id),
                serde_json::json!({ "softDelete": true }),
            )
            .await?;
        Ok(connections)
    }

    pub async fn connect_ssh_session(
        &self,
        input: SshConnectInput,
    ) -> AppResult<SshSessionSummary> {
        let session = self.ssh.connect(input.clone()).await?;
        self.activity_log
            .record(
                Some(&input.workspace_id),
                "ssh.session.connect",
                Some(&session.session_id),
                serde_json::json!({
                    "connectionId": input.connection_id,
                    "authKind": session.auth_kind,
                    "host": session.host,
                    "pty": {
                        "cols": session.cols,
                        "rows": session.rows
                    }
                }),
            )
            .await?;
        Ok(session)
    }

    pub async fn list_ssh_sessions(
        &self,
        workspace_id: String,
    ) -> AppResult<Vec<SshSessionSummary>> {
        self.ssh.list_sessions(workspace_id).await
    }

    pub async fn ssh_session_history(
        &self,
        input: SshCloseInput,
    ) -> AppResult<Vec<SshSessionEvent>> {
        self.ssh.session_history(input).await
    }

    pub async fn send_ssh_input(&self, input: SshSessionInput) -> AppResult<SshSessionEvent> {
        self.ssh.send_input(input).await
    }

    pub async fn resize_ssh_session(&self, input: SshResizeInput) -> AppResult<SshSessionEvent> {
        self.ssh.resize(input).await
    }

    pub async fn close_ssh_session(&self, input: SshCloseInput) -> AppResult<SshSessionSummary> {
        let session = self.ssh.close_session(input.clone()).await?;
        self.activity_log
            .record(
                Some(&input.workspace_id),
                "ssh.session.close",
                Some(&input.session_id),
                serde_json::json!({ "status": session.status }),
            )
            .await?;
        Ok(session)
    }

    pub async fn cancel_ssh_reconnect(
        &self,
        input: SshReconnectCancelInput,
    ) -> AppResult<SshSessionSummary> {
        self.ssh.cancel_reconnect(input).await
    }

    pub async fn export_ssh_log(&self, input: SshLogExportInput) -> AppResult<SshLogExport> {
        let export = self.ssh.export_log(input.clone()).await?;
        self.activity_log
            .record(
                Some(&input.workspace_id),
                "ssh.session.log_export",
                Some(&input.session_id),
                serde_json::json!({
                    "lineCount": export.line_count,
                    "redacted": export.redacted
                }),
            )
            .await?;
        Ok(export)
    }

    pub async fn get_ssh_host_fingerprint(
        &self,
        input: SshHostKeyInput,
    ) -> AppResult<Option<SshHostFingerprintInfo>> {
        self.ssh.get_host_fingerprint(input).await
    }

    pub async fn reset_ssh_host_fingerprint(&self, input: SshHostKeyInput) -> AppResult<bool> {
        let workspace_id = input.workspace_id.clone();
        let host = input.host.clone();
        let port = input.port;
        let deleted = self.ssh.reset_host_fingerprint(input).await?;
        if deleted {
            self.activity_log
                .record(
                    Some(&workspace_id),
                    "ssh.host_key.reset",
                    Some(&format!("{}:{}", host, port)),
                    serde_json::json!({ "host": host, "port": port }),
                )
                .await?;
        }
        Ok(deleted)
    }

    pub async fn list_all_ssh_fingerprints(
        &self,
        workspace_id: String,
    ) -> AppResult<Vec<SshHostFingerprintInfo>> {
        self.ssh.list_all_host_fingerprints(workspace_id).await
    }

    pub async fn import_ssh_known_hosts(
        &self,
        input: SshKnownHostsImportInput,
    ) -> AppResult<SshKnownHostsImportResult> {
        let workspace_id = input.workspace_id.clone();
        let result = self.ssh.import_known_hosts(input).await?;
        self.activity_log
            .record(
                Some(&workspace_id),
                "ssh.known_hosts.import",
                None,
                serde_json::json!({
                    "imported": result.imported,
                    "skipped": result.skipped,
                    "errors": result.errors.len(),
                }),
            )
            .await?;
        Ok(result)
    }

    pub async fn export_ssh_known_hosts(
        &self,
        input: SshKnownHostsExportInput,
    ) -> AppResult<SshKnownHostsExportResult> {
        self.ssh.export_known_hosts(input).await
    }

    pub fn reserved_status(&self) -> serde_json::Value {
        serde_json::json!({
            "ssh": self.ssh.capability_summary(),
            "database": self.database.capability_summary(),
            "secrets": self.secret_store.capability_summary()
        })
    }
}
