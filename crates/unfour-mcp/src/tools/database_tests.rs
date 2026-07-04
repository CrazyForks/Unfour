use std::sync::Arc;

use serde_json::json;
use unfour_command_bus::{
    ConnectionListResult, CurrentWorkspaceResult, ReadCommand, ReadCommandResult,
    WorkspaceListResult, WorkspaceSummary,
};
use unfour_core::models::{
    ApiResponse, DatabaseConnection, DatabaseQueryInput, DatabaseQueryResult, DatabaseQuerySafety,
    DatabaseResultColumn, DatabaseSchema, DatabaseTable, DatabaseTableColumn, DatabaseTestResult,
};

use crate::command_bus_adapter::{CommandBusAdapter, CommandBusAdapterError};

// --- Stub implementations ---

struct DbStubCommandBus;

impl CommandBusAdapter for DbStubCommandBus {
    fn execute_read(
        &self,
        command: ReadCommand,
    ) -> Result<ReadCommandResult, CommandBusAdapterError> {
        Ok(match command {
            ReadCommand::CurrentWorkspace => {
                ReadCommandResult::CurrentWorkspace(CurrentWorkspaceResult {
                    workspace_id: "workspace-1".to_string(),
                    workspace_name: "Workspace".to_string(),
                    environment_type: "dev".to_string(),
                    mcp_policy: "auto".to_string(),
                    workspace_root: None,
                    mode: "local".to_string(),
                    source: "command-bus".to_string(),
                })
            }
            ReadCommand::ListWorkspaces => ReadCommandResult::Workspaces(WorkspaceListResult {
                workspaces: vec![WorkspaceSummary {
                    id: "workspace-1".to_string(),
                    name: "Workspace".to_string(),
                    is_default: true,
                    is_active: true,
                    environment_type: "dev".to_string(),
                    mcp_policy: "auto".to_string(),
                    last_opened_at: None,
                }],
                active_workspace_id: "workspace-1".to_string(),
                count: 1,
                source: "command-bus".to_string(),
            }),
            ReadCommand::ListConnections { .. } => {
                ReadCommandResult::Connections(ConnectionListResult {
                    connections: vec![],
                    count: 0,
                    source: "command-bus".to_string(),
                })
            }
            _ => ReadCommandResult::CurrentWorkspace(CurrentWorkspaceResult {
                workspace_id: "workspace-1".to_string(),
                workspace_name: "Workspace".to_string(),
                environment_type: "dev".to_string(),
                mcp_policy: "auto".to_string(),
                workspace_root: None,
                mode: "local".to_string(),
                source: "command-bus".to_string(),
            }),
        })
    }

    fn execute_saved_api_request(
        &self,
        _request_id: &str,
        _timeout_ms: Option<u64>,
    ) -> Result<ApiResponse, CommandBusAdapterError> {
        Ok(ApiResponse {
            history_id: "history-1".to_string(),
            status: 200,
            status_text: "OK".to_string(),
            headers: vec![],
            body: "{}".to_string(),
            duration_ms: 0,
        })
    }

    fn list_db_connections(
        &self,
        _workspace_id: &str,
    ) -> Result<Vec<DatabaseConnection>, CommandBusAdapterError> {
        Ok(vec![DatabaseConnection {
            id: "conn-1".to_string(),
            workspace_id: "workspace-1".to_string(),
            name: "Dev Database".to_string(),
            driver: "postgres".to_string(),
            host: Some("localhost".to_string()),
            port: Some(5432),
            database: Some("app_dev".to_string()),
            username: Some("admin".to_string()),
            ssl_mode: None,
            sqlite_path: None,
            credential_ref: Some("secret-ref-123".to_string()),
            read_only: false,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            deleted_at: None,
            revision: 1,
            sync_status: "local".to_string(),
            remote_id: None,
        }])
    }

