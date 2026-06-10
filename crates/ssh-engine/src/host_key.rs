use chrono::Utc;
use sqlx::SqlitePool;
use unfour_core::models::SshKnownHostsImportResult;
use unfour_core::{AppError, AppResult};

/// Host-key verification using trust-on-first-use (TOFU).
///
/// On first connection to a host, the server's key fingerprint is recorded.
/// On subsequent connections, the stored fingerprint must match.
/// A mismatch is always rejected with a clear error.
#[derive(Clone)]
pub struct HostKeyStore {
    pool: SqlitePool,
}

impl HostKeyStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Record the fingerprint for a host on first connection.
    pub async fn record_fingerprint(
        &self,
        host: &str,
        port: u16,
        fingerprint: &str,
    ) -> AppResult<()> {
        self.record_fingerprint_full(host, port, fingerprint, None, None)
            .await
    }

    /// Record the fingerprint with optional key type and public key data.
    pub async fn record_fingerprint_full(
        &self,
        host: &str,
        port: u16,
        fingerprint: &str,
        key_type: Option<&str>,
        public_key_data: Option<&str>,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO ssh_host_keys (host, port, fingerprint, key_type, public_key_data, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(host)
        .bind(port as i64)
        .bind(fingerprint)
        .bind(key_type)
        .bind(public_key_data)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Look up the stored fingerprint for a host, if any.
    pub async fn get_fingerprint(&self, host: &str, port: u16) -> AppResult<Option<String>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT fingerprint FROM ssh_host_keys WHERE host = ?1 AND port = ?2",
        )
        .bind(host)
        .bind(port as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// Verify or record a host key fingerprint.
    ///
    /// - If no fingerprint is stored, record the given one (first connection).
    /// - If a fingerprint is stored and matches, return Ok.
    /// - If a fingerprint is stored and does NOT match, return an error.
    pub async fn verify_or_record(
        &self,
        host: &str,
        port: u16,
        fingerprint: &str,
    ) -> AppResult<()> {
        match self.get_fingerprint(host, port).await? {
            None => {
                self.record_fingerprint(host, port, fingerprint).await?;
                Ok(())
            }
            Some(stored) if stored == fingerprint => Ok(()),
            Some(_) => Err(AppError::Config(format!(
                "SSH host key verification failed for {}:{}: \
                 server key fingerprint does not match the previously recorded \
                 fingerprint. This could indicate a man-in-the-middle attack.",
                host, port
            ))),
        }
    }

    /// Remove the stored fingerprint for a host:port pair.
    ///
    /// Returns `true` if a record was deleted, `false` if no record existed.
    pub async fn delete_fingerprint(&self, host: &str, port: u16) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM ssh_host_keys WHERE host = ?1 AND port = ?2")
            .bind(host)
            .bind(port as i64)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Return the stored fingerprint and the timestamp when it was recorded.
    pub async fn get_fingerprint_info(
        &self,
        host: &str,
        port: u16,
    ) -> AppResult<Option<(String, String)>> {
        let row = sqlx::query_as::<_, (String, String)>(
            "SELECT fingerprint, created_at FROM ssh_host_keys WHERE host = ?1 AND port = ?2",
        )
        .bind(host)
        .bind(port as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    /// List all stored host-key fingerprints.
    pub async fn list_all(&self) -> AppResult<Vec<StoredHostKey>> {
        let rows = sqlx::query_as::<_, StoredHostKey>(
            r#"
            SELECT host, port, fingerprint, key_type, public_key_data, created_at
            FROM ssh_host_keys
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Import entries from OpenSSH known_hosts content.
    ///
    /// Parses each line, computes the SHA-256 fingerprint from the public key,
    /// and stores entries that are valid and not already present.
    pub async fn import_known_hosts(&self, content: &str) -> AppResult<SshKnownHostsImportResult> {
        let mut imported = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();

        for (line_number, raw_line) in content.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            match parse_known_hosts_line(line) {
                Some(entry) => {
                    let existing = self.get_fingerprint(&entry.host, entry.port).await?;
                    if existing.is_some() {
                        skipped += 1;
                        continue;
                    }
                    match self
                        .record_fingerprint_full(
                            &entry.host,
                            entry.port,
                            &entry.fingerprint,
                            Some(&entry.key_type),
                            Some(&entry.public_key_data),
                        )
                        .await
                    {
                        Ok(()) => imported += 1,
                        Err(err) => {
                            errors.push(format!(
                                "line {}: failed to store {}: {}",
                                line_number + 1,
                                entry.host,
                                err
                            ));
                        }
                    }
                }
                None => {
                    skipped += 1;
                }
            }
        }

        Ok(SshKnownHostsImportResult {
            imported,
            skipped,
            errors,
        })
    }

    /// Export stored fingerprints to OpenSSH known_hosts format.
    ///
    /// Entries with stored public key data produce full known_hosts lines.
    /// Entries without public key data are exported as comments.
    pub async fn export_known_hosts(&self) -> AppResult<(String, u32)> {
        let entries = self.list_all().await?;
        let mut lines = Vec::new();
        let mut count = 0u32;

        for entry in &entries {
            let host_port = if entry.port == 22 {
                entry.host.clone()
            } else {
                format!("[{}]:{}", entry.host, entry.port)
            };

            if let (Some(key_type), Some(key_data)) = (&entry.key_type, &entry.public_key_data) {
                lines.push(format!("{} {} {}", host_port, key_type, key_data));
                count += 1;
            } else {
                lines.push(format!(
                    "# {} {} (fingerprint only, no key data)",
                    host_port, entry.fingerprint
                ));
            }
        }

        let content = if lines.is_empty() {
            String::new()
        } else {
            let mut s = lines.join("\n");
            s.push('\n');
            s
        };

        Ok((content, count))
    }
}

/// A stored host-key record with all columns.
#[derive(Clone, sqlx::FromRow)]
pub struct StoredHostKey {
    pub host: String,
    pub port: i64,
    pub fingerprint: String,
    pub key_type: Option<String>,
    pub public_key_data: Option<String>,
    pub created_at: String,
}

struct ParsedKnownHostsEntry {
    host: String,
    port: u16,
    key_type: String,
    public_key_data: String,
    fingerprint: String,
}

fn parse_known_hosts_line(line: &str) -> Option<ParsedKnownHostsEntry> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let host_field = parts[0];
    let key_type = parts[1];
    let key_data = parts[2];

    // Validate key type looks like an SSH key type.
    if !key_type.starts_with("ssh-")
        && !key_type.starts_with("ecdsa-")
        && key_type != "sk-ssh-ed25519@openssh.com"
        && key_type != "sk-ecdsa-sha2-nistp256@openssh.com"
    {
        return None;
    }

    // Parse host:port from bracket notation or plain host.
    let (host, port) = if host_field.starts_with('[') {
        if let Some(bracket_end) = host_field.find(']') {
            let h = &host_field[1..bracket_end];
            let rest = &host_field[bracket_end + 1..];
            let p = if let Some(port_str) = rest.strip_prefix(':') {
                port_str.parse::<u16>().ok()?
            } else {
                22
            };
            (h.to_string(), p)
        } else {
            return None;
        }
    } else if host_field.contains(',') {
        // Skip entries with multiple hosts (hash groups, wildcards).
        return None;
    } else {
        (host_field.to_string(), 22)
    };

    if host.is_empty() {
        return None;
    }

    // Compute SHA-256 fingerprint from the base64 public key data.
    use sha2::{Digest, Sha256};
    let key_bytes = base64_decode(key_data).ok()?;
    let digest = Sha256::digest(&key_bytes);
    let fingerprint = format!("SHA256:{}", base64_encode_nopad(&digest));

    Some(ParsedKnownHostsEntry {
        host,
        port,
        key_type: key_type.to_string(),
        public_key_data: key_data.to_string(),
        fingerprint,
    })
}

fn base64_decode(input: &str) -> Result<Vec<u8>, ()> {
    // Standard base64 with or without padding.
    let input = input.trim_end_matches('=');
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buf = Vec::with_capacity(input.len() * 3 / 4);
    let mut accum: u32 = 0;
    let mut bits: u32 = 0;
    for &byte in input.as_bytes() {
        let val = match alphabet.iter().position(|&b| b == byte) {
            Some(v) => v as u32,
            None => return Err(()),
        };
        accum = (accum << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buf.push((accum >> bits) as u8);
            accum &= (1 << bits) - 1;
        }
    }
    Ok(buf)
}

fn base64_encode_nopad(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() * 4 + 2) / 3);
    let chunks = input.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(triple & 0x3F) as usize] as char);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use unfour_local_storage::LocalDb;

    async fn test_store() -> HostKeyStore {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect in-memory");
        let db = LocalDb::from_pool(pool.clone());
        db.migrate().await.expect("run migrations");
        HostKeyStore::new(pool)
    }

    #[tokio::test]
    async fn host_key_first_connect_records_fingerprint() {
        let store = test_store().await;

        store
            .verify_or_record("example.com", 22, "SHA256:abc123")
            .await
            .expect("first connect should record fingerprint");

        let stored = store
            .get_fingerprint("example.com", 22)
            .await
            .expect("lookup fingerprint");
        assert_eq!(stored.as_deref(), Some("SHA256:abc123"));
    }

    #[tokio::test]
    async fn host_key_matching_fingerprint_succeeds() {
        let store = test_store().await;

        store
            .verify_or_record("example.com", 22, "SHA256:abc123")
            .await
            .expect("first connect");

        store
            .verify_or_record("example.com", 22, "SHA256:abc123")
            .await
            .expect("matching fingerprint should succeed");
    }

    #[tokio::test]
    async fn host_key_mismatch_is_rejected() {
        let store = test_store().await;

        store
            .verify_or_record("example.com", 22, "SHA256:abc123")
            .await
            .expect("first connect");

        let result = store
            .verify_or_record("example.com", 22, "SHA256:different456")
            .await;
        assert!(result.is_err(), "mismatched fingerprint must be rejected");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("host key verification failed"),
            "error should mention host key verification: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn host_key_different_hosts_are_independent() {
        let store = test_store().await;

        store
            .verify_or_record("host-a.example.com", 22, "SHA256:aaa")
            .await
            .expect("host a first connect");

        store
            .verify_or_record("host-b.example.com", 22, "SHA256:bbb")
            .await
            .expect("host b first connect with different fingerprint");

        store
            .verify_or_record("host-a.example.com", 22, "SHA256:aaa")
            .await
            .expect("host a still matches");
    }

    #[tokio::test]
    async fn host_key_different_ports_are_independent() {
        let store = test_store().await;

        store
            .verify_or_record("example.com", 22, "SHA256:port22")
            .await
            .expect("port 22 first connect");

        store
            .verify_or_record("example.com", 2222, "SHA256:port2222")
            .await
            .expect("port 2222 first connect with different fingerprint");
    }

    #[tokio::test]
    async fn host_key_delete_fingerprint_removes_record() {
        let store = test_store().await;

        store
            .verify_or_record("example.com", 22, "SHA256:abc123")
            .await
            .expect("first connect");

        let deleted = store
            .delete_fingerprint("example.com", 22)
            .await
            .expect("delete fingerprint");
        assert!(deleted, "should have deleted an existing record");

        let stored = store
            .get_fingerprint("example.com", 22)
            .await
            .expect("lookup after delete");
        assert!(stored.is_none(), "fingerprint should be gone");

        // Deleting again should return false (nothing to delete).
        let deleted_again = store
            .delete_fingerprint("example.com", 22)
            .await
            .expect("delete again");
        assert!(!deleted_again, "no record to delete");
    }

    #[tokio::test]
    async fn host_key_get_fingerprint_info_returns_metadata() {
        let store = test_store().await;

        // No record yet.
        let info = store
            .get_fingerprint_info("example.com", 22)
            .await
            .expect("lookup before any record");
        assert!(info.is_none());

        store
            .verify_or_record("example.com", 22, "SHA256:abc123")
            .await
            .expect("first connect");

        let info = store
            .get_fingerprint_info("example.com", 22)
            .await
            .expect("lookup after record");
        let (fingerprint, created_at) = info.expect("should have fingerprint info");
        assert_eq!(fingerprint, "SHA256:abc123");
        assert!(!created_at.is_empty(), "created_at should be populated");
    }

    #[tokio::test]
    async fn host_key_delete_allows_new_trust() {
        let store = test_store().await;

        store
            .verify_or_record("example.com", 22, "SHA256:old_key")
            .await
            .expect("first connect");

        // Mismatch would be rejected.
        let result = store
            .verify_or_record("example.com", 22, "SHA256:new_key")
            .await;
        assert!(result.is_err(), "mismatch must be rejected");

        // After reset, a new fingerprint is accepted (TOFU).
        store
            .delete_fingerprint("example.com", 22)
            .await
            .expect("reset fingerprint");

        store
            .verify_or_record("example.com", 22, "SHA256:new_key")
            .await
            .expect("new trust after reset");

        let stored = store
            .get_fingerprint("example.com", 22)
            .await
            .expect("lookup");
        assert_eq!(stored.as_deref(), Some("SHA256:new_key"));
    }

    #[tokio::test]
    async fn list_all_returns_all_stored_fingerprints() {
        let store = test_store().await;

        store
            .verify_or_record("host-a", 22, "SHA256:aaa")
            .await
            .expect("record host-a");
        store
            .verify_or_record("host-b", 2222, "SHA256:bbb")
            .await
            .expect("record host-b");

        let all = store.list_all().await.expect("list all");
        assert_eq!(all.len(), 2);
        let hosts: Vec<&str> = all.iter().map(|e| e.host.as_str()).collect();
        assert!(hosts.contains(&"host-a"));
        assert!(hosts.contains(&"host-b"));
    }

    #[tokio::test]
    async fn import_known_hosts_parses_valid_entries() {
        let store = test_store().await;
        // Use a real SSH RSA public key (truncated for test; valid base64).
        let key_data = "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7";
        let content = format!("example.com ssh-rsa {}", key_data);

        let result = store
            .import_known_hosts(&content)
            .await
            .expect("import known_hosts");

        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 0);
        assert!(result.errors.is_empty());

        // Verify it was stored.
        let fp = store
            .get_fingerprint("example.com", 22)
            .await
            .expect("get fingerprint");
        assert!(fp.is_some());
        assert!(fp.unwrap().starts_with("SHA256:"));
    }

    #[tokio::test]
    async fn import_known_hosts_skips_duplicates() {
        let store = test_store().await;
        let key_data = "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7";
        let content = format!("example.com ssh-rsa {}", key_data);

        let result1 = store
            .import_known_hosts(&content)
            .await
            .expect("first import");
        assert_eq!(result1.imported, 1);

        let result2 = store
            .import_known_hosts(&content)
            .await
            .expect("second import");
        assert_eq!(result2.imported, 0);
        assert_eq!(result2.skipped, 1);
    }

    #[tokio::test]
    async fn import_known_hosts_skips_comments_and_blank_lines() {
        let store = test_store().await;
        let content = "# This is a comment\n\n   \n# Another comment\n";
        let result = store.import_known_hosts(content).await.expect("import");
        assert_eq!(result.imported, 0);
        assert_eq!(result.skipped, 0);
    }

    #[tokio::test]
    async fn import_known_hosts_handles_bracketed_host_with_port() {
        let store = test_store().await;
        let key_data = "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7";
        let content = format!("[myhost.com]:2222 ssh-ed25519 {}", key_data);

        let result = store.import_known_hosts(&content).await.expect("import");
        assert_eq!(result.imported, 1);

        let fp = store
            .get_fingerprint("myhost.com", 2222)
            .await
            .expect("get fingerprint");
        assert!(fp.is_some());
    }

    #[tokio::test]
    async fn export_known_hosts_produces_valid_format() {
        let store = test_store().await;
        let key_data = "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7";
        let content = format!("example.com ssh-rsa {}", key_data);

        store.import_known_hosts(&content).await.expect("import");

        let (exported, count) = store.export_known_hosts().await.expect("export");
        assert_eq!(count, 1);
        assert!(exported.contains("example.com"));
        assert!(exported.contains("ssh-rsa"));
        assert!(exported.contains(key_data));
    }

    #[tokio::test]
    async fn export_entries_without_key_data_are_comments() {
        let store = test_store().await;
        // Record without key data (old-style TOFU entry).
        store
            .record_fingerprint("oldhost.com", 22, "SHA256:old")
            .await
            .expect("record");

        let (exported, count) = store.export_known_hosts().await.expect("export");
        assert_eq!(count, 0);
        assert!(exported.starts_with('#'));
        assert!(exported.contains("SHA256:old"));
    }

    #[test]
    fn parse_known_hosts_line_valid() {
        let line = "example.com ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7";
        let entry = parse_known_hosts_line(line);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.host, "example.com");
        assert_eq!(entry.port, 22);
        assert_eq!(entry.key_type, "ssh-rsa");
        assert!(entry.fingerprint.starts_with("SHA256:"));
    }

    #[test]
    fn parse_known_hosts_line_bracketed_port() {
        let line = "[myhost.com]:2222 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA";
        let entry = parse_known_hosts_line(line);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.host, "myhost.com");
        assert_eq!(entry.port, 2222);
    }

    #[test]
    fn parse_known_hosts_line_invalid() {
        assert!(parse_known_hosts_line("# comment").is_none());
        assert!(parse_known_hosts_line("").is_none());
        assert!(parse_known_hosts_line("only one field").is_none());
        assert!(parse_known_hosts_line("host not-a-key-type AAAA").is_none());
    }

    #[test]
    fn base64_roundtrip() {
        let input = b"Hello, World!";
        let encoded = base64_encode_nopad(input);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }
}
