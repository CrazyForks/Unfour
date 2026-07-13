use super::*;

pub(super) async fn pg_connect_options(
    connection: &DatabaseConnection,
    secret_store: Option<&SecretStore>,
    password_override: Option<&str>,
) -> AppResult<PgConnectOptions> {
    let host = connection
        .host
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("127.0.0.1");
    let port = connection.port.unwrap_or(5432);
    let database = connection
        .database
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::Validation("PostgreSQL database name is required".to_string()))?;
    let username = connection
        .username
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::Validation("PostgreSQL username is required".to_string()))?;

    let password = match password_override {
        Some(secret) => Some(secret.to_string()),
        None => resolve_database_password(connection, secret_store).await?,
    };

    let mut options = PgConnectOptions::new()
        .host(host)
        .port(port as u16)
        .database(database)
        .username(username);

    if let Some(pw) = password {
        options = options.password(&pw);
    }

    Ok(options)
}

/// Load a database password from SecretStore if a credential reference is
/// present. Returns `None` when no credential_ref is configured.
pub(super) async fn resolve_database_password(
    connection: &DatabaseConnection,
    secret_store: Option<&SecretStore>,
) -> AppResult<Option<String>> {
    if let Some(credential_ref) = connection
        .credential_ref
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let store = secret_store.ok_or_else(|| {
            AppError::Config(
                "SecretStore is not available; cannot load database password".to_string(),
            )
        })?;
        let secret = store
            .read_secret(connection.workspace_id.clone(), credential_ref.to_string())
            .await
            .map_err(|_| {
                AppError::Config("Failed to load database password from SecretStore".to_string())
            })?;
        Ok(Some(secret))
    } else {
        Ok(None)
    }
}

pub(super) async fn postgres_columns(
    pool: &sqlx::PgPool,
    schema: &str,
    table_name: &str,
) -> Result<Vec<DatabaseTableColumn>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT column_name, data_type, is_nullable, column_default
        FROM information_schema.columns
        WHERE table_schema = $1 AND table_name = $2
        ORDER BY ordinal_position
        "#,
    )
    .bind(schema)
    .bind(table_name)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let name: String = row.try_get("column_name")?;
            let data_type: String = row.try_get("data_type")?;
            let is_nullable: String = row.try_get("is_nullable")?;
            let column_default: Option<String> = row.try_get("column_default")?;

            // Detect primary key from column_default (serial types get nextval)
            // For a more accurate check we'd need pg_constraint, but this is
            // sufficient for the initial phase.
            let primary_key = column_default
                .as_deref()
                .map(|d| d.starts_with("nextval("))
                .unwrap_or(false);

            Ok(DatabaseTableColumn {
                name,
                data_type,
                nullable: is_nullable == "YES",
                primary_key,
                default_value: column_default,
            })
        })
        .collect()
}

pub(super) async fn postgres_indexes(
    pool: &sqlx::PgPool,
    schema: &str,
    table_name: &str,
) -> Result<Vec<DatabaseIndex>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT i.relname AS index_name,
               ix.indisunique AS is_unique,
               ix.indisprimary AS is_primary,
               a.attname AS column_name,
               array_position(ix.indkey::int2[], a.attnum) AS ord
        FROM pg_class t
        JOIN pg_namespace n ON n.oid = t.relnamespace
        JOIN pg_index ix ON ix.indrelid = t.oid
        JOIN pg_class i ON i.oid = ix.indexrelid
        JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey::int2[])
        WHERE n.nspname = $1 AND t.relname = $2
        ORDER BY index_name, ord
        "#,
    )
    .bind(schema)
    .bind(table_name)
    .fetch_all(pool)
    .await?;

    let mut indexes: Vec<DatabaseIndex> = Vec::new();
    for row in rows {
        let name: String = row.try_get("index_name")?;
        let unique: bool = row.try_get("is_unique")?;
        let primary: bool = row.try_get("is_primary")?;
        let column_name: String = row.try_get("column_name")?;

        if let Some(existing) = indexes.iter_mut().find(|idx| idx.name == name) {
            existing.columns.push(column_name);
        } else {
            indexes.push(DatabaseIndex {
                name,
                columns: vec![column_name],
                unique,
                primary,
            });
        }
    }

    Ok(indexes)
}

