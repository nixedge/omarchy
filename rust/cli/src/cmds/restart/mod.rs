pub mod audio;

fn current_uid() -> u32 {
    fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("Uid:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000)
}

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn app(name: &str, args: &[String]) -> i32 {
    let _ = Command::new("pkill").args(["-x", "--", name]).status();
    std::thread::sleep(Duration::from_millis(500));
    let mut cmd = Command::new("setsid");
    cmd.args(["uwsm-app", "--", name]);
    cmd.args(args);
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    match cmd.spawn() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Failed to spawn {name}: {e}");
            1
        }
    }
}

pub fn bluetooth() -> i32 {
    let status = Command::new("rfkill").args(["unblock", "bluetooth"]).status();
    if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
}

pub fn btop() -> i32 {
    let _ = Command::new("pkill").args(["-SIGUSR2", "btop"]).status();
    0
}

pub fn gum() -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let lua_path = format!("{home}/.local/state/omarchy/current/theme/gum_env.lua");
    let content = match fs::read_to_string(&lua_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot read {lua_path}: {e}");
            return 1;
        }
    };

    for line in content.lines() {
        let line = line.trim();
        // Parse: KEY = "VALUE"
        if let Some((key, rest)) = line.split_once(" = ") {
            let key = key.trim();
            if !key.starts_with("GUM_") {
                continue;
            }
            let val = rest.trim().trim_matches('"');
            println!("export {key}={val}");
        }
    }
    0
}

pub fn helix() -> i32 {
    let _ = Command::new("pkill").args(["-SIGUSR1", "hx"]).status();
    0
}

pub fn herdr() -> i32 {
    let output = Command::new("herdr")
        .args(["server", "reload-config"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Parse JSON and print diagnostics
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(diagnostics) = val.get("diagnostics").and_then(|d| d.as_array()) {
                    for diag in diagnostics {
                        if let Some(msg) = diag.get("message").and_then(|m| m.as_str()) {
                            eprintln!("{msg}");
                        }
                    }
                }
                if val.get("status").and_then(|s| s.as_str()) == Some("ok") {
                    0
                } else {
                    1
                }
            } else {
                eprintln!("Failed to parse herdr output");
                1
            }
        }
        Err(e) => {
            eprintln!("Failed to run herdr: {e}");
            1
        }
    }
}

pub fn hyprctl() -> i32 {
    let status = Command::new("hyprctl").arg("reload").status();
    if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
}

pub fn hyprsunset() -> i32 {
    let _ = Command::new("pkill").arg("hyprsunset").status();
    let mut cmd = Command::new("setsid");
    cmd.args(["uwsm-app", "--", "hyprsunset"]);
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    match cmd.spawn() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Failed to spawn hyprsunset: {e}");
            1
        }
    }
}

pub fn opencode() -> i32 {
    let _ = Command::new("pkill").args(["-SIGUSR2", "opencode"]).status();
    0
}