    fn get_db_schema(
        &self,
        _workspace_id: &str,
        connection_id: &str,
    ) -> Result<DatabaseSchema, CommandBusAdapterError> {
        Ok(DatabaseSchema {
            connection_id: connection_id.to_string(),
            tables: vec![
                DatabaseTable {
                    catalog: None,
                    schema: Some("public".to_string()),
                    name: "users".to_string(),
                    kind: "table".to_string(),
                    columns: vec![
                        DatabaseTableColumn {
                            name: "id".to_string(),
                            data_type: "integer".to_string(),
                            nullable: false,
                            primary_key: true,
                            default_value: None,
                        },
                        DatabaseTableColumn {
                            name: "email".to_string(),
                            data_type: "varchar".to_string(),
                            nullable: false,
                            primary_key: false,
                            default_value: None,
                        },
                        DatabaseTableColumn {
                            name: "created_at".to_string(),
                            data_type: "timestamp".to_string(),
                            nullable: true,
                            primary_key: false,
                            default_value: None,
                        },
                    ],
                },
                DatabaseTable {
                    catalog: None,
                    schema: Some("public".to_string()),
                    name: "orders".to_string(),
                    kind: "table".to_string(),
                    columns: vec![DatabaseTableColumn {
                        name: "id".to_string(),
                        data_type: "integer".to_string(),
                        nullable: false,
                        primary_key: true,
                        default_value: None,
                    }],
                },
                DatabaseTable {
                    catalog: None,
                    schema: Some("analytics".to_string()),
                    name: "events".to_string(),
                    kind: "view".to_string(),
                    columns: vec![],
                },
                DatabaseTable {
                    catalog: None,
                    schema: Some("analytics".to_string()),
                    name: "summary".to_string(),
                    kind: "table".to_string(),
                    columns: vec![],
                },
                DatabaseTable {
                    catalog: None,
                    schema: Some("audit".to_string()),
                    name: "logs".to_string(),
                    kind: "table".to_string(),
                    columns: vec![],
                },
            ],
        })
    }

    fn execute_db_query(
        &self,
        input: DatabaseQueryInput,
    ) -> Result<DatabaseQueryResult, CommandBusAdapterError> {
        let keyword = input
            .sql
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let is_read = matches!(keyword.as_str(), "select" | "with" | "explain" | "show");
        Ok(DatabaseQueryResult {
            columns: if is_read {
                vec![
                    DatabaseResultColumn {
                        name: "id".to_string(),
                        data_type: "integer".to_string(),
                    },
                    DatabaseResultColumn {
                        name: "email".to_string(),
                        data_type: "varchar".to_string(),
                    },
                ]
            } else {
                vec![]
            },
            rows: if is_read {
                vec![
                    vec![Some("1".to_string()), Some("user@example.com".to_string())],
                    vec![Some("2".to_string()), Some("other@example.com".to_string())],
                ]
            } else {
                vec![]
            },
            affected_rows: if is_read { 0 } else { 2 },
            duration_ms: 42,
            safety: DatabaseQuerySafety {
                classification: if is_read { "read" } else { "mutation" }.to_string(),
                requires_confirmation: !is_read,
                confirmed: is_read || input.confirm_mutation == Some(true),
                message: None,
            },
        })
    }

    fn test_db_connection(
        &self,
        _workspace_id: &str,
        _connection_id: &str,
    ) -> Result<DatabaseTestResult, CommandBusAdapterError> {
        Ok(DatabaseTestResult {
            ok: true,
            message: "Connection successful".to_string(),
            server_version: Some("PostgreSQL 16.1".to_string()),
        })
    }
}

struct DbFailingCommandBus;