pub(super) async fn postgres_foreign_keys(
    pool: &sqlx::PgPool,
    schema: &str,
    table_name: &str,
) -> Result<Vec<DatabaseForeignKey>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT con.conname AS name,
               att.attname AS column_name,
               cl.relname AS referenced_table,
               fatt.attname AS referenced_column,
               k.ord AS ord
        FROM pg_constraint con
        JOIN pg_class c ON c.oid = con.conrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        JOIN LATERAL unnest(con.conkey) WITH ORDINALITY AS k(attnum, ord) ON true
        JOIN pg_attribute att ON att.attrelid = con.conrelid AND att.attnum = k.attnum
        JOIN pg_class cl ON cl.oid = con.confrelid
        JOIN LATERAL unnest(con.confkey) WITH ORDINALITY AS fk(attnum, ford) ON fk.ford = k.ord
        JOIN pg_attribute fatt ON fatt.attrelid = con.confrelid AND fatt.attnum = fk.attnum
        WHERE con.contype = 'f' AND n.nspname = $1 AND c.relname = $2
        ORDER BY name, ord
        "#,
    )
    .bind(schema)
    .bind(table_name)
    .fetch_all(pool)
    .await?;

    let mut keys: Vec<DatabaseForeignKey> = Vec::new();
    for row in rows {
        let name: String = row.try_get("name")?;
        let column_name: String = row.try_get("column_name")?;
        let referenced_table: String = row.try_get("referenced_table")?;
        let referenced_column: String = row.try_get("referenced_column")?;

        if let Some(existing) = keys.iter_mut().find(|fk| fk.name == name) {
            existing.columns.push(column_name);
            existing.referenced_columns.push(referenced_column);
        } else {
            keys.push(DatabaseForeignKey {
                name,
                columns: vec![column_name],
                referenced_table,
                referenced_columns: vec![referenced_column],
            });
        }
    }

    Ok(keys)
}

pub(super) async fn ensure_postgres_table_exists(
    pool: &sqlx::PgPool,
    schema: &str,
    table_name: &str,
) -> Result<(), AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = $1
          AND table_name = $2
          AND table_schema NOT IN ('pg_catalog', 'information_schema')
        LIMIT 1
        "#,
    )
    .bind(schema)
    .bind(table_name)
    .fetch_optional(pool)
    .await?;

    row.map(|_| ())
        .ok_or_else(|| AppError::NotFound(format!("{schema}.{table_name}")))
}

/// Resolve the object kind ("table" or "view") from information_schema so the
/// structure panel reports views correctly instead of always returning
/// "table". Falls back to "table" when the row is missing (the caller has
/// already verified existence via `ensure_postgres_table_exists`).
pub(super) async fn postgres_table_kind(
    pool: &sqlx::PgPool,
    schema: &str,
    table_name: &str,
) -> Result<String, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT table_type
        FROM information_schema.tables
        WHERE table_schema = $1 AND table_name = $2
        LIMIT 1
        "#,
    )
    .bind(schema)
    .bind(table_name)
    .fetch_optional(pool)
    .await?;
    Ok(row
        .map(|(table_type,)| {
            if table_type == "VIEW" {
                "view".to_string()
            } else {
                "table".to_string()
            }
        })
        .unwrap_or_else(|| "table".to_string()))
}

