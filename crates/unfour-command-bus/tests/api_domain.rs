use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqliteConnection;
use unfour_command_bus::{CommandBus, CommandBusExtensions, TransactionalCommandHook};
use unfour_core::domain::{
    ApiCollectionSnapshot, ApiFolderSnapshot, ApiRequestSnapshot, CommandContext, DomainEntityKey,
    DomainEntityType, DomainMutation, DomainSnapshot, ExternalApiCollectionApply,
    ExternalApiCollectionUpsert, ExternalApiFolderApply, ExternalApiFolderUpsert,
    ExternalApiRequestApply, ExternalApiRequestUpsert, ExternalApplyPage, ExternalDelete,
    ExternalWorkspaceApply, ExternalWorkspaceUpsert, MutationOrigin,
};
use unfour_core::models::{ApiRequestInput, KeyValue};
use unfour_core::{AppError, AppResult};
use unfour_local_storage::LocalDb;

#[derive(Clone)]
struct RecordingHook {
    local_only: bool,
    fail_on: Option<&'static str>,
}

impl TransactionalCommandHook for RecordingHook {
    fn on_mutations<'a>(
        &'a self,
        connection: &'a mut SqliteConnection,
        context: &'a CommandContext,
        mutations: &'a [DomainMutation],
    ) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(async move {
            if self.local_only && context.origin != MutationOrigin::Local {
                return Ok(());
            }
            for mutation in mutations {
                sqlx::query(
                    r#"
                    INSERT INTO api_hook_effects (
                      command_name, origin, entity_type, entity_id,
                      parent_entity_id, operation, revision
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    "#,
                )
                .bind(&context.command_name)
                .bind(format!("{:?}", context.origin))
                .bind(format!("{:?}", mutation.entity.entity_type))
                .bind(&mutation.entity.entity_id)
                .bind(&mutation.entity.parent_entity_id)
                .bind(format!("{:?}", mutation.operation))
                .bind(mutation.revision)
                .execute(&mut *connection)
                .await?;
            }
            if self.fail_on == Some(context.command_name.as_str()) {
                return Err(AppError::Config(format!(
                    "hook rejected {}",
                    context.command_name
                )));
            }
            Ok(())
        })
    }
}

async fn database() -> LocalDb {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect sqlite");
    let db = LocalDb::from_pool(pool);
    db.migrate().await.expect("migrate sqlite");
    db
}

async fn bus_with_hook(hook: RecordingHook) -> (CommandBus, LocalDb) {
    let db = database().await;
    CommandBus::from_db(db.clone())
        .await
        .expect("seed default workspace");
    sqlx::query(
        r#"
        CREATE TABLE api_hook_effects (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          command_name TEXT NOT NULL,
          origin TEXT NOT NULL,
          entity_type TEXT NOT NULL,
          entity_id TEXT NOT NULL,
          parent_entity_id TEXT,
          operation TEXT NOT NULL,
          revision INTEGER NOT NULL
        )
        "#,
    )
    .execute(db.pool())
    .await
    .expect("create hook table");
    let bus = CommandBus::from_db_with_extensions(
        db.clone(),
        CommandBusExtensions::new(vec![Arc::new(hook)]),
    )
    .await
    .expect("build hooked bus");
    (bus, db)
}

