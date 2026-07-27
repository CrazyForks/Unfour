use chrono::Utc;
use sqlx::SqliteConnection;
use unfour_core::domain::{
    CommandContext, DomainCommandResult, DomainEntityKey, DomainEntityType, DomainMutation,
    ExternalApplyPage, ExternalApplyReport, ExternalDelete, ExternalVariableValue,
    ExternalWorkspaceApply, ExternalWorkspaceEnvironmentApply, ExternalWorkspaceEnvironmentUpsert,
    ExternalWorkspaceEnvironmentVariableApply, ExternalWorkspaceEnvironmentVariableUpsert,
    ExternalWorkspaceUpsert, ExternalWorkspaceVariableApply, ExternalWorkspaceVariableUpsert,
    MutationOperation, MutationOrigin, SecretMaterialOutcome, SecretMaterialStatus,
};
use unfour_core::models::{Workspace, WorkspaceEnvironmentVariable, WorkspaceVariable};
use unfour_core::{AppError, AppResult};

use super::variable_executor::{
    entity_mutation, entity_mutation_with_parent, update_active_environment_after_delete_on,
};
use super::variables::{get_environment_on, normalize_description, normalize_environment_name};
use super::{
    get_workspace_on, insert_workspace_companions, normalize_environment_type,
    normalize_mcp_policy, normalize_name, read_setting_on, workspace_mutation, write_setting_on,
    WorkspaceService,
};

impl WorkspaceService {
    pub async fn apply_external_page_on(
        &self,
        connection: &mut SqliteConnection,
        context: &CommandContext,
        page: ExternalApplyPage,
    ) -> AppResult<DomainCommandResult<ExternalApplyReport>> {
        if context.origin != MutationOrigin::External {
            return Err(AppError::Config(
                "external apply requires an External command context".to_string(),
            ));
        }
        let mut mutations = Vec::new();
        let mut secret_material_outcomes = Vec::new();
        for change in page.workspaces {
            apply_workspace(connection, context, change, &mut mutations).await?;
        }
        for change in page.workspace_environments {
            apply_environment(connection, context, change, &mut mutations).await?;
        }
        for change in page.workspace_variables {
            apply_workspace_variable(
                connection,
                context,
                change,
                &mut mutations,
                &mut secret_material_outcomes,
            )
            .await?;
        }
        for change in page.workspace_environment_variables {
            apply_environment_variable(
                connection,
                context,
                change,
                &mut mutations,
                &mut secret_material_outcomes,
            )
            .await?;
        }
        let report = ExternalApplyReport {
            applied_count: mutations.len(),
            mutations: mutations.clone(),
            secret_material_outcomes,
        };
        Ok(DomainCommandResult::new(report, mutations))
    }
}

