use super::super::*;
use super::support::{service_with_workspace, sqlite_fixture, sqlite_input};
use std::fs;

#[tokio::test]
async fn sqlite_schema_query_and_safe_browse_work() {
    let (service, workspace_id) = service_with_workspace().await;
    let path = sqlite_fixture().await;
    let connection = service
        .save_connection(sqlite_input(&workspace_id, &path))
        .await
        .expect("save connection");

    let test = service
        .test_connection(workspace_id.clone(), connection.id.clone())
        .await
        .expect("test connection");
    assert!(test.ok);
    assert!(test.server_version.is_some());

    let schema = service
        .schema(workspace_id.clone(), connection.id.clone(), None)
        .await
        .expect("schema");
    let deploys = schema
        .tables
        .iter()
        .find(|table| table.name == "deploys")
        .expect("deploys table");
    assert!(deploys
        .columns
        .iter()
        .any(|column| column.name == "service" && !column.primary_key));

    // SQLite is a single-file datasource and exposes no catalogs.
    let catalogs = service
        .list_catalogs(workspace_id.clone(), connection.id.clone())
        .await
        .expect("list catalogs");
    assert!(catalogs.is_empty());

    let query = service
        .execute_query(DatabaseQueryInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            sql: "select service, version from deploys order by id".to_string(),
            limit: Some(1),
            confirm_mutation: None,
            catalog: None,
            schema: None,
            timeout_ms: None,
        })
        .await
        .expect("query");
    assert_eq!(query.rows.len(), 1);
    assert_eq!(query.rows[0][0].as_deref(), Some("api"));
    assert_eq!(query.safety.classification, "read");
    assert!(!query.safety.requires_confirmation);

    let browse = service
        .browse_table(DatabaseBrowseInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            catalog: None,
            schema: None,
            table_name: "deploys".to_string(),
            limit: Some(2),
            offset: Some(1),
            order_by: None,
            order_descending: false,
            filter: None,
            timeout_ms: None,
        })
        .await
        .expect("browse table");
    assert_eq!(browse.table_name, "deploys");
    assert_eq!(browse.sql, "SELECT * FROM \"deploys\" LIMIT 2 OFFSET 1");
    assert_eq!(browse.limit, 2);
    assert_eq!(browse.offset, 1);
    assert_eq!(browse.total_rows, 2);
    assert!(browse.read_only);
    assert_eq!(browse.result.rows.len(), 1);
    assert_eq!(browse.result.rows[0][1].as_deref(), Some("worker"));

    let first_page = service
        .browse_table(DatabaseBrowseInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            catalog: None,
            schema: None,
            table_name: "deploys".to_string(),
            limit: Some(2),
            offset: None,
            order_by: None,
            order_descending: false,
            filter: None,
            timeout_ms: None,
        })
        .await
        .expect("browse first page");
    assert_eq!(first_page.result.rows.len(), 2);

    let empty = service
        .browse_table(DatabaseBrowseInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            catalog: None,
            schema: None,
            table_name: "empty_deploys".to_string(),
            limit: Some(10),
            offset: None,
            order_by: None,
            order_descending: false,
            filter: None,
            timeout_ms: None,
        })
        .await
        .expect("browse empty table");
    assert_eq!(empty.total_rows, 0);
    assert_eq!(empty.result.rows.len(), 0);
    assert_eq!(empty.result.columns.len(), 2);
    assert_eq!(empty.result.columns[1].name, "service");

    let missing = service
        .browse_table(DatabaseBrowseInput {
            workspace_id,
            connection_id: connection.id,
            catalog: None,
            schema: None,
            table_name: "missing".to_string(),
            limit: Some(10),
            offset: None,
            order_by: None,
            order_descending: false,
            filter: None,
            timeout_ms: None,
        })
        .await;
    assert!(matches!(missing, Err(AppError::NotFound(_))));
    let _ = fs::remove_file(path);
}

#[tokio::test]
async fn sqlite_table_structure_exposes_columns_indexes_and_ddl() {
    let (service, workspace_id) = service_with_workspace().await;
    let path = sqlite_fixture().await;
    let connection = service
        .save_connection(sqlite_input(&workspace_id, &path))
        .await
        .expect("save connection");

    let structure = service
        .table_structure(DatabaseTableStructureInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            catalog: None,
            schema: None,
            table_name: "deploys".to_string(),
        })
        .await
        .expect("table structure");

    assert_eq!(structure.name, "deploys");
    assert!(structure
        .columns
        .iter()
        .any(|column| column.name == "version"));
    let ddl = structure.ddl.expect("ddl present");
    assert!(ddl.to_ascii_uppercase().contains("CREATE TABLE"));

    let missing = service
        .table_structure(DatabaseTableStructureInput {
            workspace_id,
            connection_id: connection.id,
            catalog: None,
            schema: None,
            table_name: "missing".to_string(),
        })
        .await;
    assert!(matches!(missing, Err(AppError::NotFound(_))));
    let _ = fs::remove_file(path);
}

