use serde_json::{json, Value};

pub fn structured_tool_result(
    tool: &str,
    environment: &str,
    risk_level: &str,
    duration_ms: u128,
    value: Value,
) -> Value {
    let mut envelope = json!({
        "ok": true,
        "tool": tool,
        "environment": environment,
        "risk_level": risk_level,
        "duration_ms": duration_ms,
        "data": value.clone(),
        "warnings": [],
        "redactions": []
    });
    merge_object_fields(&mut envelope, &value);
    let value = envelope;
    let text = serde_json::to_string(&value).expect("serializing a JSON value cannot fail");

    json!({
        "content": [
            {
                "type": "text",
                "text": text,
            }
        ],
        "structuredContent": value,
        "isError": false,
    })
}

pub fn structured_tool_error(
    tool: &str,
    environment: &str,
    risk_level: &str,
    duration_ms: u128,
    code: &str,
    message: &str,
) -> Value {
    let value = json!({
        "ok": false,
        "tool": tool,
        "environment": environment,
        "risk_level": risk_level,
        "duration_ms": duration_ms,
        "error": {
            "code": code,
            "message": message,
            "details": {}
        },
        "requires_confirmation": false
    });
    let text = serde_json::to_string(&value).expect("serializing a JSON value cannot fail");

    json!({
        "content": [
            {
                "type": "text",
                "text": text,
            }
        ],
        "structuredContent": value,
        "isError": true,
    })
}

pub fn structured_policy_error(
    tool: &str,
    environment: &str,
    risk_level: &str,
    duration_ms: u128,
    value: Value,
) -> Value {
    let mut envelope = json!({
        "ok": false,
        "tool": tool,
        "environment": environment,
        "risk_level": risk_level,
        "duration_ms": duration_ms,
        "error": {
            "code": value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(Value::as_str)
                .unwrap_or("PERMISSION_DENIED"),
            "message": value
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("The MCP tool call was blocked by policy."),
            "details": value,
        },
        "requires_confirmation": false
    });
    merge_object_fields(&mut envelope, &value);
    let value = envelope;
    let text = serde_json::to_string(&value).expect("serializing a JSON value cannot fail");

    json!({
        "content": [
            {
                "type": "text",
                "text": text,
            }
        ],
        "structuredContent": value,
        "isError": true,
    })
}

pub fn structured_confirmation_required(
    tool: &str,
    environment: &str,
    risk_level: &str,
    duration_ms: u128,
    value: Value,
) -> Value {
    let mut envelope = json!({
        "ok": false,
        "tool": tool,
        "environment": environment,
        "risk_level": risk_level,
        "duration_ms": duration_ms,
        "requires_confirmation": true,
        "reason": value
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("This MCP tool call requires confirmation."),
        "confirmation_text": value
            .get("confirmationText")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        "confirmation_hint": value
            .get("confirmationHint")
            .and_then(Value::as_str)
            .unwrap_or("Re-run with confirm=true and the exact confirmation_text."),
        "dry_run": true,
        "details": value,
        "error": {
            "code": "CONFIRMATION_REQUIRED",
            "message": value
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("This MCP tool call requires confirmation."),
            "details": {}
        }
    });
    merge_object_fields(&mut envelope, &value);
    let value = envelope;
    let text = serde_json::to_string(&value).expect("serializing a JSON value cannot fail");

    json!({
        "content": [
            {
                "type": "text",
                "text": text,
            }
        ],
        "structuredContent": value,
        "isError": true,
    })
}

fn merge_object_fields(target: &mut Value, source: &Value) {
    let (Some(target), Some(source)) = (target.as_object_mut(), source.as_object()) else {
        return;
    };
    for (key, value) in source {
        target.entry(key.clone()).or_insert_with(|| value.clone());
    }
}