fn request_input(
    workspace_id: &str,
    collection_id: &str,
    parent_folder_id: Option<String>,
) -> ApiRequestInput {
    ApiRequestInput {
        workspace_id: workspace_id.to_string(),
        name: Some("Create user".to_string()),
        parent_folder_id,
        collection_id: Some(collection_id.to_string()),
        auth_json: Some(
            serde_json::json!({
                "type": "bearer",
                "token": "auth-device-secret",
                "prefix": "Bearer",
            })
            .to_string(),
        ),
        method: "post".to_string(),
        url: "https://api.example.test/users?access_token=url-device-secret&page=1".to_string(),
        headers: vec![
            KeyValue {
                key: "Authorization".to_string(),
                value: "Bearer header-device-secret".to_string(),
                enabled: true,
            },
            KeyValue {
                key: "Accept".to_string(),
                value: "application/json".to_string(),
                enabled: true,
            },
        ],
        query: vec![KeyValue {
            key: "api_key".to_string(),
            value: "query-device-secret".to_string(),
            enabled: true,
        }],
        body: Some(r#"{"name":"Ada","token":"body-device-secret"}"#.to_string()),
        body_kind: "json".to_string(),
        timeout_ms: Some(9_999),
        pre_request_script: Some("pm.variables.set('trace', '1');".to_string()),
        post_response_script: Some("pm.test('ok', () => true);".to_string()),
        script_schema_version: 1,
        temporary_variables: vec![KeyValue {
            key: "runtime_only".to_string(),
            value: "not-synced".to_string(),
            enabled: true,
        }],
    }
}

#[tokio::test]
async fn local_api_mutations_are_revisioned_noop_aware_and_hierarchical() {
    let (bus, db) = bus_with_hook(RecordingHook {
        local_only: false,
        fail_on: None,
    })
    .await;
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let collection = bus
        .api_collection_create(workspace_id.clone(), "Users".to_string())
        .await
        .unwrap();
    let initial_revision: i64 =
        sqlx::query_scalar("SELECT revision FROM api_collections WHERE id = ?1")
            .bind(&collection.id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(initial_revision, 1);

    bus.api_collection_rename(
        workspace_id.clone(),
        collection.id.clone(),
        "People".to_string(),
    )
    .await
    .unwrap();
    let hook_count_before_noop: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_hook_effects")
        .fetch_one(db.pool())
        .await
        .unwrap();
    bus.api_collection_rename(
        workspace_id.clone(),
        collection.id.clone(),
        "People".to_string(),
    )
    .await
    .unwrap();
    let revision_after_noop: i64 =
        sqlx::query_scalar("SELECT revision FROM api_collections WHERE id = ?1")
            .bind(&collection.id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    let hook_count_after_noop: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_hook_effects")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(revision_after_noop, 2);
    assert_eq!(hook_count_after_noop, hook_count_before_noop);

    let root = bus
        .api_collection_folder_create(
            workspace_id.clone(),
            collection.id.clone(),
            None,
            "Root".to_string(),
        )
        .await
        .unwrap();
    let child = bus
        .api_collection_folder_create(
            workspace_id.clone(),
            collection.id.clone(),
            Some(root.id.clone()),
            "Child".to_string(),
        )
        .await
        .unwrap();
    let child_revision = child.revision;
    let child = bus
        .api_collection_folder_rename(workspace_id.clone(), child.id, "Renamed Child".to_string())
        .await
        .unwrap();
    assert_eq!(child.revision, child_revision + 1);
    let child_noop = bus
        .api_collection_folder_rename(workspace_id.clone(), child.id.clone(), child.name.clone())
        .await
        .unwrap();
    assert_eq!(child_noop.revision, child.revision);
    let other_collection = bus
        .api_collection_create(workspace_id.clone(), "Other".to_string())
        .await
        .unwrap();
    let invalid_parent = bus
        .api_collection_folder_create(
            workspace_id.clone(),
            other_collection.id,
            Some(root.id.clone()),
            "Invalid".to_string(),
        )
        .await;
    assert!(matches!(invalid_parent, Err(AppError::Validation(_))));

    let input = request_input(&workspace_id, &collection.id, Some(child.id.clone()));
    let request = bus.save_api_request(input.clone()).await.unwrap();
    let saved_revision = request.revision;
    let saved_again = bus
        .update_api_request(workspace_id.clone(), request.id.clone(), input.clone())
        .await
        .unwrap();
    assert_eq!(saved_again.revision, saved_revision);
    let mut changed = input;
    changed.method = "PUT".to_string();
    changed.url = "https://api.example.test/users/1".to_string();
    changed.body = Some(r#"{"name":"Grace","token":"body-device-secret"}"#.to_string());
    let changed_request = bus
        .update_api_request(workspace_id.clone(), request.id.clone(), changed.clone())
        .await
        .unwrap();
    assert_eq!(changed_request.revision, saved_revision + 1);
    let changed_again = bus
        .update_api_request(workspace_id.clone(), request.id.clone(), changed)
        .await
        .unwrap();
    assert_eq!(changed_again.revision, changed_request.revision);

    let snapshot = bus
        .read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::ApiRequest,
            &workspace_id,
            &request.id,
        ))
        .await
        .unwrap();
    let DomainSnapshot::ApiRequest(snapshot) = snapshot else {
        panic!("expected API request snapshot");
    };
    assert_eq!(snapshot.method, "PUT");
    let snapshot_body: serde_json::Value =
        serde_json::from_str(snapshot.body.as_deref().unwrap()).unwrap();
    assert_eq!(snapshot_body["name"], "Grace");
    assert_eq!(snapshot_body["token"], "<redacted>");
    assert_eq!(
        snapshot.pre_request_script.as_deref(),
        Some("pm.variables.set('trace', '1');")
    );
    assert_eq!(
        snapshot.post_response_script.as_deref(),
        Some("pm.test('ok', () => true);")
    );
    let serialized = serde_json::to_string(&snapshot).unwrap();
    for excluded in [
        "auth-device-secret",
        "header-device-secret",
        "query-device-secret",
        "url-device-secret",
        "body-device-secret",
        "runtime_only",
        "not-synced",
        "timeoutMs",
    ] {
        assert!(!serialized.contains(excluded), "snapshot leaked {excluded}");
    }

    bus.api_collection_folder_delete(workspace_id.clone(), root.id.clone())
        .await
        .unwrap();
    for (table, id) in [
        ("api_collection_folders", root.id.as_str()),
        ("api_collection_folders", child.id.as_str()),
        ("api_requests", request.id.as_str()),
    ] {
        let query = format!("SELECT deleted_at FROM {table} WHERE id = ?1");
        let deleted_at: Option<String> = sqlx::query_scalar(&query)
            .bind(id)
            .fetch_one(db.pool())
            .await
            .unwrap();
        assert!(deleted_at.is_some(), "{table}:{id} must be tombstoned");
    }
    let delete_types: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT entity_type FROM api_hook_effects
        WHERE command_name = 'api.collection.folder.delete'
          AND operation = 'Delete'
        ORDER BY id
        "#,
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(
        delete_types,
        vec![
            "ApiRequest".to_string(),
            "ApiFolder".to_string(),
            "ApiFolder".to_string(),
        ]
    );
    let request_tombstone = bus
        .read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::ApiRequest,
            &workspace_id,
            &request.id,
        ))
        .await
        .unwrap();
    let DomainSnapshot::Tombstone(tombstone) = request_tombstone else {
        panic!("expected request tombstone");
    };
    assert_eq!(
        tombstone.entity.parent_entity_id.as_deref(),
        Some(child.id.as_str())
    );

    let direct_request = bus
        .save_api_request(request_input(&workspace_id, &collection.id, None))
        .await
        .unwrap();
    bus.delete_api_request(workspace_id.clone(), direct_request.id.clone())
        .await
        .unwrap();
    assert!(matches!(
        bus.read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::ApiRequest,
            &workspace_id,
            &direct_request.id,
        ))
        .await
        .unwrap(),
        DomainSnapshot::Tombstone(_)
    ));

    let remaining_folder = bus
        .api_collection_folder_create(
            workspace_id.clone(),
            collection.id.clone(),
            None,
            "Remaining".to_string(),
        )
        .await
        .unwrap();
    let remaining_request = bus
        .save_api_request(request_input(
            &workspace_id,
            &collection.id,
            Some(remaining_folder.id.clone()),
        ))
        .await
        .unwrap();
    bus.api_collection_delete(workspace_id.clone(), collection.id.clone())
        .await
        .unwrap();
    let collection_delete_types: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT entity_type FROM api_hook_effects
        WHERE command_name = 'api.collection.delete' AND operation = 'Delete'
        ORDER BY id
        "#,
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(
        collection_delete_types,
        vec![
            "ApiRequest".to_string(),
            "ApiFolder".to_string(),
            "ApiCollection".to_string(),
        ]
    );
    for (entity_type, id) in [
        (DomainEntityType::ApiCollection, collection.id.as_str()),
        (DomainEntityType::ApiFolder, remaining_folder.id.as_str()),
        (DomainEntityType::ApiRequest, remaining_request.id.as_str()),
    ] {
        assert!(matches!(
            bus.read_domain_snapshot(&DomainEntityKey::new(entity_type, &workspace_id, id,))
                .await
                .unwrap(),
            DomainSnapshot::Tombstone(_)
        ));
    }
}

