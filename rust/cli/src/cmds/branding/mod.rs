use std::os::unix::process::CommandExt;
use std::process::Command;

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_default()
}

// ─── branding about ───────────────────────────────────────────────────────────

pub fn about(mode: &str) -> i32 {
    let home_dir = home();
    let target = format!("{}/.config/omarchy/branding/about.txt", home_dir);

    match mode {
        "image" => {
            let output = Command::new("omarchy-file-select")
                .args(["--title", "Pick PNG or SVG for About", "--extensions", "png svg"])
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    let image = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if image.is_empty() {
                        return 1;
                    }
                    let status = Command::new("omarchy-transcode-ascii")
                        .args([&image, &target, "--width", "54", "--height", "26"])
                        .status();
                    if status.map(|s| s.success()).unwrap_or(false) {
                        let _ = Command::new("omarchy-launch-about")
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status();
                    }
                    0
                }
                _ => 1,
            }
        }
        "text" => {
            let status = Command::new("omarchy-launch-editor")
                .arg(&target)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            if status.map(|s| s.success()).unwrap_or(false) {
                let _ = Command::new("omarchy-launch-about")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
            }
            0
        }
        "reset" => {
            let src = format!("{}/icon.txt", omarchy_path());
            if let Err(e) = std::fs::copy(&src, &target) {
                eprintln!("Failed to copy {}: {}", src, e);
                return 1;
            }
            let _ = Command::new("omarchy-launch-about")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            0
        }
        _ => {
            eprintln!("Usage: omarchy-branding-about <image|text|reset>");
            1
        }
    }
}

// ─── branding screensaver ─────────────────────────────────────────────────────

pub fn screensaver(mode: &str) -> i32 {
    let home_dir = home();
    let target = format!("{}/.config/omarchy/branding/screensaver.txt", home_dir);

    match mode {
        "image" => {
            let output = Command::new("omarchy-file-select")
                .args(["--title", "Pick PNG or SVG for screensaver", "--extensions", "png svg"])
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    let image = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if image.is_empty() {
                        return 1;
                    }
                    let status = Command::new("omarchy-transcode-ascii")
                        .args([&image, &target])
                        .status();
                    if status.map(|s| s.success()).unwrap_or(false) {
                        let _ = Command::new("omarchy-launch-screensaver")
                            .arg("force")
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status();
                    }
                    0
                }
                _ => 1,
            }
        }
        "text" => {
            let status = Command::new("omarchy-launch-editor")
                .arg(&target)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            if status.map(|s| s.success()).unwrap_or(false) {
                let _ = Command::new("omarchy-launch-screensaver")
                    .arg("force")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
            }
            0
        }
        "reset" => {
            let src = format!("{}/logo.txt", omarchy_path());
            if let Err(e) = std::fs::copy(&src, &target) {
                eprintln!("Failed to copy {}: {}", src, e);
                return 1;
            }
            let _ = Command::new("omarchy-launch-screensaver")
                .arg("force")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            0
        }
        _ => {
            eprintln!("Usage: omarchy-branding-screensaver <image|text|reset>");
            1
        }
    }
}