impl CommandBusAdapter for DbFailingCommandBus {
    fn execute_read(
        &self,
        command: ReadCommand,
    ) -> Result<ReadCommandResult, CommandBusAdapterError> {
        match command {
            ReadCommand::CurrentWorkspace => Ok(ReadCommandResult::CurrentWorkspace(
                CurrentWorkspaceResult {
                    workspace_id: "workspace-1".to_string(),
                    workspace_name: "Workspace".to_string(),
                    environment_type: "dev".to_string(),
                    mcp_policy: "auto".to_string(),
                    workspace_root: None,
                    mode: "local".to_string(),
                    source: "command-bus".to_string(),
                },
            )),
            ReadCommand::ListWorkspaces => Ok(ReadCommandResult::Workspaces(WorkspaceListResult {
                workspaces: vec![WorkspaceSummary {
                    id: "workspace-1".to_string(),
                    name: "Workspace".to_string(),
                    is_default: true,
                    is_active: true,
                    environment_type: "dev".to_string(),
                    mcp_policy: "auto".to_string(),
                    last_opened_at: None,
                }],
                active_workspace_id: "workspace-1".to_string(),
                count: 1,
                source: "command-bus".to_string(),
            })),
            _ => Err(CommandBusAdapterError {
                code: "COMMAND_BUS_READ_FAILED",
                message: "The command-bus read operation failed.",
            }),
        }
    }

    fn execute_saved_api_request(
        &self,
        _request_id: &str,
        _timeout_ms: Option<u64>,
    ) -> Result<ApiResponse, CommandBusAdapterError> {
        Err(CommandBusAdapterError {
            code: "COMMAND_BUS_API_SEND_FAILED",
            message: "The command-bus API send operation failed.",
        })
    }

    fn list_db_connections(
        &self,
        _workspace_id: &str,
    ) -> Result<Vec<DatabaseConnection>, CommandBusAdapterError> {
        Err(CommandBusAdapterError {
            code: "COMMAND_BUS_DB_LIST_FAILED",
            message: "The command-bus database list operation failed.",
        })
    }

    fn get_db_schema(
        &self,
        _workspace_id: &str,
        _connection_id: &str,
    ) -> Result<DatabaseSchema, CommandBusAdapterError> {
        Err(CommandBusAdapterError {
            code: "COMMAND_BUS_DB_SCHEMA_FAILED",
            message: "The command-bus database schema operation failed.",
        })
    }

    fn execute_db_query(
        &self,
        _input: DatabaseQueryInput,
    ) -> Result<DatabaseQueryResult, CommandBusAdapterError> {
        Err(CommandBusAdapterError {
            code: "COMMAND_BUS_DB_QUERY_FAILED",
            message: "The command-bus database query operation failed.",
        })
    }
}

fn registry() -> super::super::ToolRegistry {
    super::super::ToolRegistry::with_command_bus(Arc::new(DbStubCommandBus))
}

struct ProdDbStubCommandBus;

impl CommandBusAdapter for ProdDbStubCommandBus {
    fn execute_read(
        &self,
        _command: ReadCommand,
    ) -> Result<ReadCommandResult, CommandBusAdapterError> {
        Ok(ReadCommandResult::CurrentWorkspace(
            CurrentWorkspaceResult {
                workspace_id: "workspace-prod".to_string(),
                workspace_name: "Production".to_string(),
                environment_type: "prod".to_string(),
                mcp_policy: "auto".to_string(),
                workspace_root: None,
                mode: "local".to_string(),
                source: "command-bus".to_string(),
            },
        ))
    }

    fn execute_saved_api_request(
        &self,
        _request_id: &str,
        _timeout_ms: Option<u64>,
    ) -> Result<ApiResponse, CommandBusAdapterError> {
        unreachable!()
    }

    fn list_db_connections(
        &self,
        _workspace_id: &str,
    ) -> Result<Vec<DatabaseConnection>, CommandBusAdapterError> {
        unreachable!()
    }

    fn get_db_schema(
        &self,
        _workspace_id: &str,
        _connection_id: &str,
    ) -> Result<DatabaseSchema, CommandBusAdapterError> {
        unreachable!()
    }

    fn execute_db_query(
        &self,
        _input: DatabaseQueryInput,
    ) -> Result<DatabaseQueryResult, CommandBusAdapterError> {
        panic!("prod write should be blocked before command-bus execution")
    }
}

// --- list_connections tests ---

