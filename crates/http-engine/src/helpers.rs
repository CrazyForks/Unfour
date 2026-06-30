//! Pure helpers for the API client service: HTTP method parsing, URL building,
//! environment template resolution, validation, and the sqlx row types used by
//! the storage layer. Extracted from `api_client.rs` to keep the service module
//! focused on orchestration; visibility is `pub(super)` so only the
//! `api_client` module (and its test submodules) can reach these items.

use reqwest::{Method, Url};
use std::collections::HashSet;
use unfour_core::models::{ApiCollection, ApiEnvironment, ApiRequestInput, KeyValue};
use unfour_core::{AppError, AppResult};

pub(super) fn parse_method(method: &str) -> AppResult<Method> {
    Method::from_bytes(method.trim().to_uppercase().as_bytes())
        .map_err(|_| AppError::Validation(format!("invalid HTTP method: {}", method)))
}

#[derive(sqlx::FromRow)]
pub(super) struct EnvironmentRow {
    pub(super) id: String,
    pub(super) workspace_id: String,
    pub(super) name: String,
    pub(super) variables_json: String,
    pub(super) is_active: bool,
    pub(super) created_at: String,
    pub(super) updated_at: String,
}

impl From<EnvironmentRow> for ApiEnvironment {
    fn from(row: EnvironmentRow) -> Self {
        ApiEnvironment {
            id: row.id,
            workspace_id: row.workspace_id,
            name: row.name,
            variables: serde_json::from_str(&row.variables_json).unwrap_or_default(),
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct CollectionRow {
    pub(super) id: String,
    pub(super) workspace_id: String,
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) created_at: String,
    pub(super) updated_at: String,
}

impl From<CollectionRow> for ApiCollection {
    fn from(row: CollectionRow) -> Self {
        ApiCollection {
            id: row.id,
            workspace_id: row.workspace_id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

pub(super) fn normalize_collection_id(value: Option<String>) -> Option<String> {
    normalize_entity_id(value)
}

pub(super) fn normalize_entity_id(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub(super) fn normalize_folder_name(value: String) -> AppResult<String> {
    let name = value.trim();
    if name.is_empty() {
        return Err(AppError::Validation(
            "folder name cannot be empty".to_string(),
        ));
    }
    if name.chars().count() > 120 {
        return Err(AppError::Validation(
            "folder name must be 120 characters or fewer".to_string(),
        ));
    }
    if name
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '<' | '>' | '"' | '|'))
    {
        return Err(AppError::Validation(format!(
            "invalid folder name: {}",
            value
        )));
    }
    Ok(name.to_string())
}

pub(super) fn validate_environment(variables: &[KeyValue]) -> AppResult<()> {
    let mut seen = HashSet::new();
    for variable in variables {
        let key = variable.key.trim();
        if key.is_empty() {
            continue;
        }
        let valid = key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'));
        if !valid {
            return Err(AppError::Validation(format!(
                "invalid environment variable name: {}",
                variable.key
            )));
        }
        if variable.enabled && !seen.insert(key.to_ascii_lowercase()) {
            return Err(AppError::Validation(format!(
                "duplicate environment variable name: {}",
                variable.key
            )));
        }
    }

    Ok(())
}

pub(super) fn build_url(raw_url: &str, query: &[KeyValue]) -> AppResult<Url> {
    let mut url = Url::parse(raw_url.trim())
        .map_err(|_| AppError::Validation(format!("invalid URL: {}", raw_url)))?;

    {
        let mut pairs = url.query_pairs_mut();
        for item in query
            .iter()
            .filter(|item| item.enabled && !item.key.is_empty())
        {
            pairs.append_pair(&item.key, &item.value);
        }
    }

    Ok(url)
}

pub(super) fn validate_workspace_id(workspace_id: &str) -> AppResult<()> {
    if workspace_id.trim().is_empty() {
        return Err(AppError::Validation(
            "workspace id cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn resolve_input(
    mut input: ApiRequestInput,
    environment: &[KeyValue],
) -> AppResult<ApiRequestInput> {
    input.url = resolve_template(&input.url, environment)?;
    input.headers = resolve_key_values(&input.headers, environment)?;
    input.query = resolve_key_values(&input.query, environment)?;
    input.body = input
        .body
        .map(|body| resolve_template(&body, environment))
        .transpose()?;

    Ok(input)
}

pub(super) fn resolve_key_values(
    items: &[KeyValue],
    environment: &[KeyValue],
) -> AppResult<Vec<KeyValue>> {
    items
        .iter()
        .map(|item| {
            Ok(KeyValue {
                key: resolve_template(&item.key, environment)?,
                value: resolve_template(&item.value, environment)?,
                enabled: item.enabled,
            })
        })
        .collect()
}

pub(super) fn resolve_template(value: &str, environment: &[KeyValue]) -> AppResult<String> {
    let mut output = value.to_string();
    for variable in environment
        .iter()
        .filter(|item| item.enabled && !item.key.trim().is_empty())
    {
        let token = format!("{{{{{}}}}}", variable.key.trim());
        output = output.replace(&token, &variable.value);
    }

    if let Some(start) = output.find("{{") {
        if let Some(end) = output[start + 2..].find("}}") {
            let name = &output[start + 2..start + 2 + end];
            return Err(AppError::Validation(format!(
                "missing environment variable: {}",
                name
            )));
        }
    }

    Ok(output)
}
