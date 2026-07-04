use std::sync::Arc;

use serde_json::json;
use unfour_command_bus::{CurrentWorkspaceResult, ReadCommand, ReadCommandResult};
use unfour_core::models::{
    ApiResponse, DatabaseConnection, DatabaseQueryInput, DatabaseQueryResult, DatabaseQuerySafety,
    DatabaseSchema, SshDiagnosticInput, SshDiagnosticResult,
};

use crate::command_bus_adapter::{CommandBusAdapter, CommandBusAdapterError};
use crate::tools::ToolRegistry;

/// Stub that runs diagnostics, echoing the validated command back.
struct SshStubCommandBus;

impl CommandBusAdapter for SshStubCommandBus {
    fn execute_read(
        &self,
        command: ReadCommand,
    ) -> Result<ReadCommandResult, CommandBusAdapterError> {
        match command {
            ReadCommand::CurrentWorkspace => Ok(ReadCommandResult::CurrentWorkspace(
                CurrentWorkspaceResult {
                    workspace_id: "ws-active".to_string(),
                    workspace_name: "Active".to_string(),
                    environment_type: "dev".to_string(),
                    mcp_policy: "auto".to_string(),
                    workspace_root: None,
                    mode: "local".to_string(),
                    source: "command-bus".to_string(),
                },
            )),
            _ => Err(CommandBusAdapterError {
                code: "UNEXPECTED",
                message: "unexpected command",
            }),
        }
    }

    fn run_ssh_diagnostic(
        &self,
        input: SshDiagnosticInput,
    ) -> Result<SshDiagnosticResult, CommandBusAdapterError> {
        Ok(SshDiagnosticResult {
            connection_id: input.connection_id,
            command: input.command,
            stdout: "Filesystem Size Used Avail\n/dev/sda1 50G 20G 30G".to_string(),
            stderr: String::new(),
            exit_status: Some(0),
            truncated: false,
        })
    }

    fn run_ssh_command(
        &self,
        input: SshDiagnosticInput,
    ) -> Result<SshDiagnosticResult, CommandBusAdapterError> {
        if input.command.contains("__UNFOUR_MATCH_COUNT__") {
            let allow_multiple = input.command.contains("allow_multiple = True");
            return Ok(SshDiagnosticResult {
                connection_id: input.connection_id,
                command: input.command,
                stdout: if allow_multiple {
                    "__UNFOUR_MATCH_COUNT__ 2\n__UNFOUR_PATCHED__ 2\n".to_string()
                } else {
                    "__UNFOUR_MATCH_COUNT__ 2\n".to_string()
                },
                stderr: String::new(),
                exit_status: if allow_multiple { Some(0) } else { Some(3) },
                truncated: false,
            });
        }

        Ok(SshDiagnosticResult {
            connection_id: input.connection_id,
            command: input.command.clone(),
            stdout: format!("ran: {}", input.command),
            stderr: String::new(),
            exit_status: Some(0),
            truncated: false,
        })
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
        Ok(vec![])
    }

    fn get_db_schema(
        &self,
        _workspace_id: &str,
        _connection_id: &str,
    ) -> Result<DatabaseSchema, CommandBusAdapterError> {
        Ok(DatabaseSchema {
            connection_id: String::new(),
            tables: vec![],
        })
    }

    fn execute_db_query(
        &self,
        _input: DatabaseQueryInput,
    ) -> Result<DatabaseQueryResult, CommandBusAdapterError> {
        Ok(DatabaseQueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows: 0,
            duration_ms: 0,
            safety: DatabaseQuerySafety {
                classification: "read".to_string(),
                requires_confirmation: false,
                confirmed: true,
                message: None,
            },
        })
    }
}

/// Stub that does NOT override `run_ssh_diagnostic`, so the trait default
/// (unsupported) is exercised ? mirrors a build without `ssh-native`.
struct UnsupportedSshCommandBus;

