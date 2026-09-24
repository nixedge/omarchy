pub mod ai;
pub mod browser;
pub mod devenv;
pub mod gaming;
pub mod launcher_entry;
pub mod preinstalls;
pub mod tui;
pub mod voxtype;
pub mod webapp;

use std::fs;

/// Remove dirs/files silently, ignoring errors.
pub fn rm_rf(paths: &[&str]) {
    let home = std::env::var("HOME").unwrap_or_default();
    for path in paths {
        let expanded = path.replace("$HOME", &home).replace("~/", &format!("{home}/"));
        let _ = fs::remove_dir_all(&expanded);
        let _ = fs::remove_file(&expanded);
    }
}

pub fn home_path(rel: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::PathBuf::from(home).join(rel)
}
