pub const WINDOW_APP_ID: &str = "oneforall_app";
pub const WINDOW_TITLE: &str = "OneForAllLauncher";

// Only this fork's signed GitHub release packages are trusted.
pub const UPDATER_ENDPOINT: &str =
    "https://github.com/liwwyy/OneForAllLauncher/releases/latest/download/latest.json";
pub const UPDATER_PUBKEY: &str = include_str!("../assets/updater.pub");
pub const RELEASES_URL: &str = "https://github.com/liwwyy/OneForAllLauncher/releases/latest";
pub const FORK_NOTICE: &str = "OneForAllLauncher, a fork of OneClient by Polyfrost. Modified 2026-10-06: independent branding and offline accounts. GPL-3.0-only; no warranty.";
