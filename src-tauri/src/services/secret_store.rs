#[derive(Clone)]
pub struct SecretStore {
    service_name: String,
}

impl SecretStore {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    #[allow(dead_code)]
    pub fn make_ref(&self, workspace_id: &str, kind: &str, record_id: &str) -> String {
        format!(
            "{}:{}:{}:{}",
            self.service_name, workspace_id, kind, record_id
        )
    }

    pub fn capability_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "provider": "os-keychain-reserved",
            "plainTextStorage": false,
            "refFormat": format!("{}:<workspace>:<kind>:<record>", self.service_name)
        })
    }
}