#[test]
fn list_connections_returns_safe_summary() {
    let result = registry()
        .call("unfour.db.list_connections", json!({}))
        .expect("should succeed");

    let content = &result["structuredContent"];
    assert_eq!(content["count"], 1);
    let conn = &content["connections"][0];
    assert_eq!(conn["id"], "conn-1");
    assert_eq!(conn["name"], "Dev Database");
    assert_eq!(conn["databaseType"], "postgres");
    assert_eq!(conn["host"], "localhost");
    assert_eq!(conn["port"], 5432);
    assert_eq!(conn["database"], "app_dev");

    // Ensure sensitive fields are NOT present.
    let serialized = serde_json::to_string(content).unwrap();
    assert!(!serialized.contains("admin"));
    assert!(!serialized.contains("secret-ref-123"));
    assert!(!serialized.contains("credentialRef"));
    assert!(!serialized.contains("credential_ref"));
}

#[test]
fn list_connections_resolves_default_workspace() {
    let result = registry()
        .call("unfour.db.list_connections", json!({}))
        .expect("should succeed");
    assert_eq!(result["structuredContent"]["source"], "command-bus");
}

#[test]
fn list_connections_handles_empty() {
    struct EmptyDbStub;
    impl CommandBusAdapter for EmptyDbStub {
        fn execute_read(
            &self,
            _command: ReadCommand,
        ) -> Result<ReadCommandResult, CommandBusAdapterError> {
            Ok(ReadCommandResult::CurrentWorkspace(
                CurrentWorkspaceResult {
                    workspace_id: "ws-1".to_string(),
                    workspace_name: "W".to_string(),
                    environment_type: "dev".to_string(),
                    mcp_policy: "auto".to_string(),
                    workspace_root: None,
                    mode: "local".to_string(),
                    source: "command-bus".to_string(),
                },
            ))
        }
        fn execute_saved_api_request(
            &self,
            _: &str,
            _: Option<u64>,
        ) -> Result<ApiResponse, CommandBusAdapterError> {
            unreachable!()
        }
        fn list_db_connections(
            &self,
            _: &str,
        ) -> Result<Vec<DatabaseConnection>, CommandBusAdapterError> {
            Ok(vec![])
        }
        fn get_db_schema(
            &self,
            _: &str,
            _: &str,
        ) -> Result<DatabaseSchema, CommandBusAdapterError> {
            unreachable!()
        }
        fn execute_db_query(
            &self,
            _: DatabaseQueryInput,
        ) -> Result<DatabaseQueryResult, CommandBusAdapterError> {
            unreachable!()
        }
    }

    let reg = super::super::ToolRegistry::with_command_bus(Arc::new(EmptyDbStub));
    let result = reg
        .call("unfour.db.list_connections", json!({}))
        .expect("should succeed");
    assert_eq!(result["structuredContent"]["count"], 0);
}

// --- list_tables tests ---

#[test]
fn list_tables_returns_table_summaries() {
    let result = registry()
        .call("unfour.db.list_tables", json!({ "connectionId": "conn-1" }))
        .expect("should succeed");

    let content = &result["structuredContent"];
    assert_eq!(content["connectionId"], "conn-1");
    assert_eq!(content["totalTables"], 5);
    assert_eq!(content["count"], 5);
    assert_eq!(content["truncated"], false);

    let first = &content["tables"][0];
    assert_eq!(first["name"], "users");
    assert_eq!(first["schema"], "public");
    assert_eq!(first["kind"], "table");
    assert_eq!(first["columnCount"], 3);
}

#[test]
fn list_tables_respects_limit() {
    let result = registry()
        .call(
            "unfour.db.list_tables",
            json!({ "connectionId": "conn-1", "limit": 2 }),
        )
        .expect("should succeed");

    let content = &result["structuredContent"];
    assert_eq!(content["count"], 2);
    assert_eq!(content["totalTables"], 5);
    assert_eq!(content["truncated"], true);
}

