pub mod send;

use std::process::{Command, Stdio};
use std::time::Duration;

pub fn battery() -> i32 {
    let status_out = Command::new("omarchy-battery-status").output();
    let status_str = match status_out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => String::new(),
    };
    Command::new("omarchy-notification-send")
        .args(["-g", "󰁹", "-u", "low", &status_str])
        .status()
        .map(|s| if s.success() { 0 } else { 1 })
        .unwrap_or(1)
}

pub fn time() -> i32 {
    let date_out = Command::new("date")
        .arg("+%A %H:%M  ·  %d %B %Y  ·  Week %V")
        .output();
    let date_str = match date_out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => String::new(),
    };
    Command::new("omarchy-notification-send")
        .args(["-g", "", &date_str])
        .status()
        .map(|s| if s.success() { 0 } else { 1 })
        .unwrap_or(1)
}

pub fn weather() -> i32 {
    Command::new("omarchy-shell")
        .args(["shell", "toggle", "omarchy.weather"])
        .status()
        .map(|s| if s.success() { 0 } else { 1 })
        .unwrap_or(1)
}

pub fn dismiss(summary: &str) -> i32 {
    Command::new("omarchy-shell")
        .args(["shell", "notification", "dismiss", summary])
        .status()
        .map(|s| if s.success() { 0 } else { 1 })
        .unwrap_or(1)
}

pub fn wait(seconds: u64) -> i32 {
    // Poll busctl for notification server readiness
    let attempts = seconds * 10;
    for _ in 0..attempts {
        let shell_ok = Command::new("omarchy-shell")
            .args(["notifications", "ping"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        let server_ok = shell_ok && Command::new("busctl")
            .args(["--user", "call",
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
                "GetServerInformation"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if server_ok {
            return 0;
        }

        std::thread::sleep(Duration::from_millis(100));
    }
    1
}