#[tokio::test]
async fn snapshot_external_apply_round_trip_preserves_tree_without_secrets_or_echo() {
    let source_db = database().await;
    let source = CommandBus::from_db(source_db.clone()).await.unwrap();
    let workspace_id = source.list_workspaces().await.unwrap().active_workspace_id;
    source
        .rename_workspace(workspace_id.clone(), "Source Workspace".to_string())
        .await
        .unwrap();
    let workspace = source
        .list_workspaces()
        .await
        .unwrap()
        .workspaces
        .into_iter()
        .find(|workspace| workspace.id == workspace_id)
        .unwrap();
    let collection = source
        .api_collection_create(workspace_id.clone(), "Commerce".to_string())
        .await
        .unwrap();
    let parent = source
        .api_collection_folder_create(
            workspace_id.clone(),
            collection.id.clone(),
            None,
            "Orders".to_string(),
        )
        .await
        .unwrap();
    let child = source
        .api_collection_folder_create(
            workspace_id.clone(),
            collection.id.clone(),
            Some(parent.id.clone()),
            "Refunds".to_string(),
        )
        .await
        .unwrap();
    let request = source
        .save_api_request(request_input(
            &workspace_id,
            &collection.id,
            Some(child.id.clone()),
        ))
        .await
        .unwrap();

    let collection_snapshot = api_collection_snapshot(&source, &workspace_id, &collection.id).await;
    let parent_snapshot = api_folder_snapshot(&source, &workspace_id, &parent.id).await;
    let child_snapshot = api_folder_snapshot(&source, &workspace_id, &child.id).await;
    let request_snapshot = api_request_snapshot(&source, &workspace_id, &request.id).await;

    let (target, target_db) = bus_with_hook(RecordingHook {
        local_only: true,
        fail_on: None,
    })
    .await;
    sqlx::query("DELETE FROM api_hook_effects")
        .execute(target_db.pool())
        .await
        .unwrap();
    let page = ExternalApplyPage {
        workspaces: vec![ExternalWorkspaceApply::Upsert(ExternalWorkspaceUpsert {
            id: workspace.id.clone(),
            name: workspace.name,
            environment_type: workspace.environment_type,
            mcp_policy: workspace.mcp_policy,
            created_at: workspace.created_at,
            updated_at: workspace.updated_at,
        })],
        api_collections: vec![collection_apply(&collection_snapshot)],
        // Intentionally child-before-parent: Core must topologically apply folders.
        api_folders: vec![
            folder_apply(&child_snapshot),
            folder_apply(&parent_snapshot),
        ],
        api_requests: vec![request_apply(&request_snapshot)],
        ..ExternalApplyPage::default()
    };
    let first = target.apply_external_page(page.clone()).await.unwrap();
    assert_eq!(first.applied_count, 5);
    assert!(first
        .mutations
        .iter()
        .all(|mutation| mutation.origin == MutationOrigin::External));
    let echo_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_hook_effects")
        .fetch_one(target_db.pool())
        .await
        .unwrap();
    assert_eq!(echo_count, 0);
    let second = target.apply_external_page(page).await.unwrap();
    assert_eq!(second.applied_count, 0, "external upsert must be a no-op");

    let mut target_collection =
        api_collection_snapshot(&target, &workspace_id, &collection.id).await;
    let mut target_parent = api_folder_snapshot(&target, &workspace_id, &parent.id).await;
    let mut target_child = api_folder_snapshot(&target, &workspace_id, &child.id).await;
    let mut target_request = api_request_snapshot(&target, &workspace_id, &request.id).await;
    let mut source_collection = collection_snapshot.clone();
    let mut source_parent = parent_snapshot.clone();
    let mut source_child = child_snapshot.clone();
    let mut source_request = request_snapshot.clone();
    source_collection.revision = 0;
    target_collection.revision = 0;
    source_parent.revision = 0;
    target_parent.revision = 0;
    source_child.revision = 0;
    target_child.revision = 0;
    source_request.revision = 0;
    target_request.revision = 0;
    assert_eq!(target_collection, source_collection);
    assert_eq!(target_parent, source_parent);
    assert_eq!(target_child, source_child);
    assert_eq!(target_request, source_request);

    let stored: (String, String, String, Option<String>, String) = sqlx::query_as(
        r#"
        SELECT auth_json, headers_json, query_json, body, url
        FROM api_requests WHERE id = ?1
        "#,
    )
    .bind(&request.id)
    .fetch_one(target_db.pool())
    .await
    .unwrap();
    let stored_serialized = serde_json::to_string(&stored).unwrap();
    for forbidden in [
        "auth-device-secret",
        "header-device-secret",
        "query-device-secret",
        "body-device-secret",
        "url-device-secret",
        "<redacted>",
    ] {
        assert!(
            !stored_serialized.contains(forbidden),
            "external apply stored secret marker/material: {forbidden}"
        );
    }

    let folder_delete = ExternalApiFolderApply::Delete(ExternalDelete {
        entity: DomainEntityKey::new(DomainEntityType::ApiFolder, &workspace_id, &parent.id)
            .with_parent_entity_id(&collection.id),
        deleted_at: "2026-08-12T12:00:00Z".to_string(),
    });
    let deleted = target
        .apply_external_page(ExternalApplyPage {
            api_folders: vec![folder_delete.clone()],
            ..ExternalApplyPage::default()
        })
        .await
        .unwrap();
    assert_eq!(deleted.applied_count, 3);
    assert!(matches!(
        target
            .read_domain_snapshot(&DomainEntityKey::new(
                DomainEntityType::ApiRequest,
                &workspace_id,
                &request.id,
            ))
            .await
            .unwrap(),
        DomainSnapshot::Tombstone(_)
    ));
    let deleted_again = target
        .apply_external_page(ExternalApplyPage {
            api_folders: vec![folder_delete],
            ..ExternalApplyPage::default()
        })
        .await
        .unwrap();
    assert_eq!(deleted_again.applied_count, 0);

    let collection_delete = ExternalApiCollectionApply::Delete(ExternalDelete {
        entity: DomainEntityKey::new(
            DomainEntityType::ApiCollection,
            &workspace_id,
            &collection.id,
        ),
        deleted_at: "2026-08-12T12:01:00Z".to_string(),
    });
    assert_eq!(
        target
            .apply_external_page(ExternalApplyPage {
                api_collections: vec![collection_delete.clone()],
                ..ExternalApplyPage::default()
            })
            .await
            .unwrap()
            .applied_count,
        1
    );
    assert_eq!(
        target
            .apply_external_page(ExternalApplyPage {
                api_collections: vec![collection_delete],
                ..ExternalApplyPage::default()
            })
            .await
            .unwrap()
            .applied_count,
        0
    );
}

