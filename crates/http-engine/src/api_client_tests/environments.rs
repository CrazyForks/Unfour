use super::super::*;
use super::support::service;

#[tokio::test]
async fn environment_lifecycle_create_update_activate_delete() {
    let service = service().await;

    // First environment auto-activates.
    let dev = service
        .create_environment("workspace-a".to_string(), "Dev".to_string())
        .await
        .expect("create dev");
    assert!(dev.is_active);

    // Second does not.
    let prod = service
        .create_environment("workspace-a".to_string(), "Prod".to_string())
        .await
        .expect("create prod");
    assert!(!prod.is_active);

    // Update variables on prod.
    let prod = service
        .update_environment(
            "workspace-a".to_string(),
            prod.id.clone(),
            "Prod".to_string(),
            vec![KeyValue {
                key: "base_url".to_string(),
                value: "https://api.example.test".to_string(),
                enabled: true,
            }],
        )
        .await
        .expect("update prod");
    assert_eq!(prod.variables.len(), 1);

    // Activating prod clears dev's active flag (single-active invariant).
    let list = service
        .activate_environment("workspace-a".to_string(), Some(prod.id.clone()))
        .await
        .expect("activate prod");
    assert_eq!(list.iter().filter(|e| e.is_active).count(), 1);
    assert!(list.iter().find(|e| e.id == prod.id).unwrap().is_active);
    assert!(!list.iter().find(|e| e.id == dev.id).unwrap().is_active);

    let meta_rows: Vec<(String, i64, String)> = sqlx::query_as(
        r#"
        SELECT id, revision, sync_status
        FROM api_environments
        WHERE id = ?1 OR id = ?2
        "#,
    )
    .bind(&dev.id)
    .bind(&prod.id)
    .fetch_all(service.db.pool())
    .await
    .expect("environment metadata");
    let dev_meta = meta_rows
        .iter()
        .find(|(id, _, _)| id == &dev.id)
        .expect("dev metadata");
    let prod_meta = meta_rows
        .iter()
        .find(|(id, _, _)| id == &prod.id)
        .expect("prod metadata");
    assert_eq!(dev_meta.1, 2);
    assert_eq!(dev_meta.2, "pending");
    assert_eq!(prod_meta.1, 3);
    assert_eq!(prod_meta.2, "pending");

    // send() should resolve {{base_url}} from the active (prod) environment.
    let resolved = service
        .active_environment_variables("workspace-a")
        .await
        .expect("active vars");
    assert_eq!(resolved[0].value, "https://api.example.test");

    // Deleting the active environment leaves no active env ("No Environment").
    let remaining = service
        .delete_environment("workspace-a".to_string(), prod.id.clone())
        .await
        .expect("delete prod");
    assert!(remaining.iter().all(|e| !e.is_active));
    assert!(!remaining.iter().any(|e| e.id == prod.id));
}

#[tokio::test]
async fn environment_is_scoped_to_workspace() {
    let service = service().await;
    let env_a = service
        .create_environment("workspace-a".to_string(), "Shared".to_string())
        .await
        .expect("create in a");

    let wrong = service
        .activate_environment("workspace-b".to_string(), Some(env_a.id.clone()))
        .await;
    assert!(matches!(wrong, Err(AppError::NotFound(_))));

    let list_b = service
        .list_environments("workspace-b".to_string())
        .await
        .expect("list b");
    assert!(list_b.is_empty());
}

#[tokio::test]
async fn environment_names_are_unique_within_workspace() {
    let service = service().await;
    let dev = service
        .create_environment("workspace-a".to_string(), "Dev".to_string())
        .await
        .expect("create dev");
    let prod = service
        .create_environment("workspace-a".to_string(), "Prod".to_string())
        .await
        .expect("create prod");

    let duplicate_create = service
        .create_environment("workspace-a".to_string(), "dev".to_string())
        .await;
    assert!(matches!(
        duplicate_create,
        Err(AppError::Validation(message)) if message.contains("already exists")
    ));

    let same_name_other_workspace = service
        .create_environment("workspace-b".to_string(), "dev".to_string())
        .await
        .expect("same name in another workspace");
    assert_eq!(same_name_other_workspace.workspace_id, "workspace-b");

    let duplicate_update = service
        .update_environment(
            "workspace-a".to_string(),
            prod.id.clone(),
            "DEV".to_string(),
            vec![],
        )
        .await;
    assert!(matches!(
        duplicate_update,
        Err(AppError::Validation(message)) if message.contains("already exists")
    ));

    let own_name_update = service
        .update_environment("workspace-a".to_string(), dev.id, "dev".to_string(), vec![])
        .await
        .expect("same environment can keep its name");
    assert_eq!(own_name_update.name, "dev");
}

