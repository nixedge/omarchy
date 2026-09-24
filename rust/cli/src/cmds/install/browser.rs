use crate::cli::BrowserName;
use crate::cmds::add;
use std::fs;
use std::process::Command;

pub fn install(name: BrowserName) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    match name {
        BrowserName::Chromium => {
            println!("Installing Chromium…");
            let rc = add::run("chromium", true);
            if rc != 0 { return rc; }
            setup_chromium_policy("/etc/chromium/policies/managed", &omarchy_path);
            copy_chromium_flags("chromium-flags.conf", &omarchy_path);
            let _ = Command::new("omarchy-theme-set-browser").status();
            announce("Chromium");
            0
        }
        BrowserName::Chrome => {
            println!("Installing Chrome…");
            let rc = add::run("google-chrome", true);
            if rc != 0 { return rc; }
            setup_chromium_policy("/etc/opt/chrome/policies/managed", &omarchy_path);
            copy_chromium_flags("chrome-flags.conf", &omarchy_path);
            let _ = Command::new("omarchy-theme-set-browser").status();
            announce("Chrome");
            0
        }
        BrowserName::Edge => {
            println!("Installing Edge…");
            let rc = add::run("microsoft-edge", true);
            if rc != 0 { return rc; }
            setup_chromium_policy("/etc/opt/edge/policies/managed", &omarchy_path);
            copy_chromium_flags("microsoft-edge-stable-flags.conf", &omarchy_path);
            let _ = Command::new("omarchy-theme-set-browser").status();
            announce("Edge");
            0
        }
        BrowserName::Brave => {
            println!("Installing Brave…");
            let rc = add::run("brave", true);
            if rc != 0 { return rc; }
            setup_chromium_policy("/etc/brave/policies/managed", &omarchy_path);
            copy_chromium_flags("brave-flags.conf", &omarchy_path);
            let _ = Command::new("omarchy-theme-set-browser").status();
            announce("Brave");
            0
        }
        BrowserName::BraveOrigin => {
            println!("Installing Brave Origin…");
            let rc = add::run("brave", true);
            if rc != 0 { return rc; }
            setup_chromium_policy("/etc/brave/policies/managed", &omarchy_path);
            copy_chromium_flags("brave-origin-flags.conf", &omarchy_path);
            let _ = Command::new("omarchy-theme-set-browser").status();
            announce("Brave Origin");
            0
        }
        BrowserName::Firefox => {
            println!("Installing Firefox…");
            let rc = add::run("firefox", true);
            if rc != 0 { return rc; }
            setup_firefox_wayland();
            announce("Firefox");
            0
        }
        BrowserName::Zen => {
            println!("Installing Zen…");
            let rc = add::run("zen-browser", true);
            if rc != 0 { return rc; }
            setup_firefox_wayland();
            announce("Zen");
            0
        }
    }
}

fn setup_chromium_policy(dir: &str, omarchy_path: &str) {
    // Source the browser policy helper (call it as a subprocess)
    let _ = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "source \"{omarchy_path}/install/helpers/browser-policy.sh\" && browser_policy_setup_dir \"{dir}\""
        ))
        .status();
    let _ = Command::new("omarchy-install-chromium-copy-url").status();
    let _ = Command::new("omarchy-install-chromium-ytdlp").status();
}

fn copy_chromium_flags(flags_file: &str, omarchy_path: &str) {
    let home = std::env::var("HOME").unwrap_or_default();
    let src = format!("{omarchy_path}/config/chromium-flags.conf");
    let dst = format!("{home}/.config/{flags_file}");
    let _ = fs::create_dir_all(format!("{home}/.config"));
    let _ = fs::copy(&src, &dst);
}

fn setup_firefox_wayland() {
    let home = std::env::var("HOME").unwrap_or_default();
    let env_dir = format!("{home}/.config/environment.d");
    let _ = fs::create_dir_all(&env_dir);
    let _ = fs::write(
        format!("{env_dir}/omarchy-firefox-wayland.conf"),
        "MOZ_ENABLE_WAYLAND=1\n",
    );
}

fn announce(name: &str) {
    println!("\n{name} browser installed. Make it the default via Setup > Defaults > Browser.");
}