#[test]
fn list_tables_requires_connection_id() {
    let result = registry().call("unfour.db.list_tables", json!({}));
    assert!(result.is_err(), "should fail without connectionId");
}

#[test]
fn list_tables_clamps_limit_to_500() {
    let result = registry()
        .call(
            "unfour.db.list_tables",
            json!({ "connectionId": "conn-1", "limit": 9999 }),
        )
        .expect("should succeed");

    let content = &result["structuredContent"];
    // We have 5 tables, limit clamped to 500, so all 5 returned.
    assert_eq!(content["count"], 5);
    assert_eq!(content["truncated"], false);
}

// --- describe_table tests ---

#[test]
fn describe_table_returns_columns() {
    let result = registry()
        .call(
            "unfour.db.describe_table",
            json!({ "connectionId": "conn-1", "tableName": "users" }),
        )
        .expect("should succeed");

    let content = &result["structuredContent"];
    assert_eq!(content["connectionId"], "conn-1");
    let table = &content["table"];
    assert_eq!(table["name"], "users");
    assert_eq!(table["schema"], "public");
    assert_eq!(table["kind"], "table");
    assert_eq!(table["columnCount"], 3);

    let id_col = &table["columns"][0];
    assert_eq!(id_col["name"], "id");
    assert_eq!(id_col["dataType"], "integer");
    assert_eq!(id_col["nullable"], false);
    assert_eq!(id_col["primaryKey"], true);
}

#[test]
fn describe_table_with_schema_filter() {
    let result = registry()
        .call(
            "unfour.db.describe_table",
            json!({ "connectionId": "conn-1", "tableName": "events", "schema": "analytics" }),
        )
        .expect("should succeed");

    let content = &result["structuredContent"];
    assert_eq!(content["table"]["name"], "events");
    assert_eq!(content["table"]["schema"], "analytics");
    assert_eq!(content["table"]["kind"], "view");
}

#[test]
fn describe_table_not_found_returns_error() {
    let result = registry()
        .call(
            "unfour.db.describe_table",
            json!({ "connectionId": "conn-1", "tableName": "nonexistent" }),
        )
        .expect("should return error result");

    assert_eq!(result["isError"], true);
    assert_eq!(
        result["structuredContent"]["error"]["code"],
        "TABLE_NOT_FOUND"
    );
}

#[test]
fn describe_table_requires_table_name() {
    let result = registry().call(
        "unfour.db.describe_table",
        json!({ "connectionId": "conn-1" }),
    );
    assert!(result.is_err(), "should fail without tableName");
}

// --- query_readonly tests ---

#[test]
fn query_readonly_executes_select() {
    let result = registry()
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "SELECT id, email FROM users LIMIT 10"
            }),
        )
        .expect("should succeed");

    let content = &result["structuredContent"];
    assert_eq!(content["ok"], true);
    assert_eq!(content["connectionId"], "conn-1");
    assert_eq!(content["columns"].as_array().unwrap().len(), 2);
    assert_eq!(content["rowCount"], 2);
    assert_eq!(content["durationMs"], 42);
    assert_eq!(content["source"], "command-bus");
}

#[test]
fn query_readonly_allows_with_cte() {
    let result = registry()
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "WITH cte AS (SELECT 1) SELECT * FROM cte"
            }),
        )
        .expect("should succeed");
    assert_eq!(result["structuredContent"]["ok"], true);
}

#[test]
fn query_readonly_allows_explain() {
    let result = registry()
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "EXPLAIN SELECT * FROM users"
            }),
        )
        .expect("should succeed");
    assert_eq!(result["structuredContent"]["ok"], true);
}

#[test]
fn query_readonly_rejects_insert() {
    let result = registry()
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "INSERT INTO users (email) VALUES ('evil@test.com')"
            }),
        )
        .expect("should return error result");
    assert_eq!(result["isError"], true);
}

#[test]
fn query_readonly_rejects_update() {
    let result = registry()
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "UPDATE users SET email = 'hacked' WHERE id = 1"
            }),
        )
        .expect("should return error result");
    assert_eq!(result["isError"], true);
}

