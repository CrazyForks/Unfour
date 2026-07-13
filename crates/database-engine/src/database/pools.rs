use super::*;

impl DatabaseService {
    /// Create a PostgreSQL connection pool, loading the password from SecretStore
    /// if a credential reference is present on the connection.
    pub(super) async fn postgres_pool(
        &self,
        connection: &DatabaseConnection,
    ) -> AppResult<sqlx::PgPool> {
        self.postgres_pool_with_secret(connection, None).await
    }

    /// PostgreSQL pool that prefers an inline `password_override` (the
    /// not-yet-saved secret from the "test connection" dialog) over the stored
    /// keychain credential. Falls back to the saved credential when no override
    /// is supplied, preserving existing behavior for saved connections.
    pub(super) async fn postgres_pool_with_secret(
        &self,
        connection: &DatabaseConnection,
        password_override: Option<&str>,
    ) -> AppResult<sqlx::PgPool> {
        let options =
            pg_connect_options(connection, self.secret_store.as_ref(), password_override).await?;
        PgPoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .map_err(|e| sanitize_pg_error(e))
    }

    /// Create a MySQL connection pool, loading the password from SecretStore
    /// if a credential reference is present on the connection.
    pub(super) async fn mysql_pool(
        &self,
        connection: &DatabaseConnection,
    ) -> AppResult<sqlx::MySqlPool> {
        self.mysql_pool_with_secret(connection, None).await
    }

    /// MySQL pool that prefers an inline `password_override` (the not-yet-saved
    /// secret from the "test connection" dialog) over the stored keychain
    /// credential. Falls back to the saved credential when no override is
    /// supplied, preserving existing behavior for saved connections.
    pub(super) async fn mysql_pool_with_secret(
        &self,
        connection: &DatabaseConnection,
        password_override: Option<&str>,
    ) -> AppResult<sqlx::MySqlPool> {
        let options =
            mysql_connect_options(connection, self.secret_store.as_ref(), password_override)
                .await?;
        MySqlPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options)
            .await
            .map_err(sanitize_mysql_error)
    }

    /// Return a connection clone with `database` overridden to the given
    /// catalog when the catalog differs from the connection's current database.
    /// This is required for PostgreSQL (and MySQL) because they cannot
    /// cross-database query; the pool must target the database that owns the
    /// table being browsed, inspected, or mutated.
    pub(super) fn effective_connection(
        connection: &DatabaseConnection,
        catalog: Option<&str>,
    ) -> DatabaseConnection {
        match catalog {
            Some(name) if connection.database.as_deref() != Some(name) => {
                let mut overridden = connection.clone();
                overridden.database = Some(name.to_string());
                overridden
            }
            _ => connection.clone(),
        }
    }
}
