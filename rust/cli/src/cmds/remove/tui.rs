use crate::desktop;
use std::fs;
use std::process::Command;

fn is_tui_desktop(path: &std::path::Path) -> bool {
    fs::read_to_string(path)
        .map(|s| {
            s.lines().any(|l| {
                let l = l.trim();
                l.starts_with("Exec=") && (l.contains("$TERMINAL") || l.contains("xdg-terminal-exec")) && l.contains("-e")
            })
        })
        .unwrap_or(false)
}

fn icon_name_for(app_name: &str) -> String {
    desktop::safe_icon_name(app_name)
}

fn remove_tui_files(app_name: &str) {
    let app_dir = desktop::user_app_dir();
    let icon_dir = desktop::user_icon_dir();
    let old_icon_dir = desktop::home_dir().join(".local/share/applications/icons");

    let icon_name = icon_name_for(app_name);
    let _ = fs::remove_file(app_dir.join(format!("{app_name}.desktop")));
    let _ = fs::remove_file(icon_dir.join(format!("{icon_name}.png")));
    let _ = fs::remove_file(icon_dir.join(format!("{app_name}.png")));
    let _ = fs::remove_file(old_icon_dir.join(format!("{app_name}.png")));
}

pub fn remove(name: Option<&str>, notify: bool) -> i32 {
    let app_dir = desktop::user_app_dir();

    let app_name = if let Some(n) = name {
        n.to_owned()
    } else {
        // Interactive: scan for TUI launchers
        let mut tuis: Vec<String> = Vec::new();
        if let Ok(entries) = fs::read_dir(&app_dir) {
            let mut files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("desktop"))
                .collect();
            files.sort_by_key(|e| e.path());
            for entry in files {
                let path = entry.path();
                if is_tui_desktop(&path) {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        tuis.push(stem.to_owned());
                    }
                }
            }
        }

        if tuis.is_empty() {
            println!("No TUIs to remove.");
            return 1;
        }

        let options: Vec<&str> = tuis.iter().map(|s| s.as_str()).collect();
        match desktop::gum_choose(&options, "Select TUI to remove") {
            Some(mut v) => v.remove(0),
            None => {
                println!("You must select a TUI to remove.");
                return 1;
            }
        }
    };

    if app_name.is_empty() {
        eprintln!("You must select a TUI to remove.");
        return 1;
    }

    remove_tui_files(&app_name);

    if notify {
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", "", "TUI removed", &app_name])
            .status();
    }

    0
}

pub fn remove_all() {
    let app_dir = desktop::user_app_dir();
    let icon_dir = desktop::user_icon_dir();
    let old_icon_dir = desktop::home_dir().join(".local/share/applications/icons");

    println!("Scanning for TUIs in {}...", app_dir.display());

    let entries = match fs::read_dir(&app_dir) {
        Ok(e) => e,
        Err(_) => { println!("No TUIs found."); return; }
    };

    let mut found = false;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("desktop") {
            continue;
        }
        // Only remove TUIs created by omarchy-tui-install (app-id=TUI.)
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if !content.lines().any(|l| l.starts_with("Exec=xdg-terminal-exec --app-id=TUI.")) {
            continue;
        }

        let app_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let icon_name = desktop::safe_icon_name(app_name);
        println!("Removing TUI: {app_name}");
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(icon_dir.join(format!("{icon_name}.png")));
        let _ = fs::remove_file(icon_dir.join(format!("{app_name}.png")));
        let _ = fs::remove_file(old_icon_dir.join(format!("{app_name}.png")));
        found = true;
    }

    if !found {
        println!("No TUIs found.");
    }

    let _ = Command::new("update-desktop-database")
        .arg(&app_dir)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status();
}
