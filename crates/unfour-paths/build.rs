fn main() {
    // Optional compile-time storage profile hint. When unset, runtime defaults
    // to stable (`~/.unfour`). Only accepted values are forwarded so a typo
    // does not bake an unknown channel into the crate.
    if let Ok(channel) = std::env::var("UNFOUR_RELEASE_CHANNEL") {
        let channel = channel.trim().to_ascii_lowercase();
        if matches!(channel.as_str(), "stable" | "test" | "dev") {
            println!("cargo:rustc-env=UNFOUR_RELEASE_CHANNEL={channel}");
        }
    }
    println!("cargo:rerun-if-env-changed=UNFOUR_RELEASE_CHANNEL");
}
