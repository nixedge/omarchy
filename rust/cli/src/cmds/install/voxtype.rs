use crate::desktop;
use super::add_many;
use std::process::Command;

pub fn run() -> i32 {
    if !desktop::gum_confirm(
        "Install Voxtype + AI model (~150MB) to enable dictation?",
    ) {
        return 0;
    }

    let rc = add_many(&["wtype", "voxtype-bin"], true);
    if rc != 0 {
        return rc;
    }

    // Setup config
    let config_dir = desktop::home_dir().join(".config/voxtype");
    let _ = std::fs::create_dir_all(&config_dir);
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let src = format!("{omarchy_path}/default/voxtype/config.toml");
    let _ = std::fs::copy(&src, config_dir.join("config.toml"));

    let _ = Command::new("voxtype")
        .args(["setup", "--download", "--no-post-install"])
        .status();

    // Enable GPU acceleration if Vulkan is available
    let has_vulkan = Command::new("omarchy-hw-vulkan")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if has_vulkan {
        let _ = Command::new("voxtype").args(["setup", "gpu", "--enable"]).status();
    }

    let _ = Command::new("voxtype").args(["setup", "systemd"]).status();
    let _ = Command::new("hyprctl").arg("reload").stdout(std::process::Stdio::null()).status();
    let _ = Command::new("omarchy-restart-shell").status();
    let _ = Command::new("omarchy-notification-send")
        .args([
            "-g", "",
            "Voxtype Dictation Ready",
            "Hold F9 to dictate (or toggle with Super + Ctrl + X).",
            "-t", "10000",
        ])
        .status();

    0
}
