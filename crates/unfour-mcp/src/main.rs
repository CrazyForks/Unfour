use std::io;

fn main() {
    let _logging_guard = initialize_logging();
    let stdin = io::stdin();
    let stdout = io::stdout();

    if let Err(error) = unfour_mcp::run_stdio(stdin.lock(), stdout.lock()) {
        eprintln!("unfour-mcp stdio server failed: {error}");
        std::process::exit(1);
    }
}

fn initialize_logging() -> Option<unfour_diag::LoggingGuard> {
    let paths = unfour_paths::initialize_unfour_storage().ok()?;
    let mut config = unfour_diag::LoggingConfig::oss_dev(paths.logs_dir);
    config.app_name = "unfour-mcp".to_string();
    config.version = env!("CARGO_PKG_VERSION").to_string();
    unfour_diag::init_logging(config).ok()
}
