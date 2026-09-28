use super::launch_detached;
use crate::cmds::add;
use std::process::Command;

pub fn claude() -> i32 {
    println!("Installing Claude…");
    let rc = add::run("claude-desktop");
    if rc != 0 {
        return rc;
    }
    println!("Opening Claude…");
    launch_detached("/usr/bin/claude-desktop", &[]);
    println!("\nClaude has been installed.");
    0
}

pub fn t3code() -> i32 {
    println!("Installing T3 Code…");
    let rc = add::run("t3code");
    if rc != 0 {
        return rc;
    }

    println!("Matching T3 Code to the current theme…");
    let state_dir = super::home_path(".t3/userdata");
    let _ = std::fs::create_dir_all(&state_dir);

    // Refresh theme templates if needed
    let t3_theme = super::home_path(".local/state/omarchy/current/theme/t3code.json");
    if !t3_theme.exists() {
        let _ = Command::new("omarchy-theme-refresh").status();
    }

    let _ = Command::new("omarchy-theme-set-t3code").status();

    println!("Opening T3 Code…");
    let _ = Command::new("setsid")
        .args(["uwsm-app", "--", "gtk-launch", "t3code"])
        .spawn();

    println!("\nT3 Code has been installed.");
    0
}

pub fn chatgpt() -> i32 {
    println!("Installing ChatGPT…");
    let rc = add::run("openai-codex-desktop");
    if rc != 0 {
        return rc;
    }
    println!("Opening ChatGPT…");
    launch_detached("/usr/bin/chatgpt", &[]);
    println!("\nChatGPT has been installed.");
    0
}

pub fn openclaw() -> i32 {
    println!("Installing OpenClaw…");
    let rc = add::run("openclaw");
    if rc != 0 {
        return rc;
    }

    // Icon ships inside the openclaw package. On NixOS it lands in the nix
    // store and is symlinked into /run/current-system/sw.
    let icon_candidates = [
        "/run/current-system/sw/lib/node_modules/openclaw/dist/control-ui/apple-touch-icon.png",
        "/run/current-system/sw/share/openclaw/apple-touch-icon.png",
    ];
    let icon = icon_candidates
        .iter()
        .find(|p| std::path::Path::new(p).is_file())
        .copied()
        .unwrap_or("applications-internet");

    println!("Installing the OpenClaw web app…");
    let rc = super::webapp::run(
        Some("OpenClaw"),
        Some("http://127.0.0.1:18789"),
        Some(icon),
        Some("omarchy-launch-openclaw"),
        None,
    );
    if rc != 0 {
        return rc;
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let config_file = format!("{home}/.openclaw/openclaw.json");

    if std::path::Path::new(&config_file).exists() {
        println!("Opening OpenClaw…");
        super::launch_desktop("OpenClaw");
    } else {
        let onboarded = Command::new("omarchy-openclaw-onboard")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if onboarded && std::path::Path::new(&config_file).exists() {
            println!("Opening OpenClaw…");
            super::launch_desktop("OpenClaw");
        }
    }

    0
}

/// Hermes install delegates to the original bash script logic embedded here.
pub fn hermes() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script_path = format!("{omarchy_path}/bin/omarchy-install-ai-hermes");

    // If the original bash script still exists (during transition), exec it
    if std::path::Path::new(&script_path).exists() {
        let err = std::os::unix::process::CommandExt::exec(
            Command::new("bash").arg(&script_path)
        );
        eprintln!("exec failed: {err}");
        return 1;
    }

    // Fallback: pkg add hermes-desktop
    println!("Installing Hermes Desktop…");
    let rc = add::run("hermes-desktop");
    if rc != 0 {
        return rc;
    }
    launch_detached("/usr/bin/hermes-desktop", &[]);
    println!("\nHermes Desktop has been installed.");
    0
}
