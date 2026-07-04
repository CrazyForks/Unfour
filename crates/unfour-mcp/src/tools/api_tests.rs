use super::*;
use std::sync::Arc;
use unfour_command_bus::{
    ApiCollectionListResult, ApiCollectionSummary, ApiEnvironmentListResult,
    ApiHistoryDetailResult, ApiHistoryListResult, ApiRequestDetailResult, ApiRequestListResult,
    ApiRequestSummary, CurrentWorkspaceResult, ReadCommand, ReadCommandResult, WorkspaceListResult,
    WorkspaceSummary,
};
use unfour_core::models::{
    ApiEnvironment, ApiHistoryDetail, ApiHistoryItem, ApiResponse, ApiSavedRequest,
    DatabaseConnection, DatabaseQueryInput, DatabaseQueryResult, DatabaseQuerySafety,
    DatabaseSchema, KeyValue,
};

use crate::command_bus_adapter::{CommandBusAdapter, CommandBusAdapterError};
use crate::tools::ToolRegistry;

// --- Test stubs ---

struct ApiStubCommandBus;

impl CommandBusAdapter for ApiStubCommandBus {
    fn execute_read(
        &self,
        command: ReadCommand,
    ) -> Result<ReadCommandResult, CommandBusAdapterError> {
        Ok(match command {
                ReadCommand::CurrentWorkspace => {
                    ReadCommandResult::CurrentWorkspace(CurrentWorkspaceResult {
                        workspace_id: "ws-1".to_string(),
                        workspace_name: "API Workspace".to_string(),
                        environment_type: "dev".to_string(),
                        mcp_policy: "auto".to_string(),
                        workspace_root: None,
                        mode: "local".to_string(),
                        source: "command-bus".to_string(),
                    })
                }
                ReadCommand::ListWorkspaces => ReadCommandResult::Workspaces(WorkspaceListResult {
                    workspaces: vec![WorkspaceSummary {
                        id: "ws-1".to_string(),
                        name: "API Workspace".to_string(),
                        is_default: true,
                        is_active: true,
                        environment_type: "dev".to_string(),
                        mcp_policy: "auto".to_string(),
                        last_opened_at: None,
                    }],
                    active_workspace_id: "ws-1".to_string(),
                    count: 1,
                    source: "command-bus".to_string(),
                }),
                ReadCommand::ApiListCollections { .. } => {
                    ReadCommandResult::ApiCollections(ApiCollectionListResult {
                        collections: vec![
                            ApiCollectionSummary {
                                id: "users".to_string(),
                                name: "Users".to_string(),
                                request_count: 3,
                                workspace_id: "ws-1".to_string(),
                            },
                        ],
                        count: 1,
                        source: "command-bus".to_string(),
                    })
                }
                ReadCommand::ApiListRequests { .. } => {
                    ReadCommandResult::ApiRequests(ApiRequestListResult {
                        requests: vec![ApiRequestSummary {
                            id: "req-1".to_string(),
                            name: "Get Users".to_string(),
                            method: "GET".to_string(),
                            url_preview: "https://api.example.com/users?token=secret123&page=1".to_string(),
                            collection_id: "users".to_string(),
                            workspace_id: "ws-1".to_string(),
                            has_body: false,
                            header_count: 2,
                        }],
                        count: 1,
                        source: "command-bus".to_string(),
                    })
                }
                ReadCommand::ApiGetRequest { request_id } => {
                    ReadCommandResult::ApiRequest(ApiRequestDetailResult {
                        request: ApiSavedRequest {
                            id: request_id,
                            workspace_id: "ws-1".to_string(),
                            name: "Create User".to_string(),
                            collection_id: "users".to_string(),
                            parent_folder_id: Some("folder-users".to_string()),
                            sort_order: 0,
                            auth_json: r#"{"type":"none"}"#.to_string(),
                            method: "POST".to_string(),
                            url: "https://api.example.com/users?api_key=secret".to_string(),
                            headers_json: r#"[{"key":"Authorization","value":"Bearer secret-token","enabled":true},{"key":"Content-Type","value":"application/json","enabled":true}]"#.to_string(),
                            query_json: r#"[{"key":"page","value":"1","enabled":true},{"key":"token","value":"secret","enabled":true}]"#.to_string(),
                            body: Some(r#"{"name":"test","password":"secret123"}"#.to_string()),
                            body_kind: "json".to_string(),
                            created_at: String::new(),
                            updated_at: String::new(),
                            deleted_at: None,
                            revision: 1,
                            sync_status: "local".to_string(),
                            remote_id: None,
                        },
                        source: "command-bus".to_string(),
                    })
                }
                ReadCommand::ApiListHistory { .. } => {
                    ReadCommandResult::ApiHistory(ApiHistoryListResult {
                        history: vec![ApiHistoryItem {
                            id: "hist-1".to_string(),
                            workspace_id: "ws-1".to_string(),
                            name: Some("Get Users".to_string()),
                            method: "GET".to_string(),
                            url: "https://api.example.com/users?token=secret123&page=2".to_string(),
                            status: Some(500),
                            duration_ms: Some(87),
                            created_at: "2026-06-20T00:00:00Z".to_string(),
                            updated_at: "2026-06-20T00:00:00Z".to_string(),
                        }],
                        count: 1,
                        source: "command-bus".to_string(),
                    })
                }
                ReadCommand::ApiGetHistory { history_id, .. } => {
                    ReadCommandResult::ApiHistoryDetailResult(ApiHistoryDetailResult {
                        detail: ApiHistoryDetail {
                            id: history_id,
                            workspace_id: "ws-1".to_string(),
                            name: Some("Create User".to_string()),
                            method: "POST".to_string(),
                            url: "https://api.example.com/users?api_key=secret".to_string(),
                            request_headers_json: r#"[{"key":"Authorization","value":"Bearer secret-token","enabled":true}]"#.to_string(),
                            request_query_json: r#"[{"key":"token","value":"secret","enabled":true}]"#.to_string(),
                            request_body: Some(r#"{"name":"test","password":"secret123"}"#.to_string()),
                            status: Some(401),
                            duration_ms: Some(120),
                            response_headers_json: r#"[{"key":"Set-Cookie","value":"session=secret-session-id","enabled":true}]"#.to_string(),
                            response_body_preview: Some(r#"{"error":"unauthorized","token":"secret-jwt"}"#.to_string()),
                            created_at: "2026-06-20T00:00:00Z".to_string(),
                            updated_at: "2026-06-20T00:00:00Z".to_string(),
                        },
                        source: "command-bus".to_string(),
                    })
                }
                ReadCommand::ApiListEnvironments { .. } => {
                    ReadCommandResult::ApiEnvironments(ApiEnvironmentListResult {
                        environments: vec![ApiEnvironment {
                            id: "env-1".to_string(),
                            workspace_id: "ws-1".to_string(),
                            name: "Staging".to_string(),
                            variables: vec![
                                KeyValue {
                                    key: "baseUrl".to_string(),
                                    value: "https://api.staging.example.com".to_string(),
                                    enabled: true,
                                },
                                KeyValue {
                                    key: "token".to_string(),
                                    value: "Bearer secret-token".to_string(),
                                    enabled: true,
                                },
                            ],
                            is_active: true,
                            created_at: String::new(),
                            updated_at: String::new(),
                        }],
                        count: 1,
                        source: "command-bus".to_string(),
                    })
                }
                _ => ReadCommandResult::ApiCollections(ApiCollectionListResult {
                    collections: vec![],
                    count: 0,
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
            history_id: "hist-1".to_string(),
            status: 200,
            status_text: "OK".to_string(),
            headers: vec![
                KeyValue {
                    key: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                    enabled: true,
                },
                KeyValue {
                    key: "Set-Cookie".to_string(),
                    value: "session=secret-session-id".to_string(),
                    enabled: true,
                },
            ],
            body: r#"{"ok":true,"token":"secret-jwt"}"#.to_string(),
            duration_ms: 123,
        })
    }

    fn send_api_request(
        &self,
        input: ApiRequestInput,
    ) -> Result<ApiResponse, CommandBusAdapterError> {
        assert_eq!(input.method, "POST");
        assert_eq!(input.workspace_id, "ws-1");
        Ok(ApiResponse {
            history_id: "hist-post".to_string(),
            status: 201,
            status_text: "Created".to_string(),
            headers: vec![KeyValue {
                key: "Set-Cookie".to_string(),
                value: "session=secret-session-id".to_string(),
                enabled: true,
            }],
            body: r#"{"id":1,"token":"secret-jwt"}"#.to_string(),
            duration_ms: 77,
        })
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

struct FailingApiCommandBus;

impl CommandBusAdapter for FailingApiCommandBus {
    fn execute_read(
        &self,
        command: ReadCommand,
    ) -> Result<ReadCommandResult, CommandBusAdapterError> {
        match command {
            ReadCommand::CurrentWorkspace => Ok(ReadCommandResult::CurrentWorkspace(
                CurrentWorkspaceResult {
                    workspace_id: "ws-1".to_string(),
                    workspace_name: "API Workspace".to_string(),
                    environment_type: "dev".to_string(),
                    mcp_policy: "auto".to_string(),
                    workspace_root: None,
                    mode: "local".to_string(),
                    source: "command-bus".to_string(),
                },
            )),
            ReadCommand::ListWorkspaces => Ok(ReadCommandResult::Workspaces(WorkspaceListResult {
                workspaces: vec![WorkspaceSummary {
                    id: "ws-1".to_string(),
                    name: "API Workspace".to_string(),
                    is_default: true,
                    is_active: true,
                    environment_type: "dev".to_string(),
                    mcp_policy: "auto".to_string(),
                    last_opened_at: None,
                }],
                active_workspace_id: "ws-1".to_string(),
                count: 1,
                source: "command-bus".to_string(),
            })),
            ReadCommand::ApiGetRequest { request_id } => {
                Ok(ReadCommandResult::ApiRequest(ApiRequestDetailResult {
                    request: ApiSavedRequest {
                        id: request_id,
                        workspace_id: "ws-1".to_string(),
                        name: "Create User".to_string(),
                        collection_id: "users".to_string(),
                        parent_folder_id: None,
                        sort_order: 0,
                        auth_json: r#"{"type":"none"}"#.to_string(),
                        method: "POST".to_string(),
                        url: "https://api.example.com/users".to_string(),
                        headers_json: "[]".to_string(),
                        query_json: "[]".to_string(),
                        body: None,
                        body_kind: "json".to_string(),
                        created_at: String::new(),
                        updated_at: String::new(),
                        deleted_at: None,
                        revision: 1,
                        sync_status: "local".to_string(),
                        remote_id: None,
                    },
                    source: "command-bus".to_string(),
                }))
            }
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

fn api_registry() -> ToolRegistry {
    ToolRegistry::with_command_bus(Arc::new(ApiStubCommandBus))
}

// --- Schema tests ---

#[test]
fn api_tools_are_registered() {
    let definitions = api_registry().definitions();
    assert!(definitions
        .iter()
        .any(|d| d.name == "unfour.api.list_collections"));
    assert!(definitions
        .iter()
        .any(|d| d.name == "unfour.api.list_requests"));
    assert!(definitions
        .iter()
        .any(|d| d.name == "unfour.api.get_request"));
    assert!(definitions
        .iter()
        .any(|d| d.name == "unfour.api.send_request"));
}

#[test]
fn api_tools_have_valid_input_schemas() {
    let definitions = api_registry().definitions();
    for name in &[
        "unfour.api.list_collections",
        "unfour.api.list_requests",
        "unfour.api.get_request",
        "unfour.api.send_request",
    ] {
        let def = definitions.iter().find(|d| d.name == *name).unwrap();
        assert_eq!(
            def.input_schema["type"], "object",
            "{} should have object input schema",
            name
        );
    }
}

// --- list_collections tests ---

#[test]
fn list_collections_returns_structured_result() {
    let result = api_registry()
        .call("unfour.api.list_collections", json!({}))
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    assert_eq!(result["structuredContent"]["count"], 1);
    assert_eq!(
        result["structuredContent"]["collections"][0]["name"],
        "Users"
    );
    assert_eq!(
        result["structuredContent"]["collections"][0]["requestCount"],
        3
    );
    assert_eq!(result["structuredContent"]["source"], "command-bus");
}

// --- list_requests tests ---

#[test]
fn list_requests_redacts_sensitive_url_params() {
    let result = api_registry()
        .call("unfour.api.list_requests", json!({}))
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    let requests = &result["structuredContent"]["requests"];
    assert_eq!(requests[0]["id"], "req-1");
    // token should be redacted in urlPreview
    let url_preview = requests[0]["urlPreview"].as_str().unwrap();
    assert!(
        url_preview.contains("token=[mask "),
        "token should be masked in urlPreview"
    );
    assert!(
        !url_preview.contains("secret123"),
        "raw token should not appear"
    );
    assert!(url_preview.contains("page=1"), "safe params preserved");
}

// --- get_request tests ---

#[test]
fn get_request_redacts_sensitive_data() {
    let result = api_registry()
        .call("unfour.api.get_request", json!({ "requestId": "req-1" }))
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    let request = &result["structuredContent"]["request"];

    // URL query params masked
    let url = request["url"].as_str().unwrap();
    assert!(
        url.contains("api_key=[mask "),
        "api_key should be masked in URL"
    );
    assert!(!url.contains("=secret"), "raw secret should not appear");

    // Authorization header masked (scheme preserved for diagnosis)
    let headers = request["headers"].as_array().unwrap();
    let auth_header = headers
        .iter()
        .find(|h| h["key"] == "Authorization")
        .unwrap();
    let auth_value = auth_header["value"].as_str().unwrap();
    assert!(auth_value.starts_with("[mask "));
    assert!(auth_value.contains("scheme=Bearer"));
    assert!(!auth_value.contains("secret-token"));

    // Content-Type preserved
    let ct_header = headers.iter().find(|h| h["key"] == "Content-Type").unwrap();
    assert_eq!(ct_header["value"], "application/json");

    // Query param token masked
    let query = request["query"].as_array().unwrap();
    let token_param = query.iter().find(|q| q["key"] == "token").unwrap();
    assert!(token_param["value"].as_str().unwrap().starts_with("[mask "));

    // Body password masked
    let body = request["bodyPreview"].as_str().unwrap();
    assert!(body.contains("[mask "), "password should be masked in body");
    assert!(
        !body.contains("secret123"),
        "raw password should not appear"
    );
    assert!(body.contains("test"), "non-sensitive body values preserved");

    assert_eq!(request["collectionId"], "users");
    assert_eq!(result["structuredContent"]["source"], "command-bus");
}

#[test]
fn get_request_requires_request_id() {
    let result = api_registry().call("unfour.api.get_request", json!({}));
    assert!(result.is_err(), "should fail without requestId");
}

// --- send_request tests ---

#[test]
fn send_request_returns_success_with_redacted_response() {
    let result = api_registry()
        .call("unfour.api.send_request", json!({ "requestId": "req-1" }))
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    let content = &result["structuredContent"];
    assert_eq!(content["ok"], true);
    assert_eq!(content["status"], 200);
    assert_eq!(content["statusText"], "OK");
    assert_eq!(content["durationMs"], 123);
    assert_eq!(content["source"], "command-bus");

    // Set-Cookie response header masked
    let headers = content["headers"].as_array().unwrap();
    let set_cookie = headers.iter().find(|h| h["name"] == "Set-Cookie").unwrap();
    assert!(set_cookie["value"].as_str().unwrap().starts_with("[mask "));

    // Body token masked
    let body = content["bodyPreview"].as_str().unwrap();
    assert!(
        body.contains("[mask "),
        "token should be masked in response body"
    );
    assert!(!body.contains("secret-jwt"), "raw token should not appear");
}

#[test]
fn send_request_allows_dev_post_ad_hoc() {
    let result = api_registry()
        .call(
            "unfour.api.send_request",
            json!({
                "method": "POST",
                "url": "https://api.example.com/users",
                "headers": { "Authorization": "Bearer secret-token" },
                "body": "{\"name\":\"test\"}",
                "bodyKind": "json"
            }),
        )
        .expect("dev POST should be allowed");

    let content = &result["structuredContent"];
    assert_eq!(result["isError"], false);
    assert_eq!(content["environment"], "dev");
    assert_eq!(content["risk_level"], "medium");
    assert_eq!(content["status"], 201);
    assert!(!result.to_string().contains("secret-token"));
    assert!(!result.to_string().contains("secret-jwt"));
}

#[test]
fn send_request_blocks_prod_delete_ad_hoc() {
    struct ProdApiCommandBus;

    impl CommandBusAdapter for ProdApiCommandBus {
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
            panic!("prod DELETE should be blocked before execution")
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

    let registry = ToolRegistry::with_command_bus(Arc::new(ProdApiCommandBus));
    let result = registry
        .call(
            "unfour.api.send_request",
            json!({ "method": "DELETE", "url": "https://api.example.com/users/1" }),
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
fn send_request_clamps_timeout_to_60s() {
    // Sending with 120000ms should be clamped - the stub ignores timeout,
    // but we verify the tool doesn't reject the call
    let result = api_registry()
        .call(
            "unfour.api.send_request",
            json!({ "requestId": "req-1", "timeoutMs": 120000 }),
        )
        .expect("should succeed");
    assert_eq!(result["structuredContent"]["ok"], true);
}

#[test]
fn send_request_rejects_missing_request_id() {
    let result = api_registry().call("unfour.api.send_request", json!({}));
    assert!(result.is_err(), "should fail without requestId");
}

#[test]
fn send_request_returns_structured_error_on_failure() {
    let registry = ToolRegistry::with_command_bus(Arc::new(FailingApiCommandBus));
    let result = registry
        .call("unfour.api.send_request", json!({ "requestId": "req-1" }))
        .expect("execution errors become MCP tool results");

    assert_eq!(result["isError"], true);
    assert_eq!(
        result["structuredContent"]["error"]["code"],
        "COMMAND_BUS_API_SEND_FAILED"
    );
}

#[test]
fn command_bus_read_failure_returns_structured_error() {
    let registry = ToolRegistry::with_command_bus(Arc::new(FailingApiCommandBus));
    let result = registry
        .call("unfour.api.list_collections", json!({}))
        .expect("execution errors become MCP tool results");

    assert_eq!(result["isError"], true);
    assert_eq!(
        result["structuredContent"]["error"]["code"],
        "COMMAND_BUS_READ_FAILED"
    );
}

// --- history tests ---

#[test]
fn list_history_masks_url_and_returns_status() {
    let result = api_registry()
        .call("unfour.api.list_history", json!({}))
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    let content = &result["structuredContent"];
    assert_eq!(content["count"], 1);
    let item = &content["history"][0];
    assert_eq!(item["status"], 500);
    let url = item["url"].as_str().unwrap();
    assert!(url.contains("token=[mask "), "token should be masked");
    assert!(!url.contains("secret123"), "raw token should not appear");
    assert!(url.contains("page=2"), "safe params preserved");
}

#[test]
fn get_history_masks_request_and_response() {
    let result = api_registry()
        .call("unfour.api.get_history", json!({ "historyId": "hist-1" }))
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    let h = &result["structuredContent"]["history"];
    assert_eq!(h["status"], 401);

    let url = h["url"].as_str().unwrap();
    assert!(url.contains("api_key=[mask "));
    assert!(!url.contains("=secret"));

    let req_headers = h["requestHeaders"].as_array().unwrap();
    let auth = req_headers
        .iter()
        .find(|x| x["key"] == "Authorization")
        .unwrap();
    let auth_val = auth["value"].as_str().unwrap();
    assert!(auth_val.starts_with("[mask "));
    assert!(auth_val.contains("scheme=Bearer"));
    assert!(!auth_val.contains("secret-token"));

    let resp_headers = h["responseHeaders"].as_array().unwrap();
    let cookie = resp_headers
        .iter()
        .find(|x| x["key"] == "Set-Cookie")
        .unwrap();
    assert!(cookie["value"].as_str().unwrap().starts_with("[mask "));

    let req_body = h["requestBody"].as_str().unwrap();
    assert!(req_body.contains("[mask "));
    assert!(!req_body.contains("secret123"));

    let resp_body = h["responseBodyPreview"].as_str().unwrap();
    assert!(resp_body.contains("[mask "));
    assert!(!resp_body.contains("secret-jwt"));
}

#[test]
fn get_history_requires_history_id() {
    let result = api_registry().call("unfour.api.get_history", json!({}));
    assert!(result.is_err(), "should fail without historyId");
}

// --- environment tests ---

#[test]
fn list_environments_masks_sensitive_variables_only() {
    let result = api_registry()
        .call("unfour.api.list_environments", json!({}))
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    let env = &result["structuredContent"]["environments"][0];
    assert_eq!(env["name"], "Staging");
    assert_eq!(env["isActive"], true);
    assert_eq!(env["variableCount"], 2);

    let vars = env["variables"].as_array().unwrap();
    let base = vars.iter().find(|v| v["key"] == "baseUrl").unwrap();
    // Non-sensitive value is shown verbatim so requests are intelligible.
    assert_eq!(base["value"], "https://api.staging.example.com");

    let token = vars.iter().find(|v| v["key"] == "token").unwrap();
    let token_val = token["value"].as_str().unwrap();
    assert!(token_val.starts_with("[mask "));
    assert!(token_val.contains("scheme=Bearer"));
    assert!(!token_val.contains("secret-token"));
}

#[test]
fn unknown_tool_returns_error() {
    let result = api_registry().call("unfour.api.nonexistent", json!({}));
    assert!(result.is_err());
    match result.unwrap_err() {
        ToolCallError::UnknownTool(name) => assert_eq!(name, "unfour.api.nonexistent"),
        other => panic!("expected UnknownTool, got {:?}", other),
    }
}

#[test]
fn body_truncation_works_at_20kb() {
    let large_body = "x".repeat(30_000);
    let (truncated, was_truncated) = truncate_body(&large_body, MAX_BODY_PREVIEW_BYTES);
    assert!(was_truncated);
    assert_eq!(truncated.len(), MAX_BODY_PREVIEW_BYTES);
}
