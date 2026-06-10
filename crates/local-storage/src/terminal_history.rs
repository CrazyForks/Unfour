use chrono::Utc;
use unfour_core::models::{SshSessionEvent, SshSessionSummary};
use unfour_core::{AppError, AppResult};

use crate::LocalDb;

pub const TERMINAL_HISTORY_MAX_BYTES: usize = 256 * 1024;
const TERMINAL_HISTORY_SESSION_LIMIT: i64 = 20;

#[derive(Clone)]
pub struct TerminalHistoryService {
    db: LocalDb,
}

impl TerminalHistoryService {
    pub fn new(db: LocalDb) -> Self {
        Self { db }
    }

    pub async fn save_session(&self, summary: &SshSessionSummary) -> AppResult<()> {
        validate_identity(
            &summary.workspace_id,
            &summary.session_id,
            &summary.connection_id,
        )?;
        sqlx::query(
            r#"
            INSERT INTO ssh_terminal_history (
              workspace_id, session_id, connection_id, status, reconnect_attempt,
              auth_kind, host, username, cols, rows, content, byte_len,
              created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, '', 0, ?11, ?12)
            ON CONFLICT(workspace_id, session_id) DO UPDATE SET
              connection_id = excluded.connection_id,
              status = excluded.status,
              reconnect_attempt = excluded.reconnect_attempt,
              auth_kind = excluded.auth_kind,
              host = excluded.host,
              username = excluded.username,
              cols = excluded.cols,
              rows = excluded.rows,
              updated_at = excluded.updated_at
            "#,
        )
        .bind(&summary.workspace_id)
        .bind(&summary.session_id)
        .bind(&summary.connection_id)
        .bind(&summary.status)
        .bind(summary.reconnect_attempt as i64)
        .bind(&summary.auth_kind)
        .bind(&summary.host)
        .bind(&summary.username)
        .bind(summary.cols as i64)
        .bind(summary.rows as i64)
        .bind(&summary.created_at)
        .bind(&summary.updated_at)
        .execute(self.db.pool())
        .await?;
        Ok(())
    }

    pub async fn append_output(
        &self,
        workspace_id: &str,
        session_id: &str,
        connection_id: &str,
        output: &str,
    ) -> AppResult<()> {
        validate_identity(workspace_id, session_id, connection_id)?;
        if output.is_empty() {
            return Ok(());
        }

        let redacted = redact_terminal_output(output);
        if redacted.is_empty() {
            return Ok(());
        }

        let current: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT content
            FROM ssh_terminal_history
            WHERE workspace_id = ?1 AND session_id = ?2 AND connection_id = ?3
            "#,
        )
        .bind(workspace_id)
        .bind(session_id)
        .bind(connection_id)
        .fetch_optional(self.db.pool())
        .await?;
        let Some((mut content,)) = current else {
            return Err(AppError::NotFound(
                "ssh terminal history session".to_string(),
            ));
        };