#[tokio::test]
async fn environment_update_rejects_duplicate_enabled_names() {
    let service = service().await;
    let env = service
        .create_environment("workspace-a".to_string(), "Dev".to_string())
        .await
        .expect("create");

    let result = service
        .update_environment(
            "workspace-a".to_string(),
            env.id,
            "Dev".to_string(),
            vec![
                KeyValue {
                    key: "token".to_string(),
                    value: "a".to_string(),
                    enabled: true,
                },
                KeyValue {
                    key: "TOKEN".to_string(),
                    value: "b".to_string(),
                    enabled: true,
                },
            ],
        )
        .await;
    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn resolve_input_applies_environment_across_request_parts() {
    let input = ApiRequestInput {
        workspace_id: "workspace-a".to_string(),
        name: Some("Templated".to_string()),
        parent_folder_id: None,
        collection_id: None,
        auth_json: None,
        method: "POST".to_string(),
        url: "{{base_url}}/users/{{user_id}}".to_string(),
        headers: vec![KeyValue {
            key: "X-Tenant".to_string(),
            value: "{{tenant}}".to_string(),
            enabled: true,
        }],
        query: vec![KeyValue {
            key: "source".to_string(),
            value: "{{source}}".to_string(),
            enabled: true,
        }],
        body: Some("{\"user\":\"{{user_id}}\"}".to_string()),
        body_kind: "json".to_string(),
        timeout_ms: None,
    };

    let resolved = resolve_input(
        input,
        &[
            KeyValue {
                key: "base_url".to_string(),
                value: "https://api.example.test".to_string(),
                enabled: true,
            },
            KeyValue {
                key: "user_id".to_string(),
                value: "42".to_string(),
                enabled: true,
            },
            KeyValue {
                key: "tenant".to_string(),
                value: "ops".to_string(),
                enabled: true,
            },
            KeyValue {
                key: "source".to_string(),
                value: "workspace".to_string(),
                enabled: true,
            },
        ],
    )
    .expect("resolve input");

    assert_eq!(resolved.url, "https://api.example.test/users/42");
    assert_eq!(resolved.headers[0].value, "ops");
    assert_eq!(resolved.query[0].value, "workspace");
    assert_eq!(resolved.body.as_deref(), Some("{\"user\":\"42\"}"));
}

#[tokio::test]
async fn resolve_input_reports_missing_environment_variable() {
    let input = ApiRequestInput {
        workspace_id: "workspace-a".to_string(),
        name: None,
        parent_folder_id: None,
        collection_id: None,
        auth_json: None,
        method: "GET".to_string(),
        url: "https://example.test/{{missing}}".to_string(),
        headers: vec![],
        query: vec![],
        body: None,
        body_kind: "json".to_string(),
        timeout_ms: None,
    };

    let result = resolve_input(input, &[]);

    assert!(matches!(result, Err(AppError::Validation(message)) if message.contains("missing")));
}

#[tokio::test]
async fn build_url_appends_enabled_query_pairs_only() {
    let url = build_url(
        "https://example.test/search?existing=true",
        &[
            KeyValue {
                key: "q".to_string(),
                value: "hello world".to_string(),
                enabled: true,
            },
            KeyValue {
                key: "disabled".to_string(),
                value: "ignored".to_string(),
                enabled: false,
            },
            KeyValue {
                key: "".to_string(),
                value: "ignored".to_string(),
                enabled: true,
            },
        ],
    )
    .expect("build url");

    assert_eq!(
        url.as_str(),
        "https://example.test/search?existing=true&q=hello+world"
    );
}