#[tokio::test]
async fn api_hook_failure_rolls_back_business_row_activity_and_hook_effects() {
    let (bus, db) = bus_with_hook(RecordingHook {
        local_only: false,
        fail_on: Some("api.collection.create"),
    })
    .await;
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let error = bus
        .api_collection_create(workspace_id, "Rollback".to_string())
        .await
        .expect_err("hook must reject API collection creation");
    assert!(error.to_string().contains("hook rejected"));
    let collection_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_collections")
        .fetch_one(db.pool())
        .await
        .unwrap();
    let activity_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM activity_events WHERE action = 'api.collection.create'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    let hook_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_hook_effects")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(collection_count, 0);
    assert_eq!(activity_count, 0);
    assert_eq!(hook_count, 0);
}

#[tokio::test]
async fn external_apply_rejects_missing_or_cross_collection_parents_atomically() {
    let db = database().await;
    let bus = CommandBus::from_db(db.clone()).await.unwrap();
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let collection = bus
        .api_collection_create(workspace_id.clone(), "Valid".to_string())
        .await
        .unwrap();
    let error = bus
        .apply_external_page(ExternalApplyPage {
            api_folders: vec![ExternalApiFolderApply::Upsert(ExternalApiFolderUpsert {
                id: "external-folder".to_string(),
                workspace_id: workspace_id.clone(),
                collection_id: collection.id,
                parent_folder_id: Some("missing-parent".to_string()),
                name: "Orphan".to_string(),
                sort_order: 0,
                created_at: "2026-08-12T00:00:00Z".to_string(),
                updated_at: "2026-08-12T00:00:00Z".to_string(),
            })],
            ..ExternalApplyPage::default()
        })
        .await
        .expect_err("missing external parent must be rejected");
    assert!(matches!(error, AppError::NotFound(_)));
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_collection_folders WHERE id = 'external-folder'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn api_save_and_import_record_primary_activity_targets() {
    let (bus, db) = bus_with_hook(RecordingHook {
        local_only: false,
        fail_on: None,
    })
    .await;
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;

    let request = bus
        .save_api_request(ApiRequestInput {
            workspace_id: workspace_id.clone(),
            name: None,
            parent_folder_id: None,
            collection_id: None,
            auth_json: None,
            method: "get".to_string(),
            url: "https://api.example.test/ping".to_string(),
            headers: vec![],
            query: vec![],
            body: None,
            body_kind: "json".to_string(),
            timeout_ms: None,
            pre_request_script: None,
            post_response_script: None,
            script_schema_version: 1,
            temporary_variables: vec![],
        })
        .await
        .unwrap();
    let collection_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_collections WHERE workspace_id = ?1 AND deleted_at IS NULL",
    )
    .bind(&workspace_id)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(
        collection_count, 1,
        "save must auto-create a default collection"
    );

    let save_hook_types: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT entity_type FROM api_hook_effects
        WHERE command_name = 'api.save_request'
        ORDER BY id
        "#,
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(
        save_hook_types,
        vec!["ApiCollection".to_string(), "ApiRequest".to_string()]
    );

    let (save_target, save_details): (Option<String>, String) = sqlx::query_as(
        r#"
        SELECT target, details_json FROM activity_events
        WHERE action = 'api.save_request'
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(save_target.as_deref(), Some(request.id.as_str()));
    let save_details: serde_json::Value = serde_json::from_str(&save_details).unwrap();
    assert_eq!(save_details["name"].as_str(), Some(request.name.as_str()));
    assert_eq!(save_details["method"], "GET");

    let duplicate = bus
        .duplicate_api_request(workspace_id.clone(), request.id.clone())
        .await
        .unwrap();
    let (duplicate_target, duplicate_details): (Option<String>, String) = sqlx::query_as(
        r#"
        SELECT target, details_json FROM activity_events
        WHERE action = 'api.duplicate_request'
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(duplicate_target.as_deref(), Some(duplicate.id.as_str()));
    let duplicate_details: serde_json::Value = serde_json::from_str(&duplicate_details).unwrap();
    assert_eq!(
        duplicate_details["sourceId"].as_str(),
        Some(request.id.as_str())
    );
    assert_eq!(
        duplicate_details["name"].as_str(),
        Some(duplicate.name.as_str())
    );

    let openapi = r#"{
      "openapi":"3.0.3",
      "info":{"title":"Imported API","version":"1"},
      "servers":[{"url":"https://api.example.test"}],
      "paths":{"/users":{"get":{"operationId":"listUsers","tags":["Users"]}}}
    }"#;
    let imported = bus
        .api_collection_import(workspace_id.clone(), openapi.to_string())
        .await
        .unwrap();
    assert!(imported.imported);
    assert_eq!(imported.folder_count, 1);
    assert_eq!(imported.request_count, 1);
    let collection = imported.collection.expect("imported collection");

    let (import_target, import_details): (Option<String>, String) = sqlx::query_as(
        r#"
        SELECT target, details_json FROM activity_events
        WHERE action = 'api.collection.import'
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(import_target.as_deref(), Some(collection.id.as_str()));
    let import_details: serde_json::Value = serde_json::from_str(&import_details).unwrap();
    assert_eq!(import_details["folderCount"], 1);
    assert_eq!(import_details["requestCount"], 1);
    assert_eq!(import_details["contentBytes"], openapi.len());
}

async fn api_collection_snapshot(
    bus: &CommandBus,
    workspace_id: &str,
    id: &str,
) -> ApiCollectionSnapshot {
    let snapshot = bus
        .read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::ApiCollection,
            workspace_id,
            id,
        ))
        .await
        .unwrap();
    let DomainSnapshot::ApiCollection(snapshot) = snapshot else {
        panic!("expected collection snapshot");
    };
    snapshot
}