        content.push_str(&redacted);
        let content = retain_utf8_tail(&content, TERMINAL_HISTORY_MAX_BYTES);
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE ssh_terminal_history
            SET content = ?1, byte_len = ?2, updated_at = ?3
            WHERE workspace_id = ?4 AND session_id = ?5 AND connection_id = ?6
            "#,
        )
        .bind(&content)
        .bind(content.len() as i64)
        .bind(now)
        .bind(workspace_id)
        .bind(session_id)
        .bind(connection_id)
        .execute(self.db.pool())
        .await?;
        Ok(())
    }

    pub async fn update_session(&self, summary: &SshSessionSummary) -> AppResult<()> {
        validate_identity(
            &summary.workspace_id,
            &summary.session_id,
            &summary.connection_id,
        )?;
        sqlx::query(
            r#"
            UPDATE ssh_terminal_history
            SET status = ?1, reconnect_attempt = ?2, cols = ?3, rows = ?4, updated_at = ?5
            WHERE workspace_id = ?6 AND session_id = ?7 AND connection_id = ?8
            "#,
        )
        .bind(&summary.status)
        .bind(summary.reconnect_attempt as i64)
        .bind(summary.cols as i64)
        .bind(summary.rows as i64)
        .bind(&summary.updated_at)
        .bind(&summary.workspace_id)
        .bind(&summary.session_id)
        .bind(&summary.connection_id)
        .execute(self.db.pool())
        .await?;
        Ok(())
    }

    pub async fn list_sessions(&self, workspace_id: &str) -> AppResult<Vec<SshSessionSummary>> {
        validate_workspace_id(workspace_id)?;
        let rows = sqlx::query_as::<_, PersistedSession>(
            r#"
            SELECT
              session_id, workspace_id, connection_id, status, reconnect_attempt,
              auth_kind, host, username, cols, rows, created_at, updated_at
            FROM ssh_terminal_history
            WHERE workspace_id = ?1
            ORDER BY updated_at DESC
            LIMIT ?2
            "#,
        )
        .bind(workspace_id)
        .bind(TERMINAL_HISTORY_SESSION_LIMIT)
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| SshSessionSummary {
                session_id: row.session_id,
                workspace_id: row.workspace_id,
                connection_id: row.connection_id,
                status: if matches!(
                    row.status.as_str(),
                    "connected" | "degraded" | "reconnecting"
                ) {
                    "disconnected".to_string()
                } else {
                    row.status
                },
                reconnect_attempt: row.reconnect_attempt.clamp(0, u8::MAX as i64) as u8,
                auth_kind: row.auth_kind,
                host: row.host,
                username: row.username,
                cols: row.cols.clamp(0, u16::MAX as i64) as u16,
                rows: row.rows.clamp(0, u16::MAX as i64) as u16,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    pub async fn hydrate(
        &self,
        workspace_id: &str,
        session_id: &str,
    ) -> AppResult<Vec<SshSessionEvent>> {
        validate_workspace_id(workspace_id)?;
        validate_session_id(session_id)?;
        let row: Option<(String, String)> = sqlx::query_as(
            r#"
            SELECT content, updated_at
            FROM ssh_terminal_history
            WHERE workspace_id = ?1 AND session_id = ?2
            "#,
        )
        .bind(workspace_id)
        .bind(session_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(match row {
            Some((content, created_at)) if !content.is_empty() => vec![SshSessionEvent {
                session_id: session_id.to_string(),
                kind: "output".to_string(),
                data: content,
                created_at,
            }],
            _ => Vec::new(),
        })
    }

    pub async fn delete_connection_history(
        &self,
        workspace_id: &str,
        connection_id: &str,
    ) -> AppResult<()> {
        validate_workspace_id(workspace_id)?;
        validate_connection_id(connection_id)?;
        sqlx::query(
            "DELETE FROM ssh_terminal_history WHERE workspace_id = ?1 AND connection_id = ?2",
        )
        .bind(workspace_id)
        .bind(connection_id)
        .execute(self.db.pool())
        .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct PersistedSession {
    session_id: String,
    workspace_id: String,
    connection_id: String,
    status: String,
    reconnect_attempt: i64,
    auth_kind: String,
    host: String,
    username: String,
    cols: i64,
    rows: i64,
    created_at: String,
    updated_at: String,
}

fn retain_utf8_tail(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut start = value.len() - max_bytes;
    while !value.is_char_boundary(start) {
        start += 1;
    }
    value[start..].to_string()
}

fn redact_terminal_output(value: &str) -> String {
    value
        .split_inclusive('\n')
        .map(|segment| {
            let (line, ending) = if let Some(line) = segment.strip_suffix("\r\n") {
                (line, "\r\n")
            } else if let Some(line) = segment.strip_suffix('\n') {
                (line, "\n")
            } else {
                (segment, "")
            };
            let lower = line.to_ascii_lowercase();
            if lower.contains("authorization:")
                || lower.contains("cookie:")
                || lower.contains("proxy-authorization:")
                || lower.contains("x-api-key:")
                || lower.contains("x-auth-token:")
                || lower.contains("password=")
                || lower.contains("passphrase=")
                || lower.contains("credential_ref")
                || lower.contains("credentialref")
                || lower.contains("private key")
                || lower.contains("private-key")
            {
                format!("<redacted>{ending}")
            } else {
                segment.to_string()
            }
        })
        .collect()
}

fn validate_identity(workspace_id: &str, session_id: &str, connection_id: &str) -> AppResult<()> {
    validate_workspace_id(workspace_id)?;
    validate_session_id(session_id)?;
    validate_connection_id(connection_id)
}

fn validate_workspace_id(value: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(
            "workspace id cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_session_id(value: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(
            "ssh session id cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_connection_id(value: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(
            "ssh connection id cannot be empty".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    async fn service() -> TerminalHistoryService {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect in-memory sqlite");
        let db = LocalDb::from_pool(pool);
        db.migrate().await.expect("run migrations");
        let now = Utc::now().to_rfc3339();
        for workspace_id in ["ws-a", "ws-b"] {
            sqlx::query(
                r#"
                INSERT INTO workspaces (
                  id, name, is_default, created_at, updated_at, revision, sync_status
                )
                VALUES (?1, ?1, 0, ?2, ?2, 1, 'local')
                "#,
            )
            .bind(workspace_id)
            .bind(&now)
            .execute(db.pool())
            .await
            .expect("insert workspace");
        }
        TerminalHistoryService::new(db)
    }

    fn summary(workspace_id: &str, session_id: &str, connection_id: &str) -> SshSessionSummary {
        let now = Utc::now().to_rfc3339();
        SshSessionSummary {
            session_id: session_id.to_string(),
            workspace_id: workspace_id.to_string(),
            connection_id: connection_id.to_string(),
            status: "connected".to_string(),
            reconnect_attempt: 0,
            auth_kind: "password".to_string(),
            host: "localhost".to_string(),
            username: "developer".to_string(),
            cols: 120,
            rows: 32,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn append_and_hydrate_terminal_output() {
        let service = service().await;
        let summary = summary("ws-a", "session-a", "connection-a");
        service.save_session(&summary).await.expect("save session");
        service
            .append_output(
                &summary.workspace_id,
                &summary.session_id,
                &summary.connection_id,
                "hello\r\nworld\r\n",
            )
            .await
            .expect("append output");

        let events = service
            .hydrate(&summary.workspace_id, &summary.session_id)
            .await
            .expect("hydrate output");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello\r\nworld\r\n");
    }

    #[tokio::test]
    async fn retention_keeps_only_the_recent_utf8_tail() {
        let service = service().await;
        let summary = summary("ws-a", "session-a", "connection-a");
        service.save_session(&summary).await.expect("save session");
        let prefix = "x".repeat(TERMINAL_HISTORY_MAX_BYTES);
        service
            .append_output(
                &summary.workspace_id,
                &summary.session_id,
                &summary.connection_id,
                &prefix,
            )
            .await
            .expect("append prefix");
        service
            .append_output(
                &summary.workspace_id,
                &summary.session_id,
                &summary.connection_id,
                "recent-终端",
            )
            .await
            .expect("append recent output");

        let events = service
            .hydrate(&summary.workspace_id, &summary.session_id)
            .await
            .expect("hydrate output");
        assert!(events[0].data.len() <= TERMINAL_HISTORY_MAX_BYTES);
        assert!(events[0].data.ends_with("recent-终端"));
    }

    #[tokio::test]
    async fn sessions_and_workspaces_are_isolated() {
        let service = service().await;
        for summary in [
            summary("ws-a", "session-a", "connection-a"),
            summary("ws-a", "session-b", "connection-a"),
            summary("ws-b", "session-a", "connection-b"),
        ] {
            service.save_session(&summary).await.expect("save session");
            service
                .append_output(
                    &summary.workspace_id,
                    &summary.session_id,
                    &summary.connection_id,
                    &format!("{}:{}\r\n", summary.workspace_id, summary.session_id),
                )
                .await
                .expect("append output");
        }

        let session_a = service
            .hydrate("ws-a", "session-a")
            .await
            .expect("hydrate session a");
        let session_b = service
            .hydrate("ws-a", "session-b")
            .await
            .expect("hydrate session b");
        let other_workspace = service
            .hydrate("ws-b", "session-a")
            .await
            .expect("hydrate other workspace");
        assert_eq!(session_a[0].data, "ws-a:session-a\r\n");
        assert_eq!(session_b[0].data, "ws-a:session-b\r\n");
        assert_eq!(other_workspace[0].data, "ws-b:session-a\r\n");
    }

    #[tokio::test]
    async fn persistence_redacts_secrets_and_credential_references() {
        let service = service().await;
        let summary = summary("ws-a", "session-a", "connection-a");
        service.save_session(&summary).await.expect("save session");
        service
            .append_output(
                &summary.workspace_id,
                &summary.session_id,
                &summary.connection_id,
                "ok\r\npassword=secret\r\ncredential_ref=ssh-password-1\r\nprivate key: data\r\n",
            )
            .await
            .expect("append output");

        let events = service
            .hydrate(&summary.workspace_id, &summary.session_id)
            .await
            .expect("hydrate output");
        assert!(events[0].data.contains("ok"));
        assert!(!events[0].data.contains("secret"));
        assert!(!events[0].data.contains("ssh-password-1"));
        assert!(!events[0].data.contains("private key: data"));
    }

    #[tokio::test]
    async fn deleting_connection_cleans_up_only_matching_history() {
        let service = service().await;
        let removed = summary("ws-a", "session-a", "connection-a");
        let retained = summary("ws-a", "session-b", "connection-b");
        service.save_session(&removed).await.expect("save removed");
        service
            .save_session(&retained)
            .await
            .expect("save retained");

        service
            .delete_connection_history("ws-a", "connection-a")
            .await
            .expect("delete history");
        assert!(service
            .hydrate("ws-a", "session-a")
            .await
            .expect("hydrate removed")
            .is_empty());
        assert_eq!(
            service
                .list_sessions("ws-a")
                .await
                .expect("list sessions")
                .len(),
            1
        );
    }
}
