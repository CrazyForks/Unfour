use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqliteConnection;
use unfour_command_bus::{CommandBus, CommandBusExtensions, TransactionalCommandHook};
use unfour_core::domain::{
    CommandContext, DomainEntityKey, DomainEntityType, DomainMutation, DomainSnapshot,
    ExternalApplyPage, ExternalDelete, ExternalVariableValue, ExternalWorkspaceApply,
    ExternalWorkspaceEnvironmentApply, ExternalWorkspaceUpsert, ExternalWorkspaceVariableApply,
    ExternalWorkspaceVariableUpsert, MutationOrigin, SnapshotVariableValue,
};
use unfour_core::models::WorkspaceVariableInput;
use unfour_core::{AppError, AppResult};
use unfour_local_storage::LocalDb;

#[path = "workspace_domain/external_apply.rs"]
mod external_apply;

#[derive(Clone)]
struct SqlHook {
    name: &'static str,
    fail_on: Option<&'static str>,
    local_only: bool,
}

impl TransactionalCommandHook for SqlHook {
    fn on_mutations<'a>(
        &'a self,
        connection: &'a mut SqliteConnection,
        context: &'a CommandContext,
        mutations: &'a [DomainMutation],
    ) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(async move {
            if mutations.is_empty() || (self.local_only && context.origin != MutationOrigin::Local)
            {
                return Ok(());
            }
            for mutation in mutations {
                sqlx::query(
                    r#"
                    INSERT INTO hook_effects (
                      hook_name, command_name, origin, entity_type,
                      entity_id, operation, revision
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    "#,
                )
                .bind(self.name)
                .bind(&context.command_name)
                .bind(format!("{:?}", context.origin))
                .bind(format!("{:?}", mutation.entity.entity_type))
                .bind(&mutation.entity.entity_id)
                .bind(format!("{:?}", mutation.operation))
                .bind(mutation.revision)
                .execute(&mut *connection)
                .await?;
            }
            if self.fail_on == Some(context.command_name.as_str()) {
                return Err(AppError::Config(format!(
                    "{} rejected {}",
                    self.name, context.command_name
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

async fn bus_with_hooks(hooks: Vec<Arc<dyn TransactionalCommandHook>>) -> (CommandBus, LocalDb) {
    let db = database().await;
    CommandBus::from_db(db.clone())
        .await
        .expect("seed default workspace");
    sqlx::query(
        r#"
        CREATE TABLE hook_effects (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          hook_name TEXT NOT NULL,
          command_name TEXT NOT NULL,
          origin TEXT NOT NULL,
          entity_type TEXT NOT NULL,
          entity_id TEXT NOT NULL,
          operation TEXT NOT NULL,
          revision INTEGER NOT NULL
        )
        "#,
    )
    .execute(db.pool())
    .await
    .expect("create hook table");
    let bus = CommandBus::from_db_with_extensions(db.clone(), CommandBusExtensions::new(hooks))
        .await
        .expect("build hooked bus");
    (bus, db)
}

fn input(id: Option<String>, key: &str, value: &str, secret: bool) -> WorkspaceVariableInput {
    WorkspaceVariableInput {
        id,
        key: key.to_string(),
        value: value.to_string(),
        is_secret: secret,
        is_enabled: true,
        description: None,
        sort_order: 0,
    }
}

#[tokio::test]
async fn hook_failure_rolls_back_domain_activity_and_hook_sql() {
    let hook = Arc::new(SqlHook {
        name: "rejecting",
        fail_on: Some("workspace.create"),
        local_only: false,
    });
    let (bus, db) = bus_with_hooks(vec![hook]).await;
    let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workspaces")
        .fetch_one(db.pool())
        .await
        .unwrap();

    let error = bus
        .create_workspace("Must Roll Back".to_string())
        .await
        .expect_err("hook should reject command");
    assert!(error.to_string().contains("rejecting"));

    let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workspaces")
        .fetch_one(db.pool())
        .await
        .unwrap();
    let activity: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM activity_events WHERE action = 'workspace.create'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    let hook_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hook_effects")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(after, before);
    assert_eq!(activity, 0);
    assert_eq!(hook_rows, 0);
}

#[tokio::test]
async fn later_hook_failure_rolls_back_earlier_hook_sql() {
    let first = Arc::new(SqlHook {
        name: "first",
        fail_on: None,
        local_only: false,
    });
    let second = Arc::new(SqlHook {
        name: "second",
        fail_on: Some("workspace.create"),
        local_only: false,
    });
    let (bus, db) = bus_with_hooks(vec![first, second]).await;

    bus.create_workspace("Rollback Both Hooks".to_string())
        .await
        .expect_err("second hook should reject command");
    let hook_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hook_effects")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(hook_rows, 0);
}

#[tokio::test]
async fn community_without_hooks_keeps_normal_behavior_and_activity() {
    let db = database().await;
    let bus = CommandBus::from_db(db.clone()).await.unwrap();
    let workspace = bus
        .create_workspace("Community".to_string())
        .await
        .expect("create without hooks");
    assert_eq!(workspace.name, "Community");
    let activity: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT workspace_id, target FROM activity_events WHERE action = 'workspace.create'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(activity.0.as_deref(), Some(workspace.id.as_str()));
    assert_eq!(activity.1.as_deref(), Some(workspace.id.as_str()));
}

#[tokio::test]
async fn entity_create_activities_include_generated_targets() {
    let db = database().await;
    let bus = CommandBus::from_db(db.clone()).await.unwrap();
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let variable = bus
        .workspace_variable_create(
            workspace_id.clone(),
            input(None, "BASE_URL", "https://example.test", false),
        )
        .await
        .unwrap();
    let environment = bus
        .workspace_environment_create(workspace_id.clone(), "Development".to_string())
        .await
        .unwrap();
    let environment_variable = bus
        .workspace_environment_variable_create(
            workspace_id.clone(),
            environment.id.clone(),
            input(None, "TOKEN", "secret", true),
        )
        .await
        .unwrap();

    for (action, expected_target) in [
        ("workspace.variable.create", variable.id),
        ("workspace.environment.create", environment.id),
        (
            "workspace.environment_variable.create",
            environment_variable.id,
        ),
    ] {
        let activity: (Option<String>, Option<String>) =
            sqlx::query_as("SELECT workspace_id, target FROM activity_events WHERE action = ?1")
                .bind(action)
                .fetch_one(db.pool())
                .await
                .unwrap();
        assert_eq!(activity.0.as_deref(), Some(workspace_id.as_str()));
        assert_eq!(activity.1.as_deref(), Some(expected_target.as_str()));
    }
}

#[tokio::test]
async fn workspace_fields_default_mutations_and_snapshot_are_precise() {
    let capture = Arc::new(SqlHook {
        name: "capture",
        fail_on: None,
        local_only: false,
    });
    let (bus, db) = bus_with_hooks(vec![capture]).await;
    let original_default = bus.list_workspaces().await.unwrap().active_workspace_id;
    let created = bus.create_workspace("Mutable".to_string()).await.unwrap();
    let revision = created.revision;
    let unchanged = bus
        .rename_workspace(created.id.clone(), created.name.clone())
        .await
        .unwrap();
    assert_eq!(unchanged.revision, revision);
    let no_op_mutations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hook_effects WHERE command_name = 'workspace.rename'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(no_op_mutations, 0);
    bus.rename_workspace(created.id.clone(), "Renamed".to_string())
        .await
        .unwrap();
    bus.update_workspace_environment(created.id.clone(), "prod".to_string())
        .await
        .unwrap();
    bus.update_workspace_mcp_policy(created.id.clone(), "read_only".to_string())
        .await
        .unwrap();
    bus.set_default_workspace(created.id.clone()).await.unwrap();
    bus.set_active_workspace(created.id.clone()).await.unwrap();

    let default_mutations: Vec<(String,)> = sqlx::query_as(
        "SELECT entity_id FROM hook_effects WHERE command_name = 'workspace.default.set' ORDER BY entity_id",
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(default_mutations.len(), 2);
    assert!(default_mutations
        .iter()
        .any(|row| row.0 == original_default));
    assert!(default_mutations.iter().any(|row| row.0 == created.id));

    let snapshot = bus
        .read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::Workspace,
            &created.id,
            &created.id,
        ))
        .await
        .unwrap();
    let DomainSnapshot::Workspace(snapshot) = snapshot else {
        panic!("expected workspace snapshot");
    };
    assert_eq!(snapshot.id, created.id);
    assert_eq!(snapshot.name, "Renamed");
    assert!(snapshot.is_default);
    assert_eq!(snapshot.environment_type, "prod");
    assert_eq!(snapshot.mcp_policy, "read_only");
    assert!(snapshot.last_opened_at.is_some());
    assert!(!snapshot.created_at.is_empty());
    assert!(!snapshot.updated_at.is_empty());
    assert!(snapshot.revision > revision);
}

#[tokio::test]
async fn variable_replace_reports_only_real_diff_and_secret_snapshot_is_redacted() {
    let capture = Arc::new(SqlHook {
        name: "capture",
        fail_on: None,
        local_only: false,
    });
    let (bus, db) = bus_with_hooks(vec![capture]).await;
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let initial = bus
        .workspace_variables_replace(
            workspace_id.clone(),
            vec![
                input(None, "PLAIN", "one", false),
                input(None, "TOKEN", "top-secret", true),
            ],
        )
        .await
        .unwrap();
    sqlx::query("DELETE FROM hook_effects")
        .execute(db.pool())
        .await
        .unwrap();
    let updated = bus
        .workspace_variables_replace(
            workspace_id.clone(),
            vec![
                input(Some(initial[0].id.clone()), "PLAIN", "two", false),
                input(Some(initial[1].id.clone()), "TOKEN", "top-secret", true),
            ],
        )
        .await
        .unwrap();
    let mutations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hook_effects WHERE command_name = 'workspace.variables.replace'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(mutations, 1);

    let snapshot = bus
        .read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::WorkspaceVariable,
            &workspace_id,
            &updated[1].id,
        ))
        .await
        .unwrap();
    let DomainSnapshot::WorkspaceVariable(snapshot) = snapshot else {
        panic!("expected variable snapshot");
    };
    assert_eq!(snapshot.value, SnapshotVariableValue::SecretRedacted);
    let serialized = serde_json::to_string(&snapshot).unwrap();
    let debug = format!("{snapshot:?}");
    assert!(!serialized.contains("top-secret"));
    assert!(!debug.contains("top-secret"));

    bus.workspace_variable_delete(workspace_id.clone(), updated[1].id.clone())
        .await
        .unwrap();
    assert!(matches!(
        bus.read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::WorkspaceVariable,
            workspace_id,
            updated[1].id.clone(),
        ))
        .await
        .unwrap(),
        DomainSnapshot::Tombstone(_)
    ));
}

#[tokio::test]
async fn environment_selection_has_no_mutation_and_delete_is_atomic_with_children() {
    let hook = Arc::new(SqlHook {
        name: "conditional",
        fail_on: Some("workspace.environment.delete"),
        local_only: false,
    });
    let (bus, db) = bus_with_hooks(vec![hook]).await;
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let environment = bus
        .workspace_environment_create(workspace_id.clone(), "Development".to_string())
        .await
        .unwrap();
    let environment = bus
        .workspace_environment_update(
            workspace_id.clone(),
            environment.id.clone(),
            environment.name,
            vec![input(None, "HOST", "localhost", false)],
        )
        .await
        .unwrap();
    sqlx::query("DELETE FROM hook_effects")
        .execute(db.pool())
        .await
        .unwrap();
    bus.workspace_environment_set_active(workspace_id.clone(), None)
        .await
        .unwrap();
    let selection_mutations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hook_effects")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(selection_mutations, 0);

    bus.workspace_environment_delete(workspace_id.clone(), environment.id.clone())
        .await
        .expect_err("hook should roll back environment deletion");
    let environment_deleted: Option<String> =
        sqlx::query_scalar("SELECT deleted_at FROM workspace_environments WHERE id = ?1")
            .bind(&environment.id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    let child_deleted: Option<String> =
        sqlx::query_scalar("SELECT deleted_at FROM workspace_environment_variables WHERE id = ?1")
            .bind(&environment.variables[0].id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert!(environment_deleted.is_none());
    assert!(child_deleted.is_none());
}

#[tokio::test]
async fn external_apply_updates_revision_preserves_local_secret_and_creates_no_echo_rows() {
    let outbox = Arc::new(SqlHook {
        name: "local-outbox",
        fail_on: None,
        local_only: true,
    });
    let (bus, db) = bus_with_hooks(vec![outbox]).await;
    let state = bus.list_workspaces().await.unwrap();
    let workspace_id = state.active_workspace_id.clone();
    let secret = bus
        .workspace_variable_create(
            workspace_id.clone(),
            input(None, "TOKEN", "device-secret", true),
        )
        .await
        .unwrap();
    sqlx::query("DELETE FROM hook_effects")
        .execute(db.pool())
        .await
        .unwrap();
    let before_revision = secret.revision;
    let now = "2026-07-23T09:15:37Z".to_string();
    let applied = bus
        .apply_external_page(ExternalApplyPage {
            workspace_variables: vec![ExternalWorkspaceVariableApply::Upsert(
                ExternalWorkspaceVariableUpsert {
                    id: secret.id.clone(),
                    workspace_id: workspace_id.clone(),
                    key: "TOKEN".to_string(),
                    value: ExternalVariableValue::PreserveLocal,
                    is_secret: true,
                    is_enabled: false,
                    description: Some("external metadata".to_string()),
                    sort_order: 0,
                    created_at: secret.created_at.clone(),
                    updated_at: now,
                },
            )],
            ..ExternalApplyPage::default()
        })
        .await
        .unwrap();
    assert_eq!(applied.applied_count, 1);
    assert_eq!(applied.secret_material_outcomes.len(), 1);
    assert_eq!(
        applied.secret_material_outcomes[0].status,
        unfour_core::domain::SecretMaterialStatus::Present
    );
    let echo_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hook_effects")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(echo_rows, 0);
    let stored: (String, i64, bool) =
        sqlx::query_as("SELECT value, revision, is_enabled FROM workspace_variables WHERE id = ?1")
            .bind(&secret.id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(stored.0, "device-secret");
    assert_eq!(stored.1, before_revision + 1);
    assert!(!stored.2);

    let active_before = bus.list_workspaces().await.unwrap().active_workspace_id;
    bus.apply_external_workspaces(vec![ExternalWorkspaceApply::Upsert(
        ExternalWorkspaceUpsert {
            id: "external-workspace".to_string(),
            name: "External".to_string(),
            is_default: false,
            last_opened_at: None,
            environment_type: "test".to_string(),
            mcp_policy: "guarded".to_string(),
            created_at: "2026-07-23T09:15:38Z".to_string(),
            updated_at: "2026-07-23T09:15:38Z".to_string(),
        },
    )])
    .await
    .unwrap();
    assert_eq!(
        bus.list_workspaces().await.unwrap().active_workspace_id,
        active_before
    );
}

#[tokio::test]
async fn external_workspace_changes_preserve_local_workspace_invariants() {
    let db = database().await;
    let bus = CommandBus::from_db(db.clone()).await.unwrap();
    let original = bus.list_workspaces().await.unwrap().active_workspace_id;
    let active = bus
        .create_workspace("External Delete Target".to_string())
        .await
        .unwrap();
    let deleted_at = "2026-07-24T00:00:00Z".to_string();

    bus.apply_external_workspaces(vec![ExternalWorkspaceApply::Delete(ExternalDelete {
        entity: DomainEntityKey::new(DomainEntityType::Workspace, &active.id, &active.id),
        deleted_at: deleted_at.clone(),
    })])
    .await
    .expect("delete active workspace externally");

    let stored_active: String =
        sqlx::query_scalar("SELECT value FROM app_settings WHERE key = 'active_workspace_id'")
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(stored_active, original);
    let report = bus
        .apply_external_workspaces(vec![ExternalWorkspaceApply::Delete(ExternalDelete {
            entity: DomainEntityKey::new(DomainEntityType::Workspace, &original, &original),
            deleted_at,
        })])
        .await
        .expect("external apply should replace the last workspace with a local fallback");
    assert_eq!(report.applied_count, 2);
    assert_eq!(report.mutations.len(), 2);
    let original_tombstone: Option<String> =
        sqlx::query_scalar("SELECT deleted_at FROM workspaces WHERE id = ?1")
            .bind(&original)
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert!(original_tombstone.is_some());
    let fallback: (String, String, bool, String, String) = sqlx::query_as(
        r#"
        SELECT id, name, is_default, environment_type, mcp_policy
        FROM workspaces WHERE deleted_at IS NULL
        "#,
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_ne!(fallback.0, original);
    assert_eq!(fallback.1, "Default Workspace");
    assert!(fallback.2);
    assert_eq!(fallback.3, "dev");
    assert_eq!(fallback.4, "auto");
    let fallback_active: String =
        sqlx::query_scalar("SELECT value FROM app_settings WHERE key = 'active_workspace_id'")
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(fallback_active, fallback.0);
    let companions: (i64, i64) = sqlx::query_as(
        r#"
        SELECT
          (SELECT COUNT(*) FROM workspace_settings WHERE workspace_id = ?1),
          (SELECT COUNT(*) FROM workspace_local_state WHERE workspace_id = ?1)
        "#,
    )
    .bind(&fallback.0)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(companions, (1, 1));

    let now = "2026-07-24T00:01:00Z".to_string();
    bus.apply_external_workspaces(vec![ExternalWorkspaceApply::Upsert(
        ExternalWorkspaceUpsert {
            id: "external-default".to_string(),
            name: "External Default".to_string(),
            is_default: true,
            last_opened_at: None,
            environment_type: "dev".to_string(),
            mcp_policy: "auto".to_string(),
            created_at: now.clone(),
            updated_at: now,
        },
    )])
    .await
    .expect("apply new default workspace");
    let defaults: Vec<(String,)> =
        sqlx::query_as("SELECT id FROM workspaces WHERE is_default = 1 AND deleted_at IS NULL")
            .fetch_all(db.pool())
            .await
            .unwrap();
    assert_eq!(defaults, vec![("external-default".to_string(),)]);
}

#[tokio::test]
async fn external_last_workspace_fallback_has_no_local_echo() {
    let outbox = Arc::new(SqlHook {
        name: "local-outbox",
        fail_on: None,
        local_only: true,
    });
    let (bus, db) = bus_with_hooks(vec![outbox]).await;
    let original = bus.list_workspaces().await.unwrap().active_workspace_id;
    bus.workspace_variable_create(
        original.clone(),
        input(None, "ORIGINAL_ONLY", "value", false),
    )
    .await
    .unwrap();
    bus.workspace_environment_create(original.clone(), "Original Environment".to_string())
        .await
        .unwrap();
    sqlx::query("DELETE FROM hook_effects")
        .execute(db.pool())
        .await
        .unwrap();

    let report = bus
        .apply_external_workspaces(vec![ExternalWorkspaceApply::Delete(ExternalDelete {
            entity: DomainEntityKey::new(DomainEntityType::Workspace, &original, &original),
            deleted_at: "2026-07-24T00:01:30Z".to_string(),
        })])
        .await
        .unwrap();

    assert_eq!(report.mutations.len(), 2);
    assert!(report
        .mutations
        .iter()
        .all(|mutation| mutation.origin == MutationOrigin::External));
    let fallback_id = report
        .mutations
        .iter()
        .find(|mutation| mutation.operation == unfour_core::domain::MutationOperation::Upsert)
        .unwrap()
        .entity
        .entity_id
        .clone();
    let inherited: (i64, i64) = sqlx::query_as(
        r#"
        SELECT
          (SELECT COUNT(*) FROM workspace_variables WHERE workspace_id = ?1),
          (SELECT COUNT(*) FROM workspace_environments WHERE workspace_id = ?1)
        "#,
    )
    .bind(&fallback_id)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(inherited, (0, 0));
    let echo_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hook_effects")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(echo_rows, 0);
}

#[tokio::test]
async fn external_new_secret_preserves_metadata_and_reports_missing_material() {
    let db = database().await;
    let bus = CommandBus::from_db(db.clone()).await.unwrap();
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let now = "2026-07-24T00:02:00Z".to_string();
    let report = bus
        .apply_external_workspace_variables(vec![ExternalWorkspaceVariableApply::Upsert(
            ExternalWorkspaceVariableUpsert {
                id: "missing-local-secret".to_string(),
                workspace_id: workspace_id.clone(),
                key: "TOKEN".to_string(),
                value: ExternalVariableValue::PreserveLocal,
                is_secret: true,
                is_enabled: true,
                description: None,
                sort_order: 0,
                created_at: now.clone(),
                updated_at: now,
            },
        )])
        .await
        .expect("missing local secret metadata should be created");
    assert_eq!(report.applied_count, 1);
    assert_eq!(report.secret_material_outcomes.len(), 1);
    let outcome = &report.secret_material_outcomes[0];
    assert_eq!(outcome.entity.entity_id, "missing-local-secret");
    assert_eq!(
        outcome.status,
        unfour_core::domain::SecretMaterialStatus::Missing
    );
    let stored: (String, bool, String) = sqlx::query_as(
        "SELECT value, is_secret, key FROM workspace_variables WHERE id = 'missing-local-secret'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(stored.0, "");
    assert!(stored.1);
    assert_eq!(stored.2, "TOKEN");

    let secret = "must-not-leak";
    let error = bus
        .apply_external_workspace_variables(vec![ExternalWorkspaceVariableApply::Upsert(
            ExternalWorkspaceVariableUpsert {
                id: "rejected-secret-set".to_string(),
                workspace_id,
                key: "TOKEN_2".to_string(),
                value: ExternalVariableValue::Set(secret.to_string()),
                is_secret: true,
                is_enabled: true,
                description: None,
                sort_order: 1,
                created_at: "2026-07-24T00:02:01Z".to_string(),
                updated_at: "2026-07-24T00:02:01Z".to_string(),
            },
        )])
        .await
        .expect_err("external secret material must be rejected");
    assert!(!error.to_string().contains(secret));
    let rejected_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workspace_variables WHERE id = 'rejected-secret-set'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(rejected_rows, 0);
}

#[tokio::test]
async fn external_delete_of_active_environment_selects_first_remaining_environment() {
    let db = database().await;
    let bus = CommandBus::from_db(db.clone()).await.unwrap();
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let active = bus
        .workspace_environment_create(workspace_id.clone(), "Active".to_string())
        .await
        .unwrap();
    let second = bus
        .workspace_environment_create(workspace_id.clone(), "Second".to_string())
        .await
        .unwrap();
    let first_fallback = bus
        .workspace_environment_create(workspace_id.clone(), "First Fallback".to_string())
        .await
        .unwrap();
    bus.workspace_environments_reorder(
        workspace_id.clone(),
        vec![
            first_fallback.id.clone(),
            second.id.clone(),
            active.id.clone(),
        ],
    )
    .await
    .unwrap();

    bus.apply_external_workspace_environments(vec![ExternalWorkspaceEnvironmentApply::Delete(
        ExternalDelete {
            entity: DomainEntityKey::new(
                DomainEntityType::WorkspaceEnvironment,
                &workspace_id,
                &active.id,
            ),
            deleted_at: "2026-07-24T00:02:30Z".to_string(),
        },
    )])
    .await
    .unwrap();

    let selected: Option<String> = sqlx::query_scalar(
        "SELECT active_environment_id FROM workspace_local_state WHERE workspace_id = ?1",
    )
    .bind(&workspace_id)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(selected.as_deref(), Some(first_fallback.id.as_str()));
}

#[tokio::test]
async fn external_environment_delete_rolls_back_tombstones_and_fallback_together() {
    let rejecting = Arc::new(SqlHook {
        name: "rejecting",
        fail_on: Some("workspace.external.apply_page"),
        local_only: false,
    });
    let (bus, db) = bus_with_hooks(vec![rejecting]).await;
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let active = bus
        .workspace_environment_create(workspace_id.clone(), "Active".to_string())
        .await
        .unwrap();
    let child = bus
        .workspace_environment_variable_create(
            workspace_id.clone(),
            active.id.clone(),
            input(None, "TOKEN", "local-secret", true),
        )
        .await
        .unwrap();
    let fallback = bus
        .workspace_environment_create(workspace_id.clone(), "Fallback".to_string())
        .await
        .unwrap();
    sqlx::query("DELETE FROM hook_effects")
        .execute(db.pool())
        .await
        .unwrap();

    bus.apply_external_workspace_environments(vec![ExternalWorkspaceEnvironmentApply::Delete(
        ExternalDelete {
            entity: DomainEntityKey::new(
                DomainEntityType::WorkspaceEnvironment,
                &workspace_id,
                &active.id,
            ),
            deleted_at: "2026-07-24T00:02:31Z".to_string(),
        },
    )])
    .await
    .expect_err("hook failure must roll back environment delete transaction");

    let environment_deleted: Option<String> =
        sqlx::query_scalar("SELECT deleted_at FROM workspace_environments WHERE id = ?1")
            .bind(&active.id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    let child_deleted: Option<String> =
        sqlx::query_scalar("SELECT deleted_at FROM workspace_environment_variables WHERE id = ?1")
            .bind(&child.id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    let selected: Option<String> = sqlx::query_scalar(
        "SELECT active_environment_id FROM workspace_local_state WHERE workspace_id = ?1",
    )
    .bind(&workspace_id)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert!(environment_deleted.is_none());
    assert!(child_deleted.is_none());
    assert_eq!(selected.as_deref(), Some(active.id.as_str()));
    assert_ne!(selected.as_deref(), Some(fallback.id.as_str()));
}

#[tokio::test]
async fn external_environment_cascade_reports_returned_child_revision() {
    let capture = Arc::new(SqlHook {
        name: "capture",
        fail_on: None,
        local_only: false,
    });
    let (bus, db) = bus_with_hooks(vec![capture]).await;
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    let environment = bus
        .workspace_environment_create(workspace_id.clone(), "Cascade".to_string())
        .await
        .unwrap();
    let child = bus
        .workspace_environment_variable_create(
            workspace_id.clone(),
            environment.id.clone(),
            input(None, "VALUE", "one", false),
        )
        .await
        .unwrap();
    sqlx::query("DELETE FROM hook_effects")
        .execute(db.pool())
        .await
        .unwrap();

    let report = bus
        .apply_external_workspace_environments(vec![ExternalWorkspaceEnvironmentApply::Delete(
            ExternalDelete {
                entity: DomainEntityKey::new(
                    DomainEntityType::WorkspaceEnvironment,
                    &workspace_id,
                    &environment.id,
                ),
                deleted_at: "2026-07-24T00:03:00Z".to_string(),
            },
        )])
        .await
        .unwrap();

    let stored_revision: i64 =
        sqlx::query_scalar("SELECT revision FROM workspace_environment_variables WHERE id = ?1")
            .bind(&child.id)
            .fetch_one(db.pool())
            .await
            .unwrap();
    let hook_revision: i64 = sqlx::query_scalar(
        "SELECT revision FROM hook_effects WHERE command_name = 'workspace.external.apply_page' AND entity_id = ?1",
    )
    .bind(&child.id)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(hook_revision, stored_revision);
    let child_mutation = report
        .mutations
        .iter()
        .find(|mutation| mutation.entity.entity_id == child.id)
        .unwrap();
    assert_eq!(
        child_mutation.entity.parent_entity_id.as_deref(),
        Some(environment.id.as_str())
    );
    let selected: Option<String> = sqlx::query_scalar(
        "SELECT active_environment_id FROM workspace_local_state WHERE workspace_id = ?1",
    )
    .bind(&workspace_id)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert!(selected.is_none());
    let snapshot = bus
        .read_domain_snapshot(&DomainEntityKey::new(
            DomainEntityType::WorkspaceEnvironmentVariable,
            &workspace_id,
            &child.id,
        ))
        .await
        .unwrap();
    let DomainSnapshot::Tombstone(tombstone) = snapshot else {
        panic!("expected child tombstone");
    };
    assert_eq!(
        tombstone.entity.parent_entity_id.as_deref(),
        Some(environment.id.as_str())
    );
}

#[tokio::test]
async fn legacy_environment_api_uses_the_same_coordinator() {
    let capture = Arc::new(SqlHook {
        name: "capture",
        fail_on: None,
        local_only: false,
    });
    let (bus, db) = bus_with_hooks(vec![capture]).await;
    let workspace_id = bus.list_workspaces().await.unwrap().active_workspace_id;
    bus.api_environment_create(workspace_id, "Legacy".to_string())
        .await
        .unwrap();
    let commands: Vec<(String,)> = sqlx::query_as("SELECT DISTINCT command_name FROM hook_effects")
        .fetch_all(db.pool())
        .await
        .unwrap();
    assert_eq!(
        commands,
        vec![("workspace.environment.create".to_string(),)]
    );
}