async fn api_folder_snapshot(bus: &CommandBus, workspace_id: &str, id: &str) -> ApiFolderSnapshot {
    let snapshot = bus
        .read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::ApiFolder,
            workspace_id,
            id,
        ))
        .await
        .unwrap();
    let DomainSnapshot::ApiFolder(snapshot) = snapshot else {
        panic!("expected folder snapshot");
    };
    snapshot
}

async fn api_request_snapshot(
    bus: &CommandBus,
    workspace_id: &str,
    id: &str,
) -> ApiRequestSnapshot {
    let snapshot = bus
        .read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::ApiRequest,
            workspace_id,
            id,
        ))
        .await
        .unwrap();
    let DomainSnapshot::ApiRequest(snapshot) = snapshot else {
        panic!("expected request snapshot");
    };
    snapshot
}

fn collection_apply(snapshot: &ApiCollectionSnapshot) -> ExternalApiCollectionApply {
    ExternalApiCollectionApply::Upsert(ExternalApiCollectionUpsert {
        id: snapshot.id.clone(),
        workspace_id: snapshot.workspace_id.clone(),
        name: snapshot.name.clone(),
        description: snapshot.description.clone(),
        created_at: snapshot.created_at.clone(),
        updated_at: snapshot.updated_at.clone(),
    })
}