async fn apply_workspace(
    connection: &mut SqliteConnection,
    context: &CommandContext,
    change: ExternalWorkspaceApply,
    mutations: &mut Vec<DomainMutation>,
) -> AppResult<()> {
    match change {
        ExternalWorkspaceApply::Upsert(record) => {
            let id = record.id.clone();
            if let Some(revision) = upsert_workspace(connection, record).await? {
                mutations.push(workspace_mutation(
                    context,
                    MutationOperation::Upsert,
                    &id,
                    revision,
                ));
            }
        }
        ExternalWorkspaceApply::Delete(delete) => {
            validate_delete(&delete, DomainEntityType::Workspace)?;
            if delete.entity.workspace_id != delete.entity.entity_id {
                return Err(AppError::Validation(
                    "workspace delete key must use the workspace id as entity id".to_string(),
                ));
            }
            let current = get_workspace_on(connection, &delete.entity.workspace_id, true).await?;
            if current.deleted_at.is_some() {
                return Ok(());
            }
            let active_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM workspaces WHERE deleted_at IS NULL")
                    .fetch_one(&mut *connection)
                    .await?;
            let deleting_last = active_count <= 1;
            if let Some(revision) = delete_existing(
                connection,
                "workspaces",
                &delete.entity.workspace_id,
                &delete.entity.entity_id,
                &delete.deleted_at,
            )
            .await?
            {
                mutations.push(workspace_mutation(
                    context,
                    MutationOperation::Delete,
                    &delete.entity.workspace_id,
                    revision,
                ));
                if deleting_last {
                    let (fallback_id, fallback_revision) =
                        create_external_fallback_workspace(connection).await?;
                    mutations.push(workspace_mutation(
                        context,
                        MutationOperation::Upsert,
                        &fallback_id,
                        fallback_revision,
                    ));
                    return Ok(());
                }
                if read_setting_on(connection, "active_workspace_id")
                    .await?
                    .as_deref()
                    == Some(delete.entity.workspace_id.as_str())
                {
                    let next: String = sqlx::query_scalar(
                        r#"
                        SELECT id FROM workspaces
                        WHERE deleted_at IS NULL
                        ORDER BY is_default DESC, updated_at DESC
                        LIMIT 1
                        "#,
                    )
                    .fetch_one(&mut *connection)
                    .await?;
                    write_setting_on(connection, "active_workspace_id", &next).await?;
                }
            }
        }
    }
    Ok(())
}

async fn create_external_fallback_workspace(
    connection: &mut SqliteConnection,
) -> AppResult<(String, i64)> {
    let id = unfour_core::id::new_id();
    let now = Utc::now().to_rfc3339();
    let name = available_default_workspace_name(connection).await?;
    sqlx::query(
        r#"
        INSERT INTO workspaces (
          id, name, is_default, last_opened_at, environment_type, mcp_policy,
          created_at, updated_at, revision
        ) VALUES (?1, ?2, 1, ?3, 'dev', 'auto', ?3, ?3, 1)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(&now)
    .execute(&mut *connection)
    .await?;
    insert_workspace_companions(connection, &id, &now).await?;
    write_setting_on(connection, "active_workspace_id", &id).await?;
    Ok((id, 1))
}

async fn available_default_workspace_name(connection: &mut SqliteConnection) -> AppResult<String> {
    let base = "Default Workspace";
    for suffix in 1_u32.. {
        let candidate = if suffix == 1 {
            base.to_string()
        } else {
            format!("{base} {suffix}")
        };
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM workspaces WHERE name = ?1 COLLATE NOCASE AND deleted_at IS NULL)",
        )
        .bind(&candidate)
        .fetch_one(&mut *connection)
        .await?;
        if !exists {
            return Ok(candidate);
        }
    }
    unreachable!("u32 workspace-name suffix space exhausted")
}

async fn upsert_workspace(
    connection: &mut SqliteConnection,
    record: ExternalWorkspaceUpsert,
) -> AppResult<Option<i64>> {
    let name = normalize_name(record.name)?;
    let environment_type = normalize_environment_type(Some(record.environment_type))?;
    let mcp_policy = normalize_mcp_policy(Some(record.mcp_policy))?;
    let current = sqlx::query_as::<_, Workspace>(
        r#"
        SELECT id, name, is_default, last_opened_at, environment_type, mcp_policy,
               created_at, updated_at, deleted_at, revision
        FROM workspaces WHERE id = ?1
        "#,
    )
    .bind(&record.id)
    .fetch_optional(&mut *connection)
    .await?;
    if let Some(current) = current {
        if current.deleted_at.is_none()
            && current.name == name
            && current.environment_type == environment_type
            && current.mcp_policy == mcp_policy
            && current.created_at == record.created_at
            && current.updated_at == record.updated_at
        {
            return Ok(None);
        }
        let revision = sqlx::query_scalar(
            r#"
            UPDATE workspaces
            SET name = ?1, environment_type = ?2, mcp_policy = ?3,
                created_at = ?4, updated_at = ?5, deleted_at = NULL,
                revision = revision + 1
            WHERE id = ?6 RETURNING revision
            "#,
        )
        .bind(name)
        .bind(environment_type)
        .bind(mcp_policy)
        .bind(record.created_at)
        .bind(record.updated_at)
        .bind(record.id)
        .fetch_one(&mut *connection)
        .await?;
        return Ok(Some(revision));
    }

    sqlx::query(
        r#"
        INSERT INTO workspaces (
          id, name, is_default, last_opened_at, environment_type, mcp_policy,
          created_at, updated_at, revision
        ) VALUES (?1, ?2, 0, NULL, ?3, ?4, ?5, ?6, 1)
        "#,
    )
    .bind(&record.id)
    .bind(name)
    .bind(environment_type)
    .bind(mcp_policy)
    .bind(&record.created_at)
    .bind(&record.updated_at)
    .execute(&mut *connection)
    .await?;
    insert_workspace_companions(connection, &record.id, &record.created_at).await?;
    Ok(Some(1))
}

