use std::collections::{HashMap, VecDeque};

use reqwest::Url;
use serde_json::Value;
use unfour_core::models::KeyValue;
use unfour_core::redaction::{
    is_sensitive_key, redact_json_body, redact_key_values, REDACTED_VALUE,
};
use unfour_core::{AppError, AppResult};

use super::super::DEFAULT_AUTH_JSON;

pub(super) fn snapshot_auth_json(value: &str) -> String {
    if serde_json::from_str::<Value>(value).is_err() {
        return REDACTED_VALUE.to_string();
    }
    redact_json_body(value).0
}

pub(super) fn snapshot_key_values(value: &str) -> AppResult<Vec<KeyValue>> {
    let items = serde_json::from_str::<Vec<KeyValue>>(value).map_err(|_| {
        AppError::Config("stored API key-value configuration is invalid".to_string())
    })?;
    Ok(redact_key_values(
        items,
        |item| &item.key,
        |item, redacted| item.value = redacted,
    ))
}

pub(super) fn snapshot_body(value: Option<&str>) -> Option<String> {
    value.map(|value| redact_json_body(value).0)
}

pub(super) fn snapshot_url(value: &str) -> String {
    transform_url(value, None, true)
}

pub(super) fn restore_auth_json(external: &str, current: Option<&str>) -> String {
    if external == REDACTED_VALUE {
        return current.unwrap_or(DEFAULT_AUTH_JSON).to_string();
    }
    restore_redacted_json(external, current)
        .unwrap_or_else(|| current.unwrap_or(DEFAULT_AUTH_JSON).to_string())
}

pub(super) fn restore_key_values(
    external: Vec<KeyValue>,
    current_json: Option<&str>,
) -> Vec<KeyValue> {
    let current = current_json
        .and_then(|value| serde_json::from_str::<Vec<KeyValue>>(value).ok())
        .unwrap_or_default();
    external
        .into_iter()
        .map(|mut item| {
            if is_sensitive_key(&item.key) {
                item.value = if item.value == REDACTED_VALUE {
                    current
                        .iter()
                        .find(|candidate| candidate.key.eq_ignore_ascii_case(&item.key))
                        .map(|candidate| candidate.value.clone())
                        .unwrap_or_default()
                } else {
                    String::new()
                };
            }
            item
        })
        .collect()
}

pub(super) fn restore_body(external: Option<String>, current: Option<&str>) -> Option<String> {
    external.map(|value| restore_redacted_json(&value, current).unwrap_or(value))
}

pub(super) fn restore_url(external: &str, current: Option<&str>) -> String {
    transform_url(external, current, false)
}

fn restore_redacted_json(external: &str, current: Option<&str>) -> Option<String> {
    let mut external = serde_json::from_str::<Value>(external).ok()?;
    let current = current.and_then(|value| serde_json::from_str::<Value>(value).ok());
    restore_redacted_value(&mut external, current.as_ref());
    serde_json::to_string(&external).ok()
}

fn restore_redacted_value(external: &mut Value, current: Option<&Value>) {
    match external {
        Value::Object(fields) => {
            for (key, value) in fields {
                let current = current
                    .and_then(Value::as_object)
                    .and_then(|map| map.get(key));
                if is_sensitive_key(key) {
                    *value = if value.as_str() == Some(REDACTED_VALUE) {
                        current
                            .cloned()
                            .unwrap_or_else(|| Value::String(String::new()))
                    } else {
                        Value::String(String::new())
                    };
                } else {
                    restore_redacted_value(value, current);
                }
            }
        }
        Value::Array(items) => {
            for (index, value) in items.iter_mut().enumerate() {
                let current = current
                    .and_then(Value::as_array)
                    .and_then(|items| items.get(index));
                restore_redacted_value(value, current);
            }
        }
        _ => {}
    }
}

fn transform_url(value: &str, local: Option<&str>, snapshot: bool) -> String {
    let Some((prefix, query, fragment)) = split_url_query(value) else {
        return value.to_string();
    };
    let local_values = local
        .and_then(split_url_query)
        .map(|(_, query, _)| sensitive_query_values(query))
        .unwrap_or_default();
    let mut local_values = local_values;
    let query = query
        .split('&')
        .map(|component| {
            let Some((key, decoded_value)) = decode_query_component(component) else {
                return component.to_string();
            };
            if !is_sensitive_key(&key) {
                return component.to_string();
            }
            let raw_key = component
                .split_once('=')
                .map(|(key, _)| key)
                .unwrap_or(component);
            let value = if snapshot {
                "%3Credacted%3E".to_string()
            } else if decoded_value == REDACTED_VALUE {
                local_values
                    .get_mut(&key.to_ascii_lowercase())
                    .and_then(VecDeque::pop_front)
                    .unwrap_or_default()
            } else {
                String::new()
            };
            format!("{raw_key}={value}")
        })
        .collect::<Vec<_>>()
        .join("&");
    format!("{prefix}?{query}{fragment}")
}

fn split_url_query(value: &str) -> Option<(&str, &str, &str)> {
    let (without_fragment, fragment) = value
        .split_once('#')
        .map(|(url, _)| (url, &value[url.len()..]))
        .unwrap_or((value, ""));
    let (prefix, query) = without_fragment.split_once('?')?;
    Some((prefix, query, fragment))
}

fn sensitive_query_values(query: &str) -> HashMap<String, VecDeque<String>> {
    let mut values = HashMap::<String, VecDeque<String>>::new();
    for component in query.split('&') {
        let Some((key, _)) = decode_query_component(component) else {
            continue;
        };
        if !is_sensitive_key(&key) {
            continue;
        }
        let raw_value = component
            .split_once('=')
            .map(|(_, value)| value)
            .unwrap_or_default();
        values
            .entry(key.to_ascii_lowercase())
            .or_default()
            .push_back(raw_value.to_string());
    }
    values
}

fn decode_query_component(component: &str) -> Option<(String, String)> {
    let url = Url::parse(&format!("https://sync.invalid/?{component}")).ok()?;
    url.query_pairs()
        .next()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_secret_helpers_redact_and_restore_local_material() {
        let auth = r#"{"type":"bearer","token":"device-token"}"#;
        let snapshot = snapshot_auth_json(auth);
        assert!(!snapshot.contains("device-token"));
        assert!(snapshot.contains(REDACTED_VALUE));
        assert_eq!(
            serde_json::from_str::<Value>(&restore_auth_json(&snapshot, Some(auth))).unwrap(),
            serde_json::from_str::<Value>(auth).unwrap()
        );

        let url = "https://example.test/users?access_token=secret&page=1";
        let redacted = snapshot_url(url);
        assert!(!redacted.contains("secret"));
        assert_eq!(restore_url(&redacted, Some(url)), url);
    }

    #[test]
    fn url_secret_helpers_preserve_templates_encoding_order_and_fragments() {
        let url = "{{base_url}}/users/%7Bid%7D?access_token=device%2Bsecret&page=1&access_token=second#result";
        let redacted = snapshot_url(url);
        assert_eq!(
            redacted,
            "{{base_url}}/users/%7Bid%7D?access_token=%3Credacted%3E&page=1&access_token=%3Credacted%3E#result"
        );
        assert_eq!(restore_url(&redacted, Some(url)), url);
        assert_eq!(
            snapshot_url("{{base_url}}/users/%7Bid%7D"),
            "{{base_url}}/users/%7Bid%7D"
        );
    }
}
