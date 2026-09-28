use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_desktop_file(desktop_id: &str) -> Option<PathBuf> {
    let filename = if desktop_id.ends_with(".desktop") {
        desktop_id.to_owned()
    } else {
        format!("{desktop_id}.desktop")
    };

    let home = std::env::var("HOME").unwrap_or_default();
    let xdg_data_home = std::env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| format!("{home}/.local/share"));

    let mut search_dirs: Vec<PathBuf> = vec![
        PathBuf::from(&xdg_data_home).join("applications"),
    ];

    let system_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_owned());
    for dir in system_dirs.split(':') {
        search_dirs.push(PathBuf::from(dir).join("applications"));
    }

    for dir in &search_dirs {
        let candidate = dir.join(&filename);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn is_user_desktop_file(path: &Path) -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let xdg_data_home = std::env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| format!("{home}/.local/share"));
    let user_app_dir = PathBuf::from(&xdg_data_home).join("applications");
    path.parent().map(|p| p == user_app_dir).unwrap_or(false)
}

fn exec_line(path: &Path) -> String {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Exec="))
                .map(|l| l[5..].to_owned())
        })
        .unwrap_or_default()
}

pub fn run(desktop_id: &str, entry_name: Option<&str>) -> i32 {
    let desktop_file = match find_desktop_file(desktop_id) {
        Some(p) => p,
        None => {
            eprintln!(
                "Could not find launcher entry: {}.desktop",
                desktop_id.trim_end_matches(".desktop")
            );
            return 1;
        }
    };

    let exec = exec_line(&desktop_file);
    let basename = desktop_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(desktop_id);

    // Web app
    if exec.contains("omarchy-launch-webapp") || exec.contains("omarchy-webapp-handler") {
        let notify = std::env::var("OMARCHY_REMOVE_NOTIFY")
            .map(|v| v != "false")
            .unwrap_or(true);
        return crate::cmds::remove::webapp::remove(Some(basename), notify);
    }

    // TUI
    let is_tui = (exec.contains("$TERMINAL") || exec.contains("xdg-terminal-exec"))
        && exec.contains("-e");
    if is_tui {
        let notify = std::env::var("OMARCHY_REMOVE_NOTIFY")
            .map(|v| v != "false")
            .unwrap_or(true);
        return crate::cmds::remove::tui::remove(Some(basename), notify);
    }

    // User-created desktop file
    if is_user_desktop_file(&desktop_file) {
        let _ = fs::remove_file(&desktop_file);
        let parent = desktop_file.parent().unwrap_or(Path::new("."));
        let _ = Command::new("update-desktop-database")
            .arg(parent)
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .status();
        return 0;
    }

    // System package desktop file — on NixOS, packages come from the Nix store.
    // We can't use pacman -Qqo to identify the owning package. If the desktop file
    // is from a package tracked by the omarchy daemon, use `omarchy pkg drop`.
    // For Nix-store-owned desktop files we can't automatically identify the package;
    // let the user know.
    let display_name = entry_name.unwrap_or(basename);
    let path_str = desktop_file.to_string_lossy();
    if path_str.contains("/nix/store/") || path_str.contains("/run/current-system/") {
        eprintln!(
            "'{display_name}' is a system package desktop entry managed by NixOS.\n\
             To remove it, use:\n  omarchy pkg drop <package-name>"
        );
        return 1;
    }

    // Flatpak
    let flatpak_id = desktop_id.trim_end_matches(".desktop");
    let flatpak_present = Command::new("flatpak")
        .args(["info", flatpak_id])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if flatpak_present {
        let status = Command::new("flatpak")
            .args(["uninstall", flatpak_id])
            .status();
        return status.map(|s| if s.success() { 0 } else { 1 }).unwrap_or(1);
    }

    eprintln!("Don't know how to uninstall {}.desktop", desktop_id.trim_end_matches(".desktop"));
    1
}