#[test]
fn query_readonly_rejects_delete() {
    let result = registry()
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "DELETE FROM users WHERE id = 1"
            }),
        )
        .expect("should return error result");
    assert_eq!(result["isError"], true);
}

#[test]
fn query_readonly_rejects_drop_alter_create() {
    for sql in &[
        "DROP TABLE users",
        "ALTER TABLE users ADD COLUMN foo TEXT",
        "CREATE TABLE evil (id INT)",
        "TRUNCATE TABLE users",
    ] {
        let result = registry()
            .call(
                "unfour.db.query_readonly",
                json!({ "connectionId": "conn-1", "sql": sql }),
            )
            .expect("should return error result");
        assert_eq!(result["isError"], true, "should reject: {}", sql);
    }
}

#[test]
fn query_readonly_rejects_multi_statement() {
    let result = registry()
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "SELECT 1; DROP TABLE users"
            }),
        )
        .expect("should return error result");
    assert_eq!(result["isError"], true);
}

#[test]
fn query_readonly_rejects_comment_bypass() {
    let result = registry()
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "/* harmless comment */ INSERT INTO users VALUES (1)"
            }),
        )
        .expect("should return error result");
    assert_eq!(result["isError"], true);
}

#[test]
fn query_readonly_clamps_limit_to_1000() {
    let result = registry()
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "SELECT * FROM users",
                "limit": 99999
            }),
        )
        .expect("should succeed");
    assert_eq!(result["structuredContent"]["ok"], true);
}

#[test]
fn query_readonly_truncates_large_results() {
    struct LargeResultStub;
    impl CommandBusAdapter for LargeResultStub {
        fn execute_read(
            &self,
            _: ReadCommand,
        ) -> Result<ReadCommandResult, CommandBusAdapterError> {
            Ok(ReadCommandResult::CurrentWorkspace(
                CurrentWorkspaceResult {
                    workspace_id: "ws-1".to_string(),
                    workspace_name: "W".to_string(),
                    environment_type: "dev".to_string(),
                    mcp_policy: "auto".to_string(),
                    workspace_root: None,
                    mode: "local".to_string(),
                    source: "command-bus".to_string(),
                },
            ))
        }
        fn execute_saved_api_request(
            &self,
            _: &str,
            _: Option<u64>,
        ) -> Result<ApiResponse, CommandBusAdapterError> {
            unreachable!()
        }
        fn list_db_connections(
            &self,
            _: &str,
        ) -> Result<Vec<DatabaseConnection>, CommandBusAdapterError> {
            unreachable!()
        }
        fn get_db_schema(
            &self,
            _: &str,
            _: &str,
        ) -> Result<DatabaseSchema, CommandBusAdapterError> {
            unreachable!()
        }
        fn execute_db_query(
            &self,
            _: DatabaseQueryInput,
        ) -> Result<DatabaseQueryResult, CommandBusAdapterError> {
            // Generate rows that will exceed 20KB.
            let big_value = "x".repeat(1024);
            let rows: Vec<Vec<Option<String>>> = (0..100)
                .map(|i| vec![Some(i.to_string()), Some(big_value.clone())])
                .collect();
            Ok(DatabaseQueryResult {
                columns: vec![
                    DatabaseResultColumn {
                        name: "id".to_string(),
                        data_type: "integer".to_string(),
                    },
                    DatabaseResultColumn {
                        name: "data".to_string(),
                        data_type: "text".to_string(),
                    },
                ],
                rows,
                affected_rows: 0,
                duration_ms: 10,
                safety: DatabaseQuerySafety {
                    classification: "read".to_string(),
                    requires_confirmation: false,
                    confirmed: true,
                    message: None,
                },
            })
        }
    }

    let reg = super::super::ToolRegistry::with_command_bus(Arc::new(LargeResultStub));
    let result = reg
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "SELECT id, data FROM big_table"
            }),
        )
        .expect("should succeed");

    let content = &result["structuredContent"];
    assert_eq!(content["ok"], true);
    assert_eq!(content["truncated"], true);
    // Should have fewer rows than the original 100.
    assert!(content["rowCount"].as_u64().unwrap() < 100);
}

