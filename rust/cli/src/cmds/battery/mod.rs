use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn present() -> i32 {
    // Read /sys/class/power_supply/BAT*/type and present
    let Ok(entries) = fs::read_dir("/sys/class/power_supply") else { return 1; };
    for entry in entries.flatten() {
        let path = entry.path();
        let type_str = fs::read_to_string(path.join("type"))
            .unwrap_or_default()
            .trim()
            .to_string();
        if type_str != "Battery" {
            continue;
        }
        let present = fs::read_to_string(path.join("present"))
            .unwrap_or_default()
            .trim()
            .to_string();
        if present == "1" {
            return 0;
        }
    }
    1
}

pub fn low(percentage: u32) -> i32 {
    let msg = format!("Battery at {percentage}%");
    Command::new("omarchy-notification-send")
        .args(["-g", "󰁹", "-u", "critical", &msg])
        .status()
        .ok();
    Command::new("omarchy-hook")
        .args(["battery-low"])
        .status()
        .ok();
    0
}

pub fn status(shell_output: bool) -> i32 {
    // Run upower to find battery
    let battery_list = Command::new("upower")
        .arg("-e")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let battery = match battery_list {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            s.lines().find(|l| l.contains("BAT")).map(|l| l.to_string())
        }
        Err(_) => None,
    };

    let battery = match battery {
        Some(b) => b,
        None => return 0, // no battery, exit 0
    };

    let info_out = Command::new("upower")
        .args(["-i", &battery])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let info = match info_out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => return 1,
    };

    let percentage = parse_upower_field(&info, "percentage")
        .and_then(|s| {
            let s = s.trim_end_matches('%');
            s.parse::<f64>().ok().map(|f| f as i32)
        })
        .unwrap_or(0);

    let capacity = parse_upower_field(&info, "energy-full")
        .and_then(|s| s.split_whitespace().next().and_then(|n| n.parse::<f64>().ok()))
        .map(|f| f as i32)
        .unwrap_or(0);

    let time_remaining = parse_time_remaining(&info);

    let power_rate_upower = parse_upower_field(&info, "energy-rate")
        .and_then(|s| s.split_whitespace().next().and_then(|n| n.parse::<f64>().ok()))
        .unwrap_or(0.0);

    let native_path = parse_upower_field(&info, "native-path").unwrap_or_default();

    let ps_path = std::env::var("OMARCHY_POWER_SUPPLY_PATH")
        .unwrap_or_else(|_| "/sys/class/power_supply".to_string());
    let battery_path = format!("{ps_path}/{native_path}");

    // Use instantaneous sysfs reading if available
    let power_rate_raw = read_sysfs_power_rate(&battery_path).unwrap_or(power_rate_upower);

    let power_rate = format_power_rate(power_rate_raw);

    let state = parse_upower_field(&info, "state").unwrap_or_default();

    let threshold_end = parse_charge_threshold(&info, "charge-end-threshold")
        .or_else(|| read_sysfs_threshold(&ps_path, "charge_control_end_threshold"));
    let threshold_start = parse_charge_threshold(&info, "charge-start-threshold")
        .or_else(|| read_sysfs_threshold(&ps_path, "charge_control_start_threshold"));

    // Check AC online
    let ac_online = check_ac_online(&ps_path);

    // charge_idle: power rate <= 0.2
    let charge_idle = power_rate_raw <= 0.2;

    // charge_holding logic
    let charge_holding = if ac_online {
        if let Some(end) = threshold_end {
            if state == "pending-charge" {
                true
            } else if state == "fully-charged" && percentage < 99 {
                true
            } else if state == "charging" && charge_idle && end < 99 && percentage >= end {
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if shell_output {
        println!("percentage\t{}%", percentage);
        if charge_holding {
            println!("state\tholding");
        } else {
            println!("state\t{state}");
        }
        println!("rate\t{power_rate}W");
        println!("size\t{capacity}Wh");
        println!("time\t{time_remaining}");

        // cycles
        let cycles = read_first_sysfs_glob(&ps_path, "BAT*/cycle_count");
        if let Some(c) = cycles {
            println!("cycles\t{c}");
        }

        if let Some(end) = threshold_end {
            if let Some(start) = threshold_start {
                if start != end {
                    println!("threshold\t{start}-{end}%");
                } else {
                    println!("threshold\t{end}%");
                }
            } else {
                println!("threshold\t{end}%");
            }
        }

        return 0;
    }

    // Normal output
    if charge_holding {
        let threshold_label = format_threshold(threshold_start, threshold_end);
        println!("Battery {percentage}%  ·  Holding at {threshold_label}  ·  {power_rate}W / {capacity}Wh");
    } else if state == "charging" {
        println!("Battery {percentage}%  ·  {time_remaining} to full  ·   {power_rate}W / {capacity}Wh");
    } else {
        println!("Battery {percentage}%  ·  {time_remaining} left  ·   {power_rate}W / {capacity}Wh");
    }

    0
}

fn parse_upower_field<'a>(info: &'a str, field: &str) -> Option<String> {
    info.lines()
        .find(|l| l.trim_start().starts_with(field))
        .and_then(|l| l.splitn(2, ':').nth(1))
        .map(|v| v.trim().to_string())
}

fn parse_charge_threshold(info: &str, field: &str) -> Option<i32> {
    parse_upower_field(info, field).and_then(|s| {
        s.trim_end_matches('%').trim().parse::<f64>().ok().map(|f| f as i32)
    })
}

fn parse_time_remaining(info: &str) -> String {
    for line in info.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("time to empty") || trimmed.starts_with("time to full") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 4 {
                let value: f64 = parts[3].parse().unwrap_or(0.0);
                let unit = parts.get(4).copied().unwrap_or("");
                if unit.starts_with("minute") {
                    return format!("{}m", value as i32);
                } else {
                    let hours = value as i32;
                    let minutes = ((value - hours as f64) * 60.0) as i32;
                    if minutes > 0 {
                        return format!("{hours}h {minutes}m");
                    } else {
                        return format!("{hours}h");
                    }
                }
            }
        }
    }
    String::new()
}

