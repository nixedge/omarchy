use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn run_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_output(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

// ─── bluetooth device ─────────────────────────────────────────────────────

pub fn device(action: &str, address: &str) -> i32 {
    // Validate action and address
    let valid_actions = ["pair", "connect", "disconnect", "forget"];
    if !valid_actions.contains(&action) {
        eprintln!("Usage: omarchy-bluetooth-device [pair|connect|disconnect|forget] <address>");
        return 1;
    }

    // Validate MAC address format
    let mac_valid = {
        let parts: Vec<&str> = address.split(':').collect();
        parts.len() == 6 && parts.iter().all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
    };
    if !mac_valid {
        eprintln!("Usage: omarchy-bluetooth-device [pair|connect|disconnect|forget] <address>");
        return 1;
    }

    let power_on = || {
        let out = Command::new("timeout")
            .args(["2s", "bluetoothctl", "show"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        if out.contains("Powered: yes") {
            return;
        }
        let _ = Command::new("omarchy-bluetooth-power")
            .arg("on")
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
    };

    let trust_device = || {
        let _ = Command::new("bluetoothctl")
            .args(["trust", address])
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
    };

    match action {
        "pair" => {
            power_on();
            let _ = Command::new("timeout")
                .args(["20s", "bluetoothctl", "pair", address])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
            trust_device();
            let _ = Command::new("timeout")
                .args(["20s", "bluetoothctl", "connect", address])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
        }
        "connect" => {
            power_on();
            trust_device();
            let _ = Command::new("timeout")
                .args(["20s", "bluetoothctl", "connect", address])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
        }
        "disconnect" => {
            let _ = Command::new("timeout")
                .args(["10s", "bluetoothctl", "disconnect", address])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
        }
        "forget" => {
            power_on();
            let _ = Command::new("timeout")
                .args(["10s", "bluetoothctl", "disconnect", address])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
            let _ = Command::new("timeout")
                .args(["10s", "bluetoothctl", "remove", address])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
        }
        _ => {}
    }
    0
}

// ─── bluetooth power ──────────────────────────────────────────────────────

pub fn power(action: &str) -> i32 {
    let power_wait: u64 = std::env::var("OMARCHY_BLUETOOTH_POWER_WAIT_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    let controllers = || -> Vec<String> {
        Command::new("timeout")
            .args(["2s", "bluetoothctl", "list"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.split_whitespace().nth(1).map(|s| s.to_string()))
            .collect()
    };

    let powered = || -> bool {
        for controller in controllers() {
            let out = Command::new("timeout")
                .args(["2s", "bluetoothctl", "show", &controller])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();
            if out.contains("Powered: yes") {
                return true;
            }
        }
        false
    };

    let wait_powered = || -> bool {
        let deadline = Instant::now() + Duration::from_secs(power_wait);
        loop {
            if powered() { return true; }
            if Instant::now() >= deadline { return false; }
            std::thread::sleep(Duration::from_millis(200));
        }
    };

    let power_on = || -> i32 {
        let _ = Command::new("rfkill")
            .args(["unblock", "bluetooth"])
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
        if wait_powered() { return 0; }
        let _ = Command::new("timeout")
            .args(["5s", "bluetoothctl", "power", "on"])
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
        if wait_powered() { return 0; }
        eprintln!("omarchy-bluetooth-power: adapter did not come up");
        1
    };

    match action {
        "on" => power_on(),
        "off" => {
            let _ = Command::new("rfkill")
                .args(["block", "bluetooth"])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
            0
        }
        "toggle" => {
            if powered() {
                let _ = Command::new("rfkill")
                    .args(["block", "bluetooth"])
                    .stdout(Stdio::null()).stderr(Stdio::null()).status();
                0
            } else {
                power_on()
            }
        }
        "is-on" => {
            if powered() { 0 } else { 1 }
        }
        _ => {
            eprintln!("Usage: omarchy-bluetooth-power <on|off|toggle|is-on>");
            1
        }
    }
}