impl CommandBusAdapter for UnsupportedSshCommandBus {
    fn execute_read(
        &self,
        _command: ReadCommand,
    ) -> Result<ReadCommandResult, CommandBusAdapterError> {
        Ok(ReadCommandResult::CurrentWorkspace(
            CurrentWorkspaceResult {
                workspace_id: "ws-active".to_string(),
                workspace_name: "Active".to_string(),
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
        _request_id: &str,
        _timeout_ms: Option<u64>,
    ) -> Result<ApiResponse, CommandBusAdapterError> {
        unreachable!()
    }

    fn list_db_connections(
        &self,
        _workspace_id: &str,
    ) -> Result<Vec<DatabaseConnection>, CommandBusAdapterError> {
        Ok(vec![])
    }

    fn get_db_schema(
        &self,
        _workspace_id: &str,
        _connection_id: &str,
    ) -> Result<DatabaseSchema, CommandBusAdapterError> {
        Ok(DatabaseSchema {
            connection_id: String::new(),
            tables: vec![],
        })
    }

    fn execute_db_query(
        &self,
        _input: DatabaseQueryInput,
    ) -> Result<DatabaseQueryResult, CommandBusAdapterError> {
        unreachable!()
    }
}

fn registry() -> ToolRegistry {
    ToolRegistry::with_command_bus(Arc::new(SshStubCommandBus))
}

#[test]
fn ssh_tool_is_registered() {
    assert!(registry()
        .definitions()
        .iter()
        .any(|d| d.name == "unfour.ssh.run_diagnostic"));
}

#[test]
fn run_diagnostic_returns_captured_output() {
    let result = registry()
        .call(
            "unfour.ssh.run_diagnostic",
            json!({ "connectionId": "conn-1", "command": "df -h" }),
        )
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    let content = &result["structuredContent"];
    assert_eq!(content["connectionId"], "conn-1");
    assert_eq!(content["command"], "df -h");
    assert_eq!(content["exitStatus"], 0);
    assert!(content["stdout"].as_str().unwrap().contains("Filesystem"));
    assert_eq!(content["source"], "command-bus");
}

#[test]
fn exec_allows_dev_regular_command() {
    let result = registry()
        .call(
            "unfour.ssh.exec",
            json!({ "connectionId": "conn-1", "command": "echo ok" }),
        )
        .expect("dev command should execute");

    assert_eq!(result["isError"], false);
    assert_eq!(result["structuredContent"]["environment"], "dev");
    assert_eq!(result["structuredContent"]["risk_level"], "medium");
    assert!(result["structuredContent"]["stdout"]
        .as_str()
        .unwrap()
        .contains("echo ok"));
}

#[test]
fn exec_rm_rf_requires_confirmation_then_executes() {
    let first = registry()
        .call(
            "unfour.ssh.exec",
            json!({ "connectionId": "conn-1", "command": "rm -rf /tmp/app" }),
        )
        .expect("confirmation should be structured");
    assert_eq!(first["isError"], true);
    assert_eq!(first["structuredContent"]["requires_confirmation"], true);
    let confirmation = first["structuredContent"]["confirmation_text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(confirmation.starts_with("SSH_DELETE_COMMAND:"));

    let confirmed = registry()
        .call(
            "unfour.ssh.exec",
            json!({
                "connectionId": "conn-1",
                "command": "rm -rf /tmp/app",
                "confirm": true,
                "confirmation_text": confirmation
            }),
        )
        .expect("confirmed dev command should execute");
    assert_eq!(confirmed["isError"], false);
    assert!(confirmed["structuredContent"]["stdout"]
        .as_str()
        .unwrap()
        .contains("rm -rf /tmp/app"));
}

#[test]
fn patch_file_multi_match_requires_confirmation_then_replaces_all() {
    let first = registry()
        .call(
            "unfour.ssh.patch_file",
            json!({
                "connectionId": "conn-1",
                "path": "/srv/app/config.toml",
                "search": "debug = false",
                "replace": "debug = true"
            }),
        )
        .expect("multi-match patch should return confirmation");

    assert_eq!(first["isError"], true);
    assert_eq!(first["structuredContent"]["requires_confirmation"], true);
    let confirmation = first["structuredContent"]["confirmation_text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(confirmation.starts_with("SSH_PATCH_MULTIPLE_MATCHES:"));

    let confirmed = registry()
        .call(
            "unfour.ssh.patch_file",
            json!({
                "connectionId": "conn-1",
                "path": "/srv/app/config.toml",
                "search": "debug = false",
                "replace": "debug = true",
                "confirm": true,
                "confirmation_text": confirmation
            }),
        )
        .expect("confirmed multi-match patch should execute");

    assert_eq!(confirmed["isError"], false);
    assert_eq!(confirmed["structuredContent"]["patched"], true);
    assert_eq!(confirmed["structuredContent"]["matches"], 2);
    assert_eq!(
        confirmed["structuredContent"]["diffSummary"]["replacements"],
        2
    );
}

#[test]
fn write_file_is_blocked_in_prod() {
    struct ProdSshCommandBus;

    impl CommandBusAdapter for ProdSshCommandBus {
        fn execute_read(
            &self,
            _command: ReadCommand,
        ) -> Result<ReadCommandResult, CommandBusAdapterError> {
            Ok(ReadCommandResult::CurrentWorkspace(
                CurrentWorkspaceResult {
                    workspace_id: "ws-prod".to_string(),
                    workspace_name: "Prod".to_string(),
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
            unreachable!()
        }
    }

    let registry = ToolRegistry::with_command_bus(Arc::new(ProdSshCommandBus));
    let result = registry
        .call(
            "unfour.ssh.write_file",
            json!({
                "connectionId": "conn-1",
                "path": "/tmp/app/config.txt",
                "content": "safe"
            }),
        )
        .expect("policy denial should be structured");
    assert_eq!(result["isError"], true);
    assert_eq!(result["structuredContent"]["environment"], "prod");
    assert_eq!(
        result["structuredContent"]["error"]["code"],
        "WORKSPACE_POLICY_BLOCKED"
    );
}

#[test]
fn run_diagnostic_requires_connection_and_command() {
    assert!(registry()
        .call("unfour.ssh.run_diagnostic", json!({ "command": "df -h" }))
        .is_err());
    assert!(registry()
        .call(
            "unfour.ssh.run_diagnostic",
            json!({ "connectionId": "conn-1" })
        )
        .is_err());
}

#[test]
fn run_diagnostic_surfaces_unsupported_when_native_disabled() {
    let registry = ToolRegistry::with_command_bus(Arc::new(UnsupportedSshCommandBus));
    let result = registry
        .call(
            "unfour.ssh.run_diagnostic",
            json!({ "connectionId": "conn-1", "command": "uptime" }),
        )
        .expect("execution errors become MCP tool results");

    assert_eq!(result["isError"], true);
    assert_eq!(
        result["structuredContent"]["error"]["code"],
        "COMMAND_BUS_OPERATION_UNSUPPORTED"
    );
}
