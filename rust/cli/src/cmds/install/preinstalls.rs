use crate::desktop;
use super::add_many;
use std::process::Command;

const PREINSTALL_PKGS: &[&str] = &[
    "aether",
    "cliamp",
    "libreoffice-fresh",
    "xournalpp",
    "pinta",
    "obsidian",
    "obs-studio",
    "kdenlive",
    "moonlight-qt",
    "lazydocker",
    "omacut",
    "omacalc",
    "omawrite",
];

pub fn run() -> i32 {
    if !desktop::gum_confirm(
        "Are you sure you want to restore all preinstalled web apps, TUI wrappers, and desktop applications?",
    ) {
        return 0;
    }

    println!("Restoring preinstalled Omarchy applications...\n");

    // Recreate shipped launchers and mise stubs
    let _ = Command::new("omarchy-refresh-applications").status();

    let rc = add_many(PREINSTALL_PKGS, true);
    if rc != 0 {
        println!("\nPreinstalls are still marked as removed. Fix the errors above and try again.");
        return rc;
    }

    // Clear the opt-out marker
    let state_file = desktop::home_dir().join(".local/state/omarchy/preinstalls-removed");
    let _ = std::fs::remove_file(&state_file);

    let _ = Command::new("hyprctl").arg("reload").status();
    0
}