pub(super) async fn postgres_table_row_count(
    pool: &sqlx::PgPool,
    schema: &str,
    table_name: &str,
) -> Result<u64, AppError> {
    let sql = format!(
        "SELECT COUNT(*) AS total_rows FROM {}",
        quote_qualified_identifier(schema, table_name)
    );
    let row = sqlx::query(&sql).fetch_one(pool).await?;
    let total_rows: i64 = row.try_get("total_rows")?;
    Ok(total_rows.max(0) as u64)
}

pub(super) async fn postgres_table_result_columns(
    pool: &sqlx::PgPool,
    schema: &str,
    table_name: &str,
) -> Result<Vec<DatabaseResultColumn>, AppError> {
    Ok(postgres_columns(pool, schema, table_name)
        .await?
        .into_iter()
        .map(|column| DatabaseResultColumn {
            name: column.name,
            data_type: column.data_type,
        })
        .collect())
}

pub(super) fn postgres_result_columns(row: &sqlx::postgres::PgRow) -> Vec<DatabaseResultColumn> {
    row.columns()
        .iter()
        .map(|column| DatabaseResultColumn {
            name: column.name().to_string(),
            data_type: column.type_info().name().to_string(),
        })
        .collect()
}

pub(super) fn postgres_table_from_metadata(
    catalog: Option<String>,
    schema: String,
    name: String,
    kind: String,
    columns: Vec<DatabaseTableColumn>,
) -> DatabaseTable {
    DatabaseTable {
        catalog,
        schema: Some(schema),
        name,
        kind,
        columns,
    }
}

pub(super) fn postgres_row_values(row: &sqlx::postgres::PgRow) -> AppResult<Vec<Option<String>>> {
    (0..row.columns().len())
        .map(|index| {
            let raw = row.try_get_raw(index)?;
            if raw.is_null() {
                return Ok(None);
            }

            if let Ok(value) = row.try_get::<String, _>(index) {
                return Ok(Some(value));
            }
            if let Ok(value) = row.try_get::<i64, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<i32, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<i16, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<f64, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<f32, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<bool, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<Vec<u8>, _>(index) {
                return Ok(Some(format!("<binary {} bytes>", value.len())));
            }
            if let Ok(value) = row.try_get::<serde_json::Value, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<uuid::Uuid, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(index) {
                return Ok(Some(value.to_rfc3339()));
            }
            if let Ok(value) = row.try_get::<chrono::DateTime<chrono::Local>, _>(index) {
                return Ok(Some(value.to_rfc3339()));
            }
            if let Ok(value) = row.try_get::<chrono::NaiveDateTime, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<chrono::NaiveDate, _>(index) {
                return Ok(Some(value.to_string()));
            }
            if let Ok(value) = row.try_get::<chrono::NaiveTime, _>(index) {
                return Ok(Some(value.to_string()));
            }

            Ok(Some("<unsupported>".to_string()))
        })
        .collect()
}

/// Sanitize a sqlx::Error into an AppError.
///
/// Instead of wiping the whole message whenever it mentions credentials, we
/// keep the original diagnostic text and only scrub credential material that
/// may have leaked into it (e.g. a connection string with an embedded
/// password). This preserves useful errors such as "password authentication
/// failed for user \"x\"" while still redacting the secret value.
pub(super) fn sanitize_pg_error(error: sqlx::Error) -> AppError {
    let msg = error.to_string();
    let safe = redact_connection_string(&msg);
    if safe == msg {
        AppError::Database(error)
    } else {
        AppError::Database(sqlx::Error::Protocol(safe))
    }
}

/// Sanitize an AppError from a helper that already wraps sqlx errors.
pub(super) fn sanitize_pg_app_error(error: AppError) -> AppError {
    match error {
        AppError::Database(sqlx_err) => {
            let msg = sqlx_err.to_string();
            let safe = redact_connection_string(&msg);
            if safe == msg {
                AppError::Database(sqlx_err)
            } else {
                AppError::Database(sqlx::Error::Protocol(safe))
            }
        }
        other => other,
    }
}
