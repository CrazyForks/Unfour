// Tauri's resource selects Common Controls v6 before the Windows test harness starts.
#[cfg(all(test, target_os = "windows"))]
#[link(name = "resource", kind = "static")]
unsafe extern "C" {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Community identity is stated explicitly here — the single, compile-time
    // source of truth. `channel` and `commit` are injected by `build.rs`
    // (see UNFOUR_RELEASE_CHANNEL / UNFOUR_BUILD_COMMIT); nothing is inferred
    // from the cargo profile or `debug_assertions` at runtime.
    let config = unfour_app::UnfourAppConfig {
        edition: unfour_app::AppEdition::Community,
        app_name: "Unfour".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        package_kind: unfour_app::PackageKind::GitHub,
        channel: build_channel(),
        commit: build_commit(),
        // `core_commit` defaults to `commit` (see `normalize_config` in
        // `unfour-app`). A future Pro binary may set a distinct core commit.
        core_commit: None,
    };

    unfour_app::configure_core_app(tauri::Builder::default(), config)
        .invoke_handler(unfour_app::generate_handlers![])
        .run(tauri::generate_context!())
        .expect("error while running Unfour");
}

/// Release channel injected at build time by `build.rs`. Only "test" and
/// "stable" are emitted; anything else defaults to Test.
fn build_channel() -> unfour_app::ReleaseChannel {
    match env!("UNFOUR_RELEASE_CHANNEL") {
        "stable" => unfour_app::ReleaseChannel::Stable,
        _ => unfour_app::ReleaseChannel::Test,
    }
}

/// HEAD SHA of the workspace this binary was built from, injected by `build.rs`.
/// An empty or "unknown" value (git unavailable) becomes `None`.
fn build_commit() -> Option<String> {
    match env!("UNFOUR_BUILD_COMMIT") {
        "" | "unknown" => None,
        value => Some(value.to_string()),
    }
}