pub fn shell() -> i32 {
    // Port of omarchy-restart-shell
    let session_omarchy_path = Command::new("systemctl")
        .args(["--user", "show-environment"])
        .output()
        .ok()
        .and_then(|out| {
            String::from_utf8(out.stdout).ok().and_then(|s| {
                s.lines()
                    .filter_map(|l| l.strip_prefix("OMARCHY_PATH="))
                    .last()
                    .map(|v| v.to_string())
            })
        })
        .or_else(|| std::env::var("OMARCHY_PATH").ok())
        .unwrap_or_default();

    if session_omarchy_path.is_empty() {
        eprintln!("OMARCHY_PATH is not set");
        return 1;
    }

    let config_dir = format!("{session_omarchy_path}/shell");
    if !Path::new(&format!("{config_dir}/shell.qml")).exists() {
        eprintln!("Omarchy shell config not found: {config_dir}");
        return 1;
    }

    // Set HYPRLAND_INSTANCE_SIGNATURE if not set
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        let uid = current_uid();
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{uid}"));
        let hypr_dir = format!("{runtime_dir}/hypr");
        if let Ok(entries) = fs::read_dir(&hypr_dir) {
            let newest = entries.flatten()
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
            if let Some(dir) = newest {
                if let Some(sig) = dir.file_name().to_str() {
                    std::env::set_var("HYPRLAND_INSTANCE_SIGNATURE", sig);
                }
            }
        }
    }

    // Remember whether the notification service was up before the restart so we
    // can wait for it to come back (core IPC answers before it re-registers).
    let notifications_were_running = notifications_ready();

    // Check if session is locked
    let mut relock = false;
    let locked = Command::new("omarchy-hyprland-session-locked").status()
        .map(|s| s.success()).unwrap_or(false);

    if locked {
        let locking = shell_ipc_query(&session_omarchy_path, &["lock", "status"])
            .and_then(|out| serde_json::from_str::<serde_json::Value>(&out).ok())
            .and_then(|v| {
                let secure = v.get("secure").and_then(|s| s.as_bool()).unwrap_or(false);
                let requested = v.get("requested").and_then(|r| r.as_bool()).unwrap_or(false);
                Some(secure || requested)
            })
            .unwrap_or(false);

        if locking {
            eprintln!("Refusing to restart Omarchy shell while the session is locked.");
            return 1;
        }
        relock = true;
    }

    // Kill all quickshell instances
    loop {
        let status = Command::new("quickshell")
            .args(["kill", "-p", &config_dir, "--any-display"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match status {
            Ok(s) if s.success() => {}
            _ => break,
        }
    }

    // Spawn from Hyprland
    let _ = Command::new("hyprctl")
        .args(["dispatch", "hl.dsp.exec_cmd(\"omarchy-launch-shell\")"])
        .stdout(Stdio::null())
        .status();

    // Wait for shell to be ready
    for _ in 0..20 {
        if shell_ping(&session_omarchy_path) {
            if relock && !relock_session(&session_omarchy_path) {
                eprintln!("Omarchy shell restarted, but the session lock was not re-secured.");
                return 1;
            }
            // Core IPC answers before the notification plugin re-registers its bus
            // name; wait for it before invitation toasts fire their one-time sends.
            if notifications_were_running {
                let mut notifications_restored = false;
                for _ in 0..20 {
                    if notifications_ready() {
                        notifications_restored = true;
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                if !notifications_restored {
                    eprintln!("Omarchy shell restarted, but its notification service did not become ready.");
                    return 1;
                }
            }
            // Restart invitation services
            let _ = Command::new("systemctl")
                .args(["--user", "try-restart", "omarchy-*-invitation.service"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            return 0;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    eprintln!("Omarchy shell did not become ready after restart.");
    1
}

fn notifications_ready() -> bool {
    Command::new("busctl")
        .args(["--user", "--timeout=1s", "call",
            "org.freedesktop.DBus", "/org/freedesktop/DBus",
            "org.freedesktop.DBus", "NameHasOwner",
            "s", "org.freedesktop.Notifications"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "b true")
        .unwrap_or(false)
}

fn shell_ipc_query(omarchy_path: &str, args: &[&str]) -> Option<String> {
    let out = Command::new("omarchy-shell")
        .env("OMARCHY_PATH", omarchy_path)
        .env("OMARCHY_SHELL_IPC_TIMEOUT", "0.5s")
        .args(args)
        .output()
        .ok()?;
    String::from_utf8(out.stdout).ok()
}

fn shell_ping(omarchy_path: &str) -> bool {
    Command::new("omarchy-shell")
        .env("OMARCHY_PATH", omarchy_path)
        .env("OMARCHY_SHELL_IPC_TIMEOUT", "0.5s")
        .args(["shell", "ping"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn relock_session(omarchy_path: &str) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        if std::time::Instant::now() >= deadline {
            return false;
        }
        let state = shell_ipc_query(omarchy_path, &["lock", "status"])
            .and_then(|out| serde_json::from_str::<serde_json::Value>(&out).ok())
            .map(|v| {
                let secure = v.get("secure").and_then(|s| s.as_bool()).unwrap_or(false);
                let requested = v.get("requested").and_then(|r| r.as_bool()).unwrap_or(false);
                if secure { "secure" } else if requested { "locking" } else { "idle" }.to_string()
            })
            .unwrap_or_default();

        match state.as_str() {
            "secure" => return true,
            "locking" => {}
            _ => {
                let _ = shell_ipc_query(omarchy_path, &["lock", "lock"]);
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn terminal() -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let alacritty = format!("{home}/.config/alacritty/alacritty.toml");
    if Path::new(&alacritty).exists() {
        // touch the file: open with write access to update mtime
        let _ = std::fs::OpenOptions::new().write(true).open(&alacritty);
        // Use utime to touch it
        let _ = Command::new("touch").arg(&alacritty).status();
    }
    let _ = Command::new("killall").args(["-SIGUSR1", "kitty"]).status();
    let _ = Command::new("killall").args(["-SIGUSR2", "ghostty"]).status();
    0
}

pub fn tmux() -> i32 {
    // Check if tmux has sessions
    let has_sessions = Command::new("tmux")
        .arg("has-session")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if has_sessions {
        let home = std::env::var("HOME").unwrap_or_default();
        let conf = format!("{home}/.config/tmux/tmux.conf");
        let status = Command::new("tmux")
            .args(["source-file", &conf])
            .status();
        if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
    } else {
        0
    }
}

pub fn trackpad() -> i32 {
    // For each /sys/bus/i2c/drivers/i2c_hid_acpi/i2c-*: unbind then bind
    let driver_path = "/sys/bus/i2c/drivers/i2c_hid_acpi";
    let mut any_error = false;

    if let Ok(entries) = fs::read_dir(driver_path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if !name_str.starts_with("i2c-") {
                continue;
            }
            let device_name = name_str.to_string();

            let unbind = fs::write(format!("{driver_path}/unbind"), device_name.as_bytes());
            if unbind.is_err() {
                // Try with sudo
                let _ = Command::new("sudo")
                    .args(["tee", &format!("{driver_path}/unbind")])
                    .arg(&device_name)
                    .stdin(Stdio::piped())
                    .status();
            }

            std::thread::sleep(Duration::from_millis(100));

            let bind = fs::write(format!("{driver_path}/bind"), device_name.as_bytes());
            if bind.is_err() {
                let _ = Command::new("sudo")
                    .args(["tee", &format!("{driver_path}/bind")])
                    .arg(&device_name)
                    .stdin(Stdio::piped())
                    .status();
            }
        }
    }

    // Also modprobe intel_quicki2c if loaded
    let loaded = fs::read_to_string("/proc/modules")
        .map(|m| m.contains("intel_quicki2c"))
        .unwrap_or(false);

    if loaded {
        let r = Command::new("sudo").args(["modprobe", "-r", "intel_quicki2c"]).status();
        if r.map(|s| s.success()).unwrap_or(false) {
            let _ = Command::new("sudo").args(["modprobe", "intel_quicki2c"]).status();
        } else {
            any_error = true;
        }
    }

    if any_error { 1 } else { 0 }
}

pub fn wifi() -> i32 {
    let _ = Command::new("rfkill").args(["unblock", "wifi"]).status();
    let _ = Command::new("nmcli").args(["networking", "on"]).status();
    let _ = Command::new("nmcli").args(["radio", "wifi", "on"]).status();
    let _ = Command::new("nmcli").args(["device", "wifi", "rescan"]).status();
    let status = Command::new("rfkill").args(["list", "wifi"]).status();
    if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
}

pub fn xcompose() -> i32 {
    let _ = Command::new("systemctl")
        .args(["--user", "stop", "--now", "fcitx5.service"])
        .status();
    let _ = Command::new("pkill").args(["-x", "fcitx5"]).status();
    let status = Command::new("systemctl")
        .args(["--user", "start", "fcitx5.service"])
        .status();
    if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
}