async fn apply_environment(
    connection: &mut SqliteConnection,
    context: &CommandContext,
    change: ExternalWorkspaceEnvironmentApply,
    mutations: &mut Vec<DomainMutation>,
) -> AppResult<()> {
    match change {
        ExternalWorkspaceEnvironmentApply::Upsert(record) => {
            let workspace_id = record.workspace_id.clone();
            let id = record.id.clone();
            if let Some(revision) = upsert_environment(connection, record).await? {
                mutations.push(entity_mutation(
                    context,
                    DomainEntityType::WorkspaceEnvironment,
                    MutationOperation::Upsert,
                    &workspace_id,
                    &id,
                    revision,
                ));
            }
        }
        ExternalWorkspaceEnvironmentApply::Delete(delete) => {
            validate_delete(&delete, DomainEntityType::WorkspaceEnvironment)?;
            let was_active =
                super::variables::active_environment_id_on(connection, &delete.entity.workspace_id)
                    .await?
                    .as_deref()
                    == Some(delete.entity.entity_id.as_str());
            if let Some(revision) = delete_existing(
                connection,
                "workspace_environments",
                &delete.entity.workspace_id,
                &delete.entity.entity_id,
                &delete.deleted_at,
            )
            .await?
            {
                mutations.push(entity_mutation(
                    context,
                    DomainEntityType::WorkspaceEnvironment,
                    MutationOperation::Delete,
                    &delete.entity.workspace_id,
                    &delete.entity.entity_id,
                    revision,
                ));
                let children: Vec<(String, i64)> = sqlx::query_as(
                    r#"
                    UPDATE workspace_environment_variables
                    SET deleted_at = ?1, updated_at = ?1, revision = revision + 1
                    WHERE workspace_id = ?2 AND environment_id = ?3 AND deleted_at IS NULL
                    RETURNING id, revision
                    "#,
                )
                .bind(&delete.deleted_at)
                .bind(&delete.entity.workspace_id)
                .bind(&delete.entity.entity_id)
                .fetch_all(&mut *connection)
                .await?;
                for (id, revision) in children {
                    mutations.push(entity_mutation_with_parent(
                        context,
                        DomainEntityType::WorkspaceEnvironmentVariable,
                        MutationOperation::Delete,
                        &delete.entity.workspace_id,
                        &id,
                        &delete.entity.entity_id,
                        revision,
                    ));
                }
                if was_active {
                    update_active_environment_after_delete_on(
                        connection,
                        &delete.entity.workspace_id,
                        &delete.entity.entity_id,
                        &delete.deleted_at,
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}

async fn upsert_environment(
    connection: &mut SqliteConnection,
    record: ExternalWorkspaceEnvironmentUpsert,
) -> AppResult<Option<i64>> {
    get_workspace_on(connection, &record.workspace_id, false).await?;
    let name = normalize_environment_name(record.name)?;
    let current = sqlx::query_as::<_, super::variables::WorkspaceEnvironmentRow>(
        r#"
        SELECT id, workspace_id, name, sort_order, created_at, updated_at,
               deleted_at, revision
        FROM workspace_environments WHERE id = ?1 AND workspace_id = ?2
        "#,
    )
    .bind(&record.id)
    .bind(&record.workspace_id)
    .fetch_optional(&mut *connection)
    .await?;
    if let Some(current) = current {
        if current.deleted_at.is_none()
            && current.name == name
            && current.sort_order == record.sort_order
            && current.created_at == record.created_at
            && current.updated_at == record.updated_at
        {
            return Ok(None);
        }
        return Ok(Some(
            sqlx::query_scalar(
                r#"
                UPDATE workspace_environments
                SET name = ?1, sort_order = ?2, created_at = ?3, updated_at = ?4,
                    deleted_at = NULL, revision = revision + 1
                WHERE id = ?5 AND workspace_id = ?6 RETURNING revision
                "#,
            )
            .bind(name)
            .bind(record.sort_order)
            .bind(record.created_at)
            .bind(record.updated_at)
            .bind(record.id)
            .bind(record.workspace_id)
            .fetch_one(&mut *connection)
            .await?,
        ));
    }
    sqlx::query(
        r#"
        INSERT INTO workspace_environments (
          id, workspace_id, name, sort_order, created_at, updated_at, revision
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)
        "#,
    )
    .bind(record.id)
    .bind(record.workspace_id)
    .bind(name)
    .bind(record.sort_order)
    .bind(record.created_at)
    .bind(record.updated_at)
    .execute(&mut *connection)
    .await?;
    Ok(Some(1))
}

async fn apply_workspace_variable(
    connection: &mut SqliteConnection,
    context: &CommandContext,
    change: ExternalWorkspaceVariableApply,
    mutations: &mut Vec<DomainMutation>,
    secret_material_outcomes: &mut Vec<SecretMaterialOutcome>,
) -> AppResult<()> {
    match change {
        ExternalWorkspaceVariableApply::Upsert(record) => {
            let workspace_id = record.workspace_id.clone();
            let id = record.id.clone();
            let (revision, secret_status) = upsert_workspace_variable(connection, record).await?;
            if let Some(status) = secret_status {
                secret_material_outcomes.push(SecretMaterialOutcome {
                    entity: DomainEntityKey::new(
                        DomainEntityType::WorkspaceVariable,
                        &workspace_id,
                        &id,
                    ),
                    status,
                });
            }
            if let Some(revision) = revision {
                mutations.push(entity_mutation(
                    context,
                    DomainEntityType::WorkspaceVariable,
                    MutationOperation::Upsert,
                    &workspace_id,
                    &id,
                    revision,
                ));
            }
        }
        ExternalWorkspaceVariableApply::Delete(delete) => {
            validate_delete(&delete, DomainEntityType::WorkspaceVariable)?;
            if let Some(revision) = delete_existing(
                connection,
                "workspace_variables",
                &delete.entity.workspace_id,
                &delete.entity.entity_id,
                &delete.deleted_at,
            )
            .await?
            {
                mutations.push(entity_mutation(
                    context,
                    DomainEntityType::WorkspaceVariable,
                    MutationOperation::Delete,
                    &delete.entity.workspace_id,
                    &delete.entity.entity_id,
                    revision,
                ));
            }
        }
    }
    Ok(())
}

async fn upsert_workspace_variable(
    connection: &mut SqliteConnection,
    record: ExternalWorkspaceVariableUpsert,
) -> AppResult<(Option<i64>, Option<SecretMaterialStatus>)> {
    get_workspace_on(connection, &record.workspace_id, false).await?;
    let key = normalized_key(&record.key)?;
    let current = sqlx::query_as::<_, WorkspaceVariable>(
        r#"
        SELECT id, workspace_id, key, value, is_secret, is_enabled, description,
               sort_order, created_at, updated_at, deleted_at, revision
        FROM workspace_variables WHERE id = ?1 AND workspace_id = ?2
        "#,
    )
    .bind(&record.id)
    .bind(&record.workspace_id)
    .fetch_optional(&mut *connection)
    .await?;
    let (value, secret_status) = external_value(
        record.is_secret,
        &record.value,
        current.as_ref().map(|v| &v.value),
    )?;
    let description = normalize_description(record.description);
    if let Some(current) = current {
        if current.deleted_at.is_none()
            && current.key == key
            && current.value == value
            && current.is_secret == record.is_secret
            && current.is_enabled == record.is_enabled
            && current.description == description
            && current.sort_order == record.sort_order
            && current.created_at == record.created_at
            && current.updated_at == record.updated_at
        {
            return Ok((None, secret_status));
        }
        return Ok((
            Some(
                sqlx::query_scalar(
                    r#"
                UPDATE workspace_variables
                SET key = ?1, value = ?2, is_secret = ?3, is_enabled = ?4,
                    description = ?5, sort_order = ?6, created_at = ?7,
                    updated_at = ?8, deleted_at = NULL, revision = revision + 1
                WHERE id = ?9 AND workspace_id = ?10 RETURNING revision
                "#,
                )
                .bind(key)
                .bind(value)
                .bind(record.is_secret)
                .bind(record.is_enabled)
                .bind(description)
                .bind(record.sort_order)
                .bind(record.created_at)
                .bind(record.updated_at)
                .bind(record.id)
                .bind(record.workspace_id)
                .fetch_one(&mut *connection)
                .await?,
            ),
            secret_status,
        ));
    }
    sqlx::query(
        r#"
        INSERT INTO workspace_variables (
          id, workspace_id, key, value, is_secret, is_enabled, description,
          sort_order, created_at, updated_at, revision
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1)
        "#,
    )
    .bind(record.id)
    .bind(record.workspace_id)
    .bind(key)
    .bind(value)
    .bind(record.is_secret)
    .bind(record.is_enabled)
    .bind(description)
    .bind(record.sort_order)
    .bind(record.created_at)
    .bind(record.updated_at)
    .execute(&mut *connection)
    .await?;
    Ok((Some(1), secret_status))
}

async fn apply_environment_variable(
    connection: &mut SqliteConnection,
    context: &CommandContext,
    change: ExternalWorkspaceEnvironmentVariableApply,
    mutations: &mut Vec<DomainMutation>,
    secret_material_outcomes: &mut Vec<SecretMaterialOutcome>,
) -> AppResult<()> {
    match change {
        ExternalWorkspaceEnvironmentVariableApply::Upsert(record) => {
            let workspace_id = record.workspace_id.clone();
            let environment_id = record.environment_id.clone();
            let id = record.id.clone();
            let (revision, secret_status) = upsert_environment_variable(connection, record).await?;
            if let Some(status) = secret_status {
                secret_material_outcomes.push(SecretMaterialOutcome {
                    entity: DomainEntityKey::new(
                        DomainEntityType::WorkspaceEnvironmentVariable,
                        &workspace_id,
                        &id,
                    )
                    .with_parent_entity_id(&environment_id),
                    status,
                });
            }
            if let Some(revision) = revision {
                mutations.push(entity_mutation_with_parent(
                    context,
                    DomainEntityType::WorkspaceEnvironmentVariable,
                    MutationOperation::Upsert,
                    &workspace_id,
                    &id,
                    &environment_id,
                    revision,
                ));
            }
        }
        ExternalWorkspaceEnvironmentVariableApply::Delete(delete) => {
            validate_delete(&delete, DomainEntityType::WorkspaceEnvironmentVariable)?;
            let parent_entity_id: Option<String> = sqlx::query_scalar(
                "SELECT environment_id FROM workspace_environment_variables WHERE id = ?1 AND workspace_id = ?2",
            )
            .bind(&delete.entity.entity_id)
            .bind(&delete.entity.workspace_id)
            .fetch_optional(&mut *connection)
            .await?;
            if let Some(revision) = delete_existing(
                connection,
                "workspace_environment_variables",
                &delete.entity.workspace_id,
                &delete.entity.entity_id,
                &delete.deleted_at,
            )
            .await?
            {
                let parent_entity_id = parent_entity_id.ok_or_else(|| {
                    AppError::Config(
                        "environment variable delete lost its parent environment".to_string(),
                    )
                })?;
                if delete
                    .entity
                    .parent_entity_id
                    .as_deref()
                    .is_some_and(|provided| provided != parent_entity_id.as_str())
                {
                    return Err(AppError::Validation(
                        "environment variable delete parent entity does not match local metadata"
                            .to_string(),
                    ));
                }
                mutations.push(entity_mutation_with_parent(
                    context,
                    DomainEntityType::WorkspaceEnvironmentVariable,
                    MutationOperation::Delete,
                    &delete.entity.workspace_id,
                    &delete.entity.entity_id,
                    &parent_entity_id,
                    revision,
                ));
            }
        }
    }
    Ok(())
}

async fn upsert_environment_variable(
    connection: &mut SqliteConnection,
    record: ExternalWorkspaceEnvironmentVariableUpsert,
) -> AppResult<(Option<i64>, Option<SecretMaterialStatus>)> {
    get_environment_on(
        connection,
        &record.workspace_id,
        &record.environment_id,
        false,
    )
    .await?;
    let key = normalized_key(&record.key)?;
    let current = sqlx::query_as::<_, WorkspaceEnvironmentVariable>(
        r#"
        SELECT id, workspace_id, environment_id, key, value, is_secret,
               is_enabled, description, sort_order, created_at, updated_at,
               deleted_at, revision
        FROM workspace_environment_variables WHERE id = ?1 AND workspace_id = ?2
        "#,
    )
    .bind(&record.id)
    .bind(&record.workspace_id)
    .fetch_optional(&mut *connection)
    .await?;
    let (value, secret_status) = external_value(
        record.is_secret,
        &record.value,
        current.as_ref().map(|v| &v.value),
    )?;
    let description = normalize_description(record.description);
    if let Some(current) = current {
        if current.deleted_at.is_none()
            && current.environment_id == record.environment_id
            && current.key == key
            && current.value == value
            && current.is_secret == record.is_secret
            && current.is_enabled == record.is_enabled
            && current.description == description
            && current.sort_order == record.sort_order
            && current.created_at == record.created_at
            && current.updated_at == record.updated_at
        {
            return Ok((None, secret_status));
        }
        return Ok((
            Some(
                sqlx::query_scalar(
                    r#"
                UPDATE workspace_environment_variables
                SET environment_id = ?1, key = ?2, value = ?3, is_secret = ?4,
                    is_enabled = ?5, description = ?6, sort_order = ?7,
                    created_at = ?8, updated_at = ?9, deleted_at = NULL,
                    revision = revision + 1
                WHERE id = ?10 AND workspace_id = ?11 RETURNING revision
                "#,
                )
                .bind(record.environment_id)
                .bind(key)
                .bind(value)
                .bind(record.is_secret)
                .bind(record.is_enabled)
                .bind(description)
                .bind(record.sort_order)
                .bind(record.created_at)
                .bind(record.updated_at)
                .bind(record.id)
                .bind(record.workspace_id)
                .fetch_one(&mut *connection)
                .await?,
            ),
            secret_status,
        ));
    }
    sqlx::query(
        r#"
        INSERT INTO workspace_environment_variables (
          id, workspace_id, environment_id, key, value, is_secret, is_enabled,
          description, sort_order, created_at, updated_at, revision
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1)
        "#,
    )
    .bind(record.id)
    .bind(record.workspace_id)
    .bind(record.environment_id)
    .bind(key)
    .bind(value)
    .bind(record.is_secret)
    .bind(record.is_enabled)
    .bind(description)
    .bind(record.sort_order)
    .bind(record.created_at)
    .bind(record.updated_at)
    .execute(&mut *connection)
    .await?;
    Ok((Some(1), secret_status))
}

fn external_value(
    is_secret: bool,
    value: &ExternalVariableValue,
    current: Option<&String>,
) -> AppResult<(String, Option<SecretMaterialStatus>)> {
    match (is_secret, value) {
        (true, ExternalVariableValue::Set(_)) => Err(AppError::Validation(
            "secret external values must use PreserveLocal or Clear".to_string(),
        )),
        (true, ExternalVariableValue::PreserveLocal) => {
            let value = current.cloned().unwrap_or_default();
            let status = if current.is_some_and(|value| !value.is_empty()) {
                SecretMaterialStatus::Present
            } else {
                SecretMaterialStatus::Missing
            };
            Ok((value, Some(status)))
        }
        (true, ExternalVariableValue::Clear) => {
            Ok((String::new(), Some(SecretMaterialStatus::Missing)))
        }
        (false, ExternalVariableValue::Clear) => Ok((String::new(), None)),
        (false, ExternalVariableValue::Set(value)) => Ok((value.clone(), None)),
        (false, ExternalVariableValue::PreserveLocal) => Err(AppError::Validation(
            "plain external values must use Set or Clear".to_string(),
        )),
    }
}

async fn delete_existing(
    connection: &mut SqliteConnection,
    table: &str,
    workspace_id: &str,
    entity_id: &str,
    deleted_at: &str,
) -> AppResult<Option<i64>> {
    if deleted_at.trim().is_empty() {
        return Err(AppError::Validation(
            "external delete requires deleted_at".to_string(),
        ));
    }
    let sql = match table {
        "workspaces" => {
            "UPDATE workspaces SET deleted_at = ?1, updated_at = ?1, revision = revision + 1 WHERE id = ?2 AND deleted_at IS NULL RETURNING revision"
        }
        "workspace_variables" => {
            "UPDATE workspace_variables SET deleted_at = ?1, updated_at = ?1, revision = revision + 1 WHERE id = ?2 AND workspace_id = ?3 AND deleted_at IS NULL RETURNING revision"
        }
        "workspace_environments" => {
            "UPDATE workspace_environments SET deleted_at = ?1, updated_at = ?1, revision = revision + 1 WHERE id = ?2 AND workspace_id = ?3 AND deleted_at IS NULL RETURNING revision"
        }
        "workspace_environment_variables" => {
            "UPDATE workspace_environment_variables SET deleted_at = ?1, updated_at = ?1, revision = revision + 1 WHERE id = ?2 AND workspace_id = ?3 AND deleted_at IS NULL RETURNING revision"
        }
        _ => return Err(AppError::Config("unsupported external entity table".to_string())),
    };
    let mut query = sqlx::query_scalar(sql).bind(deleted_at).bind(entity_id);
    if table != "workspaces" {
        query = query.bind(workspace_id);
    }
    Ok(query.fetch_optional(&mut *connection).await?)
}

fn validate_delete(delete: &ExternalDelete, expected: DomainEntityType) -> AppResult<()> {
    if delete.entity.entity_type != expected {
        return Err(AppError::Validation(
            "external delete entity type does not match its apply collection".to_string(),
        ));
    }
    if delete.entity.workspace_id.trim().is_empty() || delete.entity.entity_id.trim().is_empty() {
        return Err(AppError::Validation(
            "external delete requires non-empty entity ids".to_string(),
        ));
    }
    Ok(())
}

fn normalized_key(key: &str) -> AppResult<String> {
    let key = key.trim();
    if key.is_empty() {
        return Err(AppError::Validation(
            "variable key cannot be empty".to_string(),
        ));
    }
    if key.chars().count() > 120 {
        return Err(AppError::Validation(
            "variable key must be 120 characters or fewer".to_string(),
        ));
    }
    Ok(key.to_string())
}
