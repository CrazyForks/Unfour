use serde::Serialize;
use unfour_core::AppResult;

/// Whether the running app is a debug/dev build or a release/installed build.
///
/// Surfaced to the UI so it can tailor the "binary not found" guidance:
/// dev builds tell the user how to compile the sidecar, release builds tell
/// them to reinstall.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpBuildKind {
    Dev,
    Release,
}

/// Resolved location of the `unfour-mcp` sidecar binary for external MCP
/// clients (Codex/Claude/Cursor). The path is dynamic: it points at wherever
/// the current executable (and its bundled resources) actually live.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpBinaryPathResult {
    /// Absolute path the external MCP client should invoke.
    pub path: String,
    /// Whether a runnable binary actually exists at `path`.
    pub found: bool,
    /// Build kind, so the UI can tailor its guidance.
    pub build_kind: McpBuildKind,
}

#[tauri::command]
pub fn mcp_binary_path() -> AppResult<McpBinaryPathResult> {
    let build_kind = if cfg!(debug_assertions) {
        McpBuildKind::Dev
    } else {
        McpBuildKind::Release
    };

    Ok(resolve_mcp_binary_path(build_kind))
}

/// Plain runnable name (no target triple), used for the dev `target/debug`
/// sibling and the intuitive "next to the app" release layout.
fn binary_name() -> String {
    if cfg!(windows) {
        "unfour-mcp.exe".to_string()
    } else {
        "unfour-mcp".to_string()
    }
}

/// Tauri `externalBin` name, which carries the full target triple.
fn sidecar_name() -> String {
    let ext = if cfg!(windows) { ".exe" } else { "" };
    format!("unfour-mcp-{}{}", target_triple(), ext)
}

#[allow(clippy::needless_return)]
fn target_triple() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "x86_64-pc-windows-msvc";
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return "aarch64-pc-windows-msvc";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "x86_64-apple-darwin";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "aarch64-apple-darwin";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "x86_64-unknown-linux-gnu";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "aarch64-unknown-linux-gnu";
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
    )))]
    return "unknown";
}

fn current_exe_dir() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(|p| p.to_path_buf())
}

fn resolve_mcp_binary_path(build_kind: McpBuildKind) -> McpBinaryPathResult {
    let recommended = current_exe_dir()
        .map(|dir| dir.join(binary_name()))
        .unwrap_or_else(|| std::path::PathBuf::from(binary_name()));

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(dir) = current_exe_dir() {
        // Dev build sibling and the intuitive "next to the app" release layout.
        candidates.push(dir.join(binary_name()));
        // Tauri v2 sidecar bundling: <app>/resources/bin/<name>-<triple>[.exe].
        candidates.push(dir.join("resources").join("bin").join(sidecar_name()));
        // macOS: <app>/../Resources/<name>-<triple>.
        candidates.push(dir.join("..").join("Resources").join(sidecar_name()));
        // Dev `tauri dev` prepared externalBin: <target>/<profile>/../../src-tauri/binaries.
        candidates.push(
            dir.join("..")
                .join("..")
                .join("src-tauri")
                .join("binaries")
                .join(sidecar_name()),
        );
    }

    for candidate in &candidates {
        if candidate.is_file() {
            return McpBinaryPathResult {
                path: candidate.to_string_lossy().to_string(),
                found: true,
                build_kind,
            };
        }
    }

    McpBinaryPathResult {
        path: recommended.to_string_lossy().to_string(),
        found: false,
        build_kind,
    }
}