fn folder_apply(snapshot: &ApiFolderSnapshot) -> ExternalApiFolderApply {
    ExternalApiFolderApply::Upsert(ExternalApiFolderUpsert {
        id: snapshot.id.clone(),
        workspace_id: snapshot.workspace_id.clone(),
        collection_id: snapshot.collection_id.clone(),
        parent_folder_id: snapshot.parent_folder_id.clone(),
        name: snapshot.name.clone(),
        sort_order: snapshot.sort_order,
        created_at: snapshot.created_at.clone(),
        updated_at: snapshot.updated_at.clone(),
    })
}

fn request_apply(snapshot: &ApiRequestSnapshot) -> ExternalApiRequestApply {
    ExternalApiRequestApply::Upsert(Box::new(ExternalApiRequestUpsert {
        id: snapshot.id.clone(),
        workspace_id: snapshot.workspace_id.clone(),
        collection_id: snapshot.collection_id.clone(),
        parent_folder_id: snapshot.parent_folder_id.clone(),
        name: snapshot.name.clone(),
        sort_order: snapshot.sort_order,
        auth_json: snapshot.auth_json.clone(),
        method: snapshot.method.clone(),
        url: snapshot.url.clone(),
        headers: snapshot.headers.clone(),
        query: snapshot.query.clone(),
        body: snapshot.body.clone(),
        body_kind: snapshot.body_kind.clone(),
        pre_request_script: snapshot.pre_request_script.clone(),
        post_response_script: snapshot.post_response_script.clone(),
        script_schema_version: snapshot.script_schema_version,
        created_at: snapshot.created_at.clone(),
        updated_at: snapshot.updated_at.clone(),
    }))
}
