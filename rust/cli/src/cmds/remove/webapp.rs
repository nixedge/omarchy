use crate::desktop;
use std::fs;
use std::process::Command;

fn is_webapp_desktop(path: &std::path::Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if line.starts_with("Exec=")
            && (line.contains("omarchy-launch-webapp") || line.contains("omarchy-webapp-handler"))
        {
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}

fn icon_name_for(app_name: &str) -> String {
    desktop::safe_icon_name(app_name)
}

fn remove_webapp_files(app_name: &str, desktop_file: Option<&str>) {
    let app_dir = desktop::user_app_dir();
    let icon_dir = desktop::user_icon_dir();
    let old_icon_dir = desktop::home_dir().join(".local/share/applications/icons");

    let icon_name = icon_name_for(app_name);
    let fallback = app_dir.join(format!("{app_name}.desktop"));
    let desktop_path = desktop_file
        .map(std::path::PathBuf::from)
        .unwrap_or(fallback);
    let _ = fs::remove_file(&desktop_path);
    let _ = fs::remove_file(icon_dir.join(format!("{icon_name}.png")));
    let _ = fs::remove_file(icon_dir.join(format!("{app_name}.png")));
    let _ = fs::remove_file(old_icon_dir.join(format!("{app_name}.png")));
}

pub fn remove(name: Option<&str>, notify: bool) -> i32 {
    let app_dir = desktop::user_app_dir();

    // Build index of web apps
    let mut web_apps: Vec<String> = Vec::new();
    let mut web_app_paths: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&app_dir) {
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("desktop"))
            .collect();
        files.sort_by_key(|e| e.path());
        for entry in files {
            let path = entry.path();
            if let Some(found_path) = is_webapp_desktop(&path) {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_owned();
                web_apps.push(stem);
                web_app_paths.push(found_path);
            }
        }
    }

    let (app_name, desktop_file) = if let Some(n) = name {
        // Find matching path
        let idx = web_apps.iter().position(|s| s == n);
        (n.to_owned(), idx.map(|i| web_app_paths[i].clone()))
    } else {
        if web_apps.is_empty() {
            println!("No web apps to remove.");
            return 1;
        }
        let options: Vec<&str> = web_apps.iter().map(|s| s.as_str()).collect();
        match desktop::gum_choose(&options, "Select web app to remove") {
            Some(mut v) => {
                let chosen = v.remove(0);
                let idx = web_apps.iter().position(|s| *s == chosen);
                (chosen, idx.map(|i| web_app_paths[i].clone()))
            }
            None => {
                println!("You must select a web app to remove.");
                return 1;
            }
        }
    };

    if app_name.is_empty() {
        eprintln!("You must select a web app to remove.");
        return 1;
    }

    remove_webapp_files(&app_name, desktop_file.as_deref());

    if notify {
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", "", "Web app removed", &app_name])
            .status();
    }

    let _ = Command::new("update-desktop-database")
        .arg(&app_dir)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status();

    0
}

pub fn remove_all() {
    let app_dir = desktop::user_app_dir();
    let icon_dir = desktop::user_icon_dir();
    let old_icon_dir = desktop::home_dir().join(".local/share/applications/icons");

    println!("Scanning for web apps in {}...", app_dir.display());

    let entries = match fs::read_dir(&app_dir) {
        Ok(e) => e,
        Err(_) => { println!("No web apps found."); return; }
    };

    let mut found = false;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("desktop") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if !content.lines().any(|l| {
            l.starts_with("Exec=omarchy-launch-webapp") || l.starts_with("Exec=omarchy-webapp-handler")
        }) {
            continue;
        }

        let app_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let icon_name = desktop::safe_icon_name(app_name);
        println!("Removing web app: {app_name}");
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(icon_dir.join(format!("{icon_name}.png")));
        let _ = fs::remove_file(icon_dir.join(format!("{app_name}.png")));
        let _ = fs::remove_file(old_icon_dir.join(format!("{app_name}.png")));
        found = true;
    }

    if !found {
        println!("No web apps found.");
    }

    let _ = Command::new("update-desktop-database")
        .arg(&app_dir)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status();
}
