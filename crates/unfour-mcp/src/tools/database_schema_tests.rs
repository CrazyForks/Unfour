use serde_json::json;

use super::super::ToolDefinition;
use super::registered_tools;

fn definitions() -> Vec<ToolDefinition> {
    registered_tools()
        .into_iter()
        .map(|tool| tool.definition)
        .collect()
}

// --- Schema / registration tests ---

#[test]
fn db_tools_are_registered() {
    let definitions = definitions();
    assert!(definitions
        .iter()
        .any(|d| d.name == "unfour.db.list_connections"));
    assert!(definitions
        .iter()
        .any(|d| d.name == "unfour.db.list_tables"));
    assert!(definitions
        .iter()
        .any(|d| d.name == "unfour.db.describe_table"));
    assert!(definitions
        .iter()
        .any(|d| d.name == "unfour.db.query_readonly"));
    assert!(definitions.iter().any(|d| d.name == "unfour.db.execute"));
    assert!(definitions.iter().any(|d| d.name == "unfour.db.explain"));
}

#[test]
fn db_list_connections_input_schema() {
    let definitions = definitions();
    let tool = definitions
        .iter()
        .find(|d| d.name == "unfour.db.list_connections")
        .unwrap();
    assert_eq!(tool.input_schema["type"], "object");
    assert!(tool.input_schema["properties"]["workspaceId"].is_object());
}

#[test]
fn db_list_tables_input_schema() {
    let definitions = definitions();
    let tool = definitions
        .iter()
        .find(|d| d.name == "unfour.db.list_tables")
        .unwrap();
    assert_eq!(tool.input_schema["type"], "object");
    assert_eq!(
        tool.input_schema["required"].as_array().unwrap(),
        &vec![json!("connectionId")]
    );
}

#[test]
fn db_describe_table_input_schema() {
    let definitions = definitions();
    let tool = definitions
        .iter()
        .find(|d| d.name == "unfour.db.describe_table")
        .unwrap();
    assert_eq!(tool.input_schema["type"], "object");
    let required = tool.input_schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("connectionId")));
    assert!(required.contains(&json!("tableName")));
}
