use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value;
use sqlx::SqliteConnection;
use unfour_core::domain::{CommandContext, DomainCommandResult, DomainMutation, MutationOrigin};
use unfour_core::AppResult;
use unfour_local_storage::ActivityLogService;

use crate::CommandBus;

pub trait TransactionalCommandHook: Send + Sync {
    /// Runs inside the Command Bus-owned transaction. Implementations must not
    /// commit the connection or perform network requests.
    fn on_mutations<'a>(
        &'a self,
        connection: &'a mut SqliteConnection,
        context: &'a CommandContext,
        mutations: &'a [DomainMutation],
    ) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;
}

#[derive(Clone, Default)]
pub struct CommandBusExtensions {
    transactional_hooks: Arc<[Arc<dyn TransactionalCommandHook>]>,
}

impl CommandBusExtensions {
    pub fn new(transactional_hooks: Vec<Arc<dyn TransactionalCommandHook>>) -> Self {
        Self {
            transactional_hooks: transactional_hooks.into(),
        }
    }

    pub fn transactional_hooks(&self) -> &[Arc<dyn TransactionalCommandHook>] {
        &self.transactional_hooks
    }
}

pub(crate) struct CommandActivity {
    pub workspace_id: Option<String>,
    pub action: &'static str,
    pub target: Option<String>,
    pub details: Value,
}

pub(crate) type CommandExecutorFuture<'a, T> =
    Pin<Box<dyn Future<Output = AppResult<DomainCommandResult<T>>> + Send + 'a>>;

impl CommandBus {
    pub(crate) async fn execute_domain_command<T, F>(
        &self,
        context: CommandContext,
        activity: Option<CommandActivity>,
        executor: F,
    ) -> AppResult<T>
    where
        T: Send,
        F: for<'a> FnOnce(&'a mut SqliteConnection) -> CommandExecutorFuture<'a, T>,
    {
        let mut transaction = self.db.pool().begin().await?;
        let outcome = executor(&mut transaction).await?;

        if !outcome.mutations.is_empty() {
            if let Some(activity) = activity {
                // Create commands do not know their generated entity id until
                // the executor returns. Derive only missing local activity
                // scope from a single mutation; bulk, migration, and external
                // activities retain their explicitly supplied scope.
                let created_entity = (context.origin == MutationOrigin::Local
                    && outcome.mutations.len() == 1)
                    .then(|| &outcome.mutations[0].entity);
                let workspace_id = activity
                    .workspace_id
                    .as_deref()
                    .or_else(|| created_entity.map(|entity| entity.workspace_id.as_str()));
                let target = activity
                    .target
                    .as_deref()
                    .or_else(|| created_entity.map(|entity| entity.entity_id.as_str()));
                ActivityLogService::record_on(
                    &mut transaction,
                    workspace_id,
                    activity.action,
                    target,
                    activity.details,
                )
                .await?;
            }
            for hook in self.extensions.transactional_hooks() {
                hook.on_mutations(&mut transaction, &context, &outcome.mutations)
                    .await?;
            }
        }

        transaction.commit().await?;
        Ok(outcome.value)
    }
}
