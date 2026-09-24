use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run(name: &str) -> i32 {
    match name {
        "lid" => lid(),
        "external-monitors" => external_monitors(),
        "clamshell" => clamshell(),
        "display" => display(),
        "touchpad" => touchpad(),
        "touchscreen" => touchscreen(),
        "webcam" => webcam(),
        _ => {
            eprintln!("hw state: unknown state '{name}'");
            1
        }
    }
}

fn lid() -> i32 {
    let Ok(entries) = fs::read_dir("/proc/acpi/button/lid") else {
        // No lid switch — not a laptop or driver not loaded.
        return 1;
    };
    for entry in entries.flatten() {
        let state_path = entry.path().join("state");
        let content = fs::read_to_string(&state_path).unwrap_or_default();
        if content.contains("closed") {
            println!("closed");
            return 0;
        }
    }
    println!("open");
    1
}

fn external_monitors() -> i32 {
    let drm_path = std::env::var("OMARCHY_DRM_PATH")
        .unwrap_or_else(|_| "/sys/class/drm".to_string());
    let Ok(entries) = fs::read_dir(&drm_path) else {
        return 1;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // Skip internal panel connectors.
        if name_str.contains("-eDP-")
            || name_str.contains("-LVDS-")
            || name_str.contains("-DSI-")
        {
            continue;
        }
        let status_path = entry.path().join("status");
        if fs::read_to_string(&status_path)
            .unwrap_or_default()
            .trim()
            == "connected"
        {
            return 0;
        }
    }
    1
}

fn clamshell() -> i32 {
    if lid() == 0 && external_monitors() == 0 { 0 } else { 1 }
}

fn display() -> i32 {
    let backlight_path = std::env::var("OMARCHY_BACKLIGHT_PATH")
        .unwrap_or_else(|_| "/sys/class/backlight".to_string());
    let Ok(entries) = fs::read_dir(&backlight_path) else {
        return 1;
    };

    // Priority order: gmux > amdgpu_bl* > intel_backlight > acpi_video* > first-found.
    // Exclude appletb_backlight (Touch Bar, not the panel).
    let mut candidates: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != "appletb_backlight")
        .collect();

    let priority = ["gmux_backlight", "amdgpu_bl", "intel_backlight", "acpi_video"];

    for prefix in priority {
        if let Some(name) = candidates.iter().find(|n| n.starts_with(prefix)) {
            println!("{name}");
            return 0;
        }
    }

    if let Some(first) = candidates.first() {
        println!("{first}");
        return 0;
    }

    1
}

fn hyprctl_device_query(jq_expr: &str) -> Option<String> {
    let out = Command::new("hyprctl")
        .args(["devices", "-j"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let result = Command::new("jq")
        .args(["-r", jq_expr])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    use std::io::Write;
    let mut child = result;
    child.stdin.as_mut()?.write_all(&out.stdout).ok()?;
    let output = child.wait_with_output().ok()?;
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() || s == "null" { None } else { Some(s) }
}

fn touchpad() -> i32 {
    match hyprctl_device_query(
        "[.mice[] | .name | select(test(\"touchpad|trackpad\"; \"i\"))] | first // empty",
    ) {
        Some(name) => {
            println!("{name}");
            0
        }
        None => 1,
    }
}

fn touchscreen() -> i32 {
    match hyprctl_device_query(
        "[.touch[]?.name, .tablets[]?.name] | first // empty",
    ) {
        Some(name) => {
            println!("{name}");
            0
        }
        None => 1,
    }
}

fn webcam() -> i32 {
    // Delegate to the existing webcam-list script which handles v4l2.
    let status = Command::new("omarchy-capture-webcam-list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    match status {
        Ok(out) if !out.stdout.is_empty() => 0,
        _ => 1,
    }
}
