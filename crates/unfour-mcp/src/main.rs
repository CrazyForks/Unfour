use std::io;
use std::sync::Arc;

use unfour_mcp::{LocalCommandBusAdapter, Shutdown};

fn main() {
    let _logging_guard = initialize_logging();

    // Unified shutdown signal shared between the stdio loop and the signal
    // handlers. The first trigger wins; every observer sees the same value.
    let shutdown = Shutdown::new();

    let stdin = io::stdin();
    let stdout = io::stdout();

    let adapter = match LocalCommandBusAdapter::default_storage() {
        Ok(adapter) => adapter,
        Err(error) => {
            eprintln!(
                "unfour-mcp failed to initialize command bus: {}: {}",
                error.code, error.message
            );
            std::process::exit(1);
        }
    };

    // Install Ctrl+C / SIGTERM handlers. The stdio read is a blocking syscall
    // that cannot be interrupted from another thread, so on a signal we release
    // background tasks (bounded) and then hard-exit the whole process.
    install_signal_handlers(shutdown.clone(), adapter.clone());

    let result = unfour_mcp::run_stdio_with_adapter(adapter.clone(), stdin.lock(), stdout.lock());

    // Normal exit path: EOF on stdin or a clean client disconnect. `run_stdio_with_adapter`
    // already shut the runtime down; mark the signal for completeness.
    shutdown.trigger();

    match result {
        Ok(()) => {}
        Err(error) => {
            // A broken stdout pipe (client already gone) is an expected shutdown,
            // not a failure. The loop already returns `Ok` for it; guard here too.
            if error.kind() == io::ErrorKind::BrokenPipe {
                return;
            }
            eprintln!("unfour-mcp stdio server failed: {error}");
            std::process::exit(1);
        }
    }
}

/// Spawn a tiny dedicated tokio runtime on a thread that only waits for the
/// termination signals, so the blocking stdio loop on the main thread is never
/// disturbed. On a signal we trigger the shared [`Shutdown`] flag, perform a
/// bounded release of background tasks, then exit the whole process.
fn install_signal_handlers(shutdown: Shutdown, adapter: Arc<LocalCommandBusAdapter>) {
    #[cfg(unix)]
    {
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("unfour-mcp signal runtime");
            runtime.block_on(async {
                let mut sigterm = tokio::signal::unix::signal(
                    tokio::signal::unix::SignalKind::terminate(),
                )
                .expect("install SIGTERM handler");
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = sigterm.recv() => {}
                }
                shutdown.trigger();
                adapter.shutdown();
                std::process::exit(0);
            });
        });
    }
    #[cfg(windows)]
    {
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("unfour-mcp signal runtime");
            runtime.block_on(async {
                let _ = tokio::signal::ctrl_c().await;
                shutdown.trigger();
                adapter.shutdown();
                std::process::exit(0);
            });
        });
    }
}

fn initialize_logging() -> Option<unfour_diag::LoggingGuard> {
    let paths = unfour_paths::initialize_unfour_storage().ok()?;
    let mut config = unfour_diag::LoggingConfig::oss_dev(paths.logs_dir);
    config.app_name = "unfour-mcp".to_string();
    config.version = env!("CARGO_PKG_VERSION").to_string();
    unfour_diag::init_logging(config).ok()
}