fn read_sysfs_power_rate(battery_path: &str) -> Option<f64> {
    let power_now = format!("{battery_path}/power_now");
    if Path::new(&power_now).exists() {
        let microwatts: f64 = fs::read_to_string(&power_now).ok()?.trim().parse().ok()?;
        return Some(microwatts / 1_000_000.0);
    }
    let current_now = format!("{battery_path}/current_now");
    let voltage_now = format!("{battery_path}/voltage_now");
    if Path::new(&current_now).exists() && Path::new(&voltage_now).exists() {
        let microamps: f64 = fs::read_to_string(&current_now).ok()?.trim().parse().ok()?;
        let microvolts: f64 = fs::read_to_string(&voltage_now).ok()?.trim().parse().ok()?;
        return Some(microamps * microvolts / 1_000_000_000_000.0);
    }
    None
}

fn format_power_rate(rate: f64) -> String {
    let rounded = format!("{rate:.1}");
    if rounded.ends_with(".0") {
        rounded[..rounded.len() - 2].to_string()
    } else {
        rounded
    }
}

fn check_ac_online(ps_path: &str) -> bool {
    let Ok(entries) = fs::read_dir(ps_path) else { return false; };
    for entry in entries.flatten() {
        let path = entry.path();
        let type_str = fs::read_to_string(path.join("type"))
            .unwrap_or_default().trim().to_string();
        if type_str != "Mains" { continue; }
        let online = fs::read_to_string(path.join("online"))
            .unwrap_or_default().trim().to_string();
        if online == "1" { return true; }
    }
    false
}

fn read_sysfs_threshold(ps_path: &str, filename: &str) -> Option<i32> {
    read_first_sysfs_glob(ps_path, &format!("BAT*/{filename}"))
        .and_then(|s| s.trim().parse().ok())
}

fn read_first_sysfs_glob(ps_path: &str, pattern: &str) -> Option<String> {
    let (prefix, suffix) = pattern.split_once('/')?;
    let Ok(entries) = fs::read_dir(ps_path) else { return None; };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // Simple glob: prefix is like "BAT*"
        let glob_prefix = prefix.trim_end_matches('*');
        if name_str.starts_with(glob_prefix) {
            let file_path = entry.path().join(suffix);
            if let Ok(content) = fs::read_to_string(&file_path) {
                return Some(content.trim().to_string());
            }
        }
    }
    None
}

fn format_threshold(start: Option<i32>, end: Option<i32>) -> String {
    match (start, end) {
        (Some(s), Some(e)) if s != e => format!("{s}-{e}%"),
        (_, Some(e)) => format!("{e}%"),
        _ => String::new(),
    }
}
