//! Shared Tauri composition layer.
//!
//! This crate owns everything in the app shell that is edition-independent:
//! the shared plugins, the command-bus setup, the managed [`AppState`], and the
//! `commands` adapters. Each edition's binary (core `apps/desktop`, Pro
//! `apps/desktop-pro`) supplies only its own `invoke_handler!` list and
//! `generate_context!` — which are compile-time, per-binary concerns — and
//! delegates the rest to [`configure`].

pub mod commands;

use tauri::Manager;
use unfour_command_bus::CommandBus;
use unfour_local_storage::LocalDb;
use unfour_secret_store::SecretStore;

pub struct AppState {
    pub command_bus: CommandBus,
}

/// Apply the shared plugins and command-bus setup to a Tauri builder.
///
/// The caller is responsible for the per-edition tail of the chain:
/// `.invoke_handler(tauri::generate_handler![..])` and
/// `.run(tauri::generate_context!())`.
pub fn configure(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let command_bus = tauri::async_runtime::block_on(async {
                let db = LocalDb::connect(&app_handle).await?;
                db.migrate().await?;
                CommandBus::from_db_with_secret_store(db, SecretStore::new("unfour-workspace"))
                    .await
            })?;

            #[cfg(feature = "ssh-native")]
            {
                let event_app = app_handle.clone();
                command_bus.set_terminal_output_callback(std::sync::Arc::new(move |payload| {
                    use tauri::Emitter;
                    if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&payload) {
                        let _ = event_app.emit("ssh://terminal-data", payload);
                    }
                }));
            }

            app.manage(AppState { command_bus });
            Ok(())
        })
}
