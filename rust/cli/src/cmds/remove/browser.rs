use crate::cli::BrowserName;
use crate::cmds::drop;
use std::fs;
use std::process::Command;

pub fn remove(name: BrowserName) -> i32 {
    match name {
        BrowserName::Chromium => {
            println!("Removing Chromium…");
            // Chromium is often the fallback default, so don't try to replace it
            drop::run("chromium")
        }
        BrowserName::Chrome => {
            println!("Removing Chrome…");
            set_fallback_default("google-chrome.desktop");
            let rc = drop::run("google-chrome");
            if rc != 0 { return rc; }
            let _ = fs::remove_file(home_config("chrome-flags.conf"));
            let _ = sudo_rm("/etc/opt/chrome/policies/managed/color.json");
            0
        }
        BrowserName::Edge => {
            println!("Removing Edge…");
            set_fallback_default("microsoft-edge.desktop");
            let rc = drop::run("microsoft-edge");
            if rc != 0 { return rc; }
            let _ = fs::remove_file(home_config("microsoft-edge-stable-flags.conf"));
            let _ = sudo_rm("/etc/opt/edge/policies/managed/color.json");
            0
        }
        BrowserName::Brave => {
            println!("Removing Brave…");
            set_fallback_default("brave-browser.desktop");
            let rc = drop::run("brave");
            if rc != 0 { return rc; }
            let _ = fs::remove_file(home_config("brave-flags.conf"));
            0
        }
        BrowserName::BraveOrigin => {
            println!("Removing Brave Origin…");
            set_fallback_default("brave-origin.desktop");
            let rc = drop::run("brave");
            if rc != 0 { return rc; }
            let _ = fs::remove_file(home_config("brave-origin-flags.conf"));
            0
        }
        BrowserName::Firefox => {
            println!("Removing Firefox…");
            set_fallback_default("firefox.desktop");
            drop::run("firefox")
        }
        BrowserName::Zen => {
            println!("Removing Zen…");
            set_fallback_default("zen.desktop");
            drop::run("zen-browser")
        }
    }
}

fn home_config(rel: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::PathBuf::from(home).join(".config").join(rel)
}

fn set_fallback_default(current_desktop: &str) {
    let output = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .env_remove("BROWSER")
        .output();

    if let Ok(out) = output {
        let current = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        if current == current_desktop {
            // Try to fall back to chromium if available
            if Command::new("which").arg("chromium").output().map(|o| o.status.success()).unwrap_or(false) {
                let _ = Command::new("xdg-settings")
                    .args(["set", "default-web-browser", "chromium.desktop"])
                    .env_remove("BROWSER")
                    .status();
            }
        }
    }
}

fn sudo_rm(path: &str) -> bool {
    Command::new("sudo").args(["rm", "-f", path]).status()
        .map(|s| s.success()).unwrap_or(false)
}