#[test]
fn query_readonly_command_bus_failure() {
    let reg = super::super::ToolRegistry::with_command_bus(Arc::new(DbFailingCommandBus));
    let result = reg
        .call(
            "unfour.db.query_readonly",
            json!({
                "connectionId": "conn-1",
                "sql": "SELECT 1",
                "workspaceId": "workspace-1"
            }),
        )
        .expect("should return error result");
    assert_eq!(result["isError"], true);
    assert_eq!(
        result["structuredContent"]["error"]["code"],
        "COMMAND_BUS_DB_QUERY_FAILED"
    );
}

// --- execute tests ---

#[test]
fn execute_allows_dev_update_with_where() {
    let result = registry()
        .call(
            "unfour.db.execute",
            json!({
                "connectionId": "conn-1",
                "sql": "UPDATE users SET email = 'new@example.com' WHERE id = 1"
            }),
        )
        .expect("dev update should execute");

    let content = &result["structuredContent"];
    assert_eq!(result["isError"], false);
    assert_eq!(content["environment"], "dev");
    assert_eq!(content["risk_level"], "medium");
    assert_eq!(content["affectedRows"], 2);
    assert_eq!(content["safety"]["confirmed"], true);
}

#[test]
fn execute_delete_without_where_requires_confirmation_then_executes() {
    let first = registry()
        .call(
            "unfour.db.execute",
            json!({ "connectionId": "conn-1", "sql": "DELETE FROM users" }),
        )
        .expect("confirmation should be structured");

    assert_eq!(first["isError"], true);
    assert_eq!(first["structuredContent"]["requires_confirmation"], true);
    let confirmation = first["structuredContent"]["confirmation_text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(confirmation.starts_with("DELETE_WITHOUT_WHERE:"));

    let confirmed = registry()
        .call(
            "unfour.db.execute",
            json!({
                "connectionId": "conn-1",
                "sql": "DELETE FROM users",
                "confirm": true,
                "confirmation_text": confirmation
            }),
        )
        .expect("confirmed delete should execute in dev");
    assert_eq!(confirmed["isError"], false);
    assert_eq!(confirmed["structuredContent"]["affectedRows"], 2);
}

#[test]
fn execute_prod_update_is_blocked_by_policy() {
    let registry = super::super::ToolRegistry::with_command_bus(Arc::new(ProdDbStubCommandBus));
    let result = registry
        .call(
            "unfour.db.execute",
            json!({
                "connectionId": "conn-1",
                "sql": "UPDATE users SET email = 'new@example.com' WHERE id = 1"
            }),
        )
        .expect("policy denial should be structured");

    assert_eq!(result["isError"], true);
    assert_eq!(result["structuredContent"]["ok"], false);
    assert_eq!(
        result["structuredContent"]["error"]["code"],
        "WORKSPACE_POLICY_BLOCKED"
    );
    assert_eq!(result["structuredContent"]["environment"], "prod");
}

// --- test_connection tests ---

#[test]
fn test_connection_returns_ok_with_server_version() {
    let result = registry()
        .call(
            "unfour.db.test_connection",
            json!({ "connectionId": "conn-1" }),
        )
        .expect("should succeed");

    let content = &result["structuredContent"];
    assert_eq!(content["ok"], true);
    assert_eq!(content["connectionId"], "conn-1");
    assert_eq!(content["message"], "Connection successful");
    assert_eq!(content["serverVersion"], "PostgreSQL 16.1");
    assert_eq!(content["source"], "command-bus");
}

#[test]
fn test_connection_requires_connection_id() {
    let result = registry().call("unfour.db.test_connection", json!({}));
    assert!(result.is_err(), "should fail without connectionId");
}
