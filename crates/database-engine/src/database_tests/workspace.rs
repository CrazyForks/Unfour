use super::super::*;
use super::support::{service_with_workspace, sqlite_fixture, sqlite_input};
use std::fs;

#[tokio::test]
async fn query_history_is_workspace_scoped_ordered_limited_and_clearable() {
    let (service, workspace_id) = service_with_workspace().await;
    let other_workspace_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO workspaces (
          id, name, is_default, last_opened_at, created_at, updated_at,
          revision, sync_status
        )
        VALUES (?1, 'Other Workspace', 0, ?2, ?2, ?2, 1, 'local')
        "#,
    )
    .bind(&other_workspace_id)
    .bind(&now)
    .execute(service.db.pool())
    .await
    .expect("insert other workspace");

    service
        .record_query_history(DbQueryHistoryRecordInput {
            id: "history-old".to_string(),
            workspace_id: workspace_id.clone(),
            connection_id: Some("connection-1".to_string()),
            connection_name: "Local SQLite".to_string(),
            sql: "select 1".to_string(),
            status: "success".to_string(),
            classification: Some("read".to_string()),
            row_count: Some(1),
            affected_rows: Some(0),
            duration_ms: Some(3),
            error: None,
            executed_at: "2026-01-01T00:00:00Z".to_string(),
        })
        .await
        .expect("record old history");
    service
        .record_query_history(DbQueryHistoryRecordInput {
            id: "history-new".to_string(),
            workspace_id: workspace_id.clone(),
            connection_id: Some("connection-1".to_string()),
            connection_name: "Local SQLite".to_string(),
            sql: "select 2".to_string(),
            status: "success".to_string(),
            classification: Some("read".to_string()),
            row_count: Some(2),
            affected_rows: Some(0),
            duration_ms: Some(5),
            error: None,
            executed_at: "2026-01-01T00:00:02Z".to_string(),
        })
        .await
        .expect("record new history");
    service
        .record_query_history(DbQueryHistoryRecordInput {
            id: "history-other".to_string(),
            workspace_id: other_workspace_id.clone(),
            connection_id: None,
            connection_name: "Other SQLite".to_string(),
            sql: "select other".to_string(),
            status: "failed".to_string(),
            classification: None,
            row_count: None,
            affected_rows: None,
            duration_ms: None,
            error: Some("syntax error".to_string()),
            executed_at: "2026-01-01T00:00:03Z".to_string(),
        })
        .await
        .expect("record other workspace history");

    let listed = service
        .list_query_history(workspace_id.clone(), Some(10))
        .await
        .expect("list history");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].id, "history-new");
    assert_eq!(listed[0].row_count, Some(2));
    assert_eq!(listed[1].id, "history-old");

    let limited = service
        .list_query_history(workspace_id.clone(), Some(1))
        .await
        .expect("list limited history");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].id, "history-new");

    service
        .clear_query_history(workspace_id.clone())
        .await
        .expect("clear workspace history");
    let cleared = service
        .list_query_history(workspace_id, Some(10))
        .await
        .expect("list cleared history");
    assert!(cleared.is_empty());

    let other = service
        .list_query_history(other_workspace_id, Some(10))
        .await
        .expect("list other workspace history");
    assert_eq!(other.len(), 1);
    assert_eq!(other[0].id, "history-other");
}

#[tokio::test]
async fn saved_sql_crud_is_workspace_scoped_and_validated() {
    let (service, workspace_id) = service_with_workspace().await;

    let created = service
        .save_sql(SavedSqlInput {
            id: None,
            workspace_id: workspace_id.clone(),
            connection_id: Some("conn-1".to_string()),
            name: "Recent users".to_string(),
            sql: "SELECT * FROM users".to_string(),
        })
        .await
        .expect("create saved sql");
    assert_eq!(created.name, "Recent users");
    assert_eq!(created.connection_id.as_deref(), Some("conn-1"));

    let updated = service
        .save_sql(SavedSqlInput {
            id: Some(created.id.clone()),
            workspace_id: workspace_id.clone(),
            connection_id: None,
            name: "Active users".to_string(),
            sql: "SELECT * FROM users WHERE active".to_string(),
        })
        .await
        .expect("update saved sql");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "Active users");
    assert!(updated.connection_id.is_none());

    let listed = service
        .list_saved_sql(workspace_id.clone())
        .await
        .expect("list saved sql");
    assert_eq!(listed.len(), 1);

    // Blank name and blank SQL are rejected.
    assert!(matches!(
        service
            .save_sql(SavedSqlInput {
                id: None,
                workspace_id: workspace_id.clone(),
                connection_id: None,
                name: "   ".to_string(),
                sql: "SELECT 1".to_string(),
            })
            .await,
        Err(AppError::Validation(_))
    ));

    let remaining = service
        .delete_saved_sql(workspace_id.clone(), created.id.clone())
        .await
        .expect("delete saved sql");
    assert!(remaining.is_empty());

    assert!(matches!(
        service.delete_saved_sql(workspace_id, created.id).await,
        Err(AppError::NotFound(_))
    ));
}

#[tokio::test]
async fn connection_crud_is_workspace_scoped_and_soft_deletes() {
    let (service, workspace_id) = service_with_workspace().await;
    let path = sqlite_fixture().await;

    let created = service
        .save_connection(sqlite_input(&workspace_id, &path))
        .await
        .expect("save connection");
    assert_eq!(created.name, "Local fixture");
    assert_eq!(created.driver, "sqlite");
    assert!(created.credential_ref.is_none());

    let listed = service
        .list_connections(workspace_id.clone())
        .await
        .expect("list connections");
    assert_eq!(listed.len(), 1);

    let updated = service
        .save_connection(DatabaseConnectionInput {
            id: Some(created.id.clone()),
            name: "Renamed fixture".to_string(),
            ..sqlite_input(&workspace_id, &path)
        })
        .await
        .expect("update connection");
    assert_eq!(updated.name, "Renamed fixture");
    assert_eq!(updated.revision, created.revision + 1);

    let after_delete = service
        .delete_connection(workspace_id.clone(), created.id)
        .await
        .expect("delete connection");
    assert!(after_delete.is_empty());

    let listed = service
        .list_connections(workspace_id)
        .await
        .expect("list after delete");
    assert!(listed.is_empty());
    let _ = fs::remove_file(path);
}
