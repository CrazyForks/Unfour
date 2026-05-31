#[derive(Clone)]
pub struct DatabaseService;

impl DatabaseService {
    pub fn new() -> Self {
        Self
    }

    pub fn capability_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "reserved",
            "plannedBackend": "sqlx",
            "mvpDrivers": ["postgres", "mysql", "sqlite"],
            "features": [
                "connection-test",
                "schema-browser",
                "sql-editor",
                "paged-query-results",
                "read-only-table-view"
            ]
        })
    }
}
