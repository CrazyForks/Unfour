use super::*;

#[tokio::test]
async fn external_last_workspace_delete_and_fallback_roll_back_on_hook_failure() {
    let rejecting = Arc::new(SqlHook {
        name: "rejecting",
        fail_on: Some("workspace.external.apply_page"),
        local_only: false,
    });
    let (bus, db) = bus_with_hooks(vec![rejecting]).await;
    let original = bus.list_workspaces().await.unwrap().active_workspace_id;
    sqlx::query("DELETE FROM hook_effects")
        .execute(db.pool())
        .await
        .unwrap();

    bus.apply_external_workspaces(vec![ExternalWorkspaceApply::Delete(ExternalDelete {
        entity: DomainEntityKey::new(DomainEntityType::Workspace, &original, &original),
        deleted_at: "2026-07-24T00:01:31Z".to_string(),
    })])
    .await
    .expect_err("hook failure must roll back delete and fallback creation");

    let original_deleted_at: Option<String> =
        sqlx::query_scalar("SELECT deleted_at FROM workspaces WHERE id = ?1")
            .bind(&original)
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert!(original_deleted_at.is_none());
    let active_workspaces: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM workspaces WHERE deleted_at IS NULL")
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(active_workspaces, 1);
    assert_eq!(
        bus.list_workspaces().await.unwrap().active_workspace_id,
        original
    );
    let hook_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hook_effects")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(hook_rows, 0);
}
