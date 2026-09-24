use crate::desktop;
use crate::cmds::install::remove_many;
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
        "Are you sure you want to remove all preinstalled web apps, TUI wrappers, and desktop applications?",
    ) {
        return 0;
    }

    println!("Removing preinstalled Omarchy applications...\n");

    // Remove web apps and TUI launchers
    crate::cmds::remove::webapp::remove_all();
    crate::cmds::remove::tui::remove_all();

    // Mark preinstalls as removed
    let state_dir = desktop::home_dir().join(".local/state/omarchy");
    let _ = std::fs::create_dir_all(&state_dir);
    let _ = std::fs::write(state_dir.join("preinstalls-removed"), "");

    let _ = Command::new("hyprctl").arg("reload").status();

    // Remove mise stubs
    let home = std::env::var("HOME").unwrap_or_default();
    for stub in &[
        "codex", "claude", "agy", "copilot", "gh", "opencode",
        "playwright", "playwright-cli", "pi", "omp", "ori", "grok",
        "crush", "ghui", "hunk",
    ] {
        let _ = std::fs::remove_file(format!("{home}/.local/bin/{stub}"));
    }

    // Remove cursor-agent if it's an Omarchy-managed mise stub
    let cursor_agent = format!("{home}/.local/bin/cursor-agent");
    if let Ok(content) = std::fs::read_to_string(&cursor_agent) {
        let is_regular = !std::path::Path::new(&cursor_agent)
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        if is_regular && content.contains("mise use -g") && content.contains("cursor-agent") {
            let _ = std::fs::remove_file(&cursor_agent);
        }
    }

    // Remove muse if it's an Omarchy-managed mise stub
    let muse = format!("{home}/.local/bin/muse");
    if let Ok(content) = std::fs::read_to_string(&muse) {
        let is_regular = !std::path::Path::new(&muse)
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        if is_regular && content.contains("mise use -g") && content.contains("http:muse[") {
            let _ = std::fs::remove_file(&muse);
        }
    }

    // Remove hermes CLI if owned by Omarchy
    let hermes_owned = Command::new("omarchy-install-hermes-cli")
        .arg("--owns")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if hermes_owned {
        let _ = std::fs::remove_file(format!("{home}/.local/bin/hermes"));
    }

    remove_many(PREINSTALL_PKGS)
}
