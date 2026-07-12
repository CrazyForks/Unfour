use std::path::PathBuf;
use std::process::Command;

fn main() {
    tauri_build::build();

    #[cfg(target_os = "windows")]
    {
        let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        println!("cargo:rustc-link-search=native={}", out_dir.display());
    }

    // Release channel. Formal CI must set `UNFOUR_RELEASE_CHANNEL` explicitly
    // (Test for pre-releases, Stable for formal releases). Local/dev builds
    // default to "test". The channel is NEVER inferred from the cargo profile
    // or `debug_assertions`; only these two values are accepted.
    let channel = std::env::var("UNFOUR_RELEASE_CHANNEL")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| value == "test" || value == "stable")
        .unwrap_or_else(|| "test".to_string());
    println!("cargo:rustc-env=UNFOUR_RELEASE_CHANNEL={channel}");

    // Build commit. Precedence:
    //   1. Explicit `UNFOUR_BUILD_COMMIT` override.
    //   2. CI's `GITHUB_SHA`.
    //   3. The actual HEAD of the checked-out workspace (`git rev-parse HEAD`),
    //      suffixed with `-dirty` if the working tree has local modifications.
    // If git is unavailable we fall back to "unknown" so ordinary dev builds
    // never fail. This always reflects the commit actually being built, not the
    // latest remote commit.
    let commit = resolve_build_commit();
    println!("cargo:rustc-env=UNFOUR_BUILD_COMMIT={commit}");

    println!("cargo:rerun-if-env-changed=UNFOUR_RELEASE_CHANNEL");
    println!("cargo:rerun-if-env-changed=UNFOUR_BUILD_COMMIT");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");

    // Rebuild when HEAD moves so the embedded commit stays accurate.
    if let Some(git_dir) = locate_git_dir() {
        let head = git_dir.join("HEAD");
        if head.exists() {
            println!("cargo:rerun-if-changed={}", head.display());
        }
        let packed_refs = git_dir.join("packed-refs");
        if packed_refs.exists() {
            println!("cargo:rerun-if-changed={}", packed_refs.display());
        }
        let refs_heads = git_dir.join("refs").join("heads");
        if refs_heads.exists() {
            println!("cargo:rerun-if-changed={}", refs_heads.display());
        }
    }
}

fn resolve_build_commit() -> String {
    if let Ok(explicit) = std::env::var("UNFOUR_BUILD_COMMIT") {
        let explicit = explicit.trim();
        if !explicit.is_empty() {
            return explicit.to_string();
        }
    }
    if let Ok(github_sha) = std::env::var("GITHUB_SHA") {
        let github_sha = github_sha.trim();
        if !github_sha.is_empty() {
            return github_sha.to_string();
        }
    }
    resolve_git_head().unwrap_or_else(|| "unknown".to_string())
}

fn resolve_git_head() -> Option<String> {
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|sha| !sha.is_empty())?;

    // Mark dirty working trees so support can tell an exact release build apart
    // from a locally modified one. Formal CI publishes from a clean checkout.
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);

    if dirty {
        Some(format!("{sha}-dirty"))
    } else {
        Some(sha)
    }
}

fn locate_git_dir() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--absolute-git-dir"])
        .output()
        .ok()
        .filter(|output| output.status.success())?;
    let dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if dir.is_empty() {
        None
    } else {
        Some(PathBuf::from(dir))
    }
}