#[tokio::test]
async fn sqlite_row_mutation_inserts_updates_and_deletes_by_primary_key() {
    let (service, workspace_id) = service_with_workspace().await;
    let path = sqlite_fixture().await;
    let connection = service
        .save_connection(sqlite_input(&workspace_id, &path))
        .await
        .expect("save connection");

    let insert = service
        .mutate_table_row(DatabaseRowMutationInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            catalog: None,
            schema: None,
            table_name: "deploys".to_string(),
            operation: "insert".to_string(),
            values: vec![
                DatabaseCellValue {
                    column: "id".to_string(),
                    value: Some("99".to_string()),
                },
                DatabaseCellValue {
                    column: "service".to_string(),
                    // Embedded quote must be escaped, not break the statement.
                    value: Some("ai'gent".to_string()),
                },
                DatabaseCellValue {
                    column: "version".to_string(),
                    value: Some("9.9.9".to_string()),
                },
            ],
            primary_key: vec![],
        })
        .await
        .expect("insert row");
    assert_eq!(insert.affected_rows, 1);

    let update = service
        .mutate_table_row(DatabaseRowMutationInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            catalog: None,
            schema: None,
            table_name: "deploys".to_string(),
            operation: "update".to_string(),
            values: vec![DatabaseCellValue {
                column: "version".to_string(),
                value: Some("10.0.0".to_string()),
            }],
            primary_key: vec![DatabaseCellValue {
                column: "id".to_string(),
                value: Some("99".to_string()),
            }],
        })
        .await
        .expect("update row");
    assert_eq!(update.affected_rows, 1);

    // Update/delete without a primary key must be rejected outright.
    let unsafe_update = service
        .mutate_table_row(DatabaseRowMutationInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            catalog: None,
            schema: None,
            table_name: "deploys".to_string(),
            operation: "update".to_string(),
            values: vec![DatabaseCellValue {
                column: "version".to_string(),
                value: Some("0".to_string()),
            }],
            primary_key: vec![],
        })
        .await;
    assert!(matches!(unsafe_update, Err(AppError::Validation(_))));

    let delete = service
        .mutate_table_row(DatabaseRowMutationInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            catalog: None,
            schema: None,
            table_name: "deploys".to_string(),
            operation: "delete".to_string(),
            values: vec![],
            primary_key: vec![DatabaseCellValue {
                column: "id".to_string(),
                value: Some("99".to_string()),
            }],
        })
        .await
        .expect("delete row");
    assert_eq!(delete.affected_rows, 1);

    let _ = fs::remove_file(path);
}

#[tokio::test]
async fn mutating_sql_requires_confirmation_and_rejects_multiple_statements() {
    let (service, workspace_id) = service_with_workspace().await;
    let path = sqlite_fixture().await;
    let connection = service
        .save_connection(sqlite_input(&workspace_id, &path))
        .await
        .expect("save connection");

    let unconfirmed = service
        .execute_query(DatabaseQueryInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            sql: "update deploys set version = '1.0.2' where service = 'api'".to_string(),
            limit: Some(100),
            confirm_mutation: None,
            catalog: None,
            schema: None,
            timeout_ms: None,
        })
        .await;
    assert!(matches!(
        unconfirmed,
        Err(AppError::ConfirmationRequired { .. })
    ));

    let confirmed = service
        .execute_query(DatabaseQueryInput {
            workspace_id: workspace_id.clone(),
            connection_id: connection.id.clone(),
            sql: "update deploys set version = '1.0.2' where service = 'api'".to_string(),
            limit: Some(100),
            confirm_mutation: Some(true),
            catalog: None,
            schema: None,
            timeout_ms: None,
        })
        .await
        .expect("confirmed update");
    assert_eq!(confirmed.affected_rows, 1);
    assert_eq!(confirmed.safety.classification, "mutation");
    assert!(confirmed.safety.confirmed);

    let multiple = service
        .execute_query(DatabaseQueryInput {
            workspace_id,
            connection_id: connection.id,
            sql: "select * from deploys; select * from deploys;".to_string(),
            limit: Some(100),
            confirm_mutation: Some(true),
            catalog: None,
            schema: None,
            timeout_ms: None,
        })
        .await;
    assert!(matches!(multiple, Err(AppError::Validation(_))));
    let _ = fs::remove_file(path);
}
