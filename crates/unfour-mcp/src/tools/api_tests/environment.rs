use super::*;

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
fn create_environment_returns_created_summary() {
    let result = api_registry()
        .call("unfour.api.create_environment", json!({ "name": "QA" }))
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    assert_eq!(
        result["structuredContent"]["apiEnvironment"]["id"],
        "env-created"
    );
    assert_eq!(result["structuredContent"]["apiEnvironment"]["name"], "QA");
    assert_eq!(
        result["structuredContent"]["apiEnvironment"]["variableCount"],
        0
    );
}

#[test]
fn update_environment_masks_sensitive_variables_in_result() {
    let result = api_registry()
        .call(
            "unfour.api.update_environment",
            json!({
                "environmentId": "env-1",
                "name": "Staging 2",
                "variables": [
                    { "key": "baseUrl", "value": "https://example.com", "enabled": true },
                    { "key": "token", "value": "Bearer secret-token", "enabled": true }
                ]
            }),
        )
        .expect("should succeed");

    assert_eq!(result["isError"], false);
    let environment = &result["structuredContent"]["apiEnvironment"];
    assert_eq!(environment["name"], "Staging 2");
    assert_eq!(environment["variables"][0]["value"], "https://example.com");
    let token = environment["variables"][1]["value"].as_str().unwrap();
    assert!(token.starts_with("[mask "));
    assert!(!token.contains("secret-token"));
}

#[test]
fn update_environment_requires_variables_array() {
    let error = api_registry()
        .call(
            "unfour.api.update_environment",
            json!({
                "environmentId": "env-1",
                "name": "Staging",
                "variables": { "baseUrl": "https://example.com" }
            }),
        )
        .expect_err("should reject invalid variables");

    assert!(matches!(error, ToolCallError::InvalidArguments(_)));
}

#[test]
fn delete_environment_reports_soft_delete() {
    let result = api_registry()
        .call(
            "unfour.api.delete_environment",
            json!({ "environmentId": "env-1" }),
        )
        .expect("full-access dev workspace should allow deletion");

    assert_eq!(result["isError"], false);
    assert_eq!(result["structuredContent"]["softDelete"], true);
    assert_eq!(result["structuredContent"]["deletedEnvironmentId"], "env-1");
    assert_eq!(result["structuredContent"]["remainingCount"], 0);
}
