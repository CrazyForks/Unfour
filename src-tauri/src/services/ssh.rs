#[derive(Clone)]
pub struct SshService;

impl SshService {
    pub fn new() -> Self {
        Self
    }

    pub fn capability_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "reserved",
            "plannedBackend": "russh",
            "features": [
                "password-auth",
                "private-key-auth",
                "multi-session",
                "terminal-streaming",
                "redacted-session-log"
            ]
        })
    }
}
