use std::fs;
use std::process::{Command, Stdio};

pub fn run(args: &[String]) -> i32 {
    // Parse: [-i|--interactive] | <minutes> [message] | show [-j|--json] | clear
    let first = args.first().map(|s| s.as_str()).unwrap_or("");

    match first {
        "-i" | "--interactive" => open_interactive(),
        "show" | "list" => {
            let second = args.get(1).map(|s| s.as_str()).unwrap_or("");
            match second {
                "-j" | "--json" => show_json(),
                "" => show_reminders(),
                _ => { eprintln!("Usage: omarchy-reminder show [-j|--json]"); 1 }
            }
        }
        "clear" => clear_reminders(),
        "" => { print_usage(); 1 }
        minutes_str => {
            // Could be a number
            match minutes_str.parse::<u32>() {
                Ok(0) | Err(_) => { print_usage(); 1 }
                Ok(minutes) => {
                    let message = if args.len() > 1 {
                        args[1..].join(" ")
                    } else {
                        String::new()
                    };
                    set_reminder(minutes, &message)
                }
            }
        }
    }
}

fn print_usage() {
    println!("Usage: omarchy-reminder [-i|--interactive]");
    println!("       omarchy-reminder <minutes> [message]");
    println!("       omarchy-reminder show [-j|--json]");
    println!("       omarchy-reminder clear");
}

fn open_interactive() -> i32 {
    Command::new("omarchy-shell")
        .args(["shell", "summon", "omarchy.reminders", "{}"])
        .status()
        .map(|s| if s.success() { 0 } else { 1 })
        .unwrap_or(1)
}

fn format_remaining(seconds: i64) -> String {
    let minutes = seconds / 60;
    let remainder = seconds % 60;
    if minutes > 0 && remainder > 0 {
        format!("{minutes}m {remainder}s")
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        format!("{remainder}s")
    }
}

fn active_reminder_timers() -> Vec<(String, i64)> {
    let now = unix_now();
    let out = Command::new("systemctl")
        .args(["--user", "list-timers", "--all", "--output=json", "omarchy-reminder-*.timer"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    match out {
        Ok(o) => {
            let json: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap_or_default();
            json.as_array()
                .map(|arr| {
                    arr.iter().filter_map(|item| {
                        let timer = item.get("unit").and_then(|u| u.as_str())?;
                        let next_us = item.get("next").and_then(|n| n.as_i64()).unwrap_or(0);
                        let next_sec = next_us / 1_000_000;
                        if next_sec <= now { return None; }
                        Some((timer.to_string(), next_sec))
                    }).collect()
                })
                .unwrap_or_default()
        }
        Err(_) => Vec::new(),
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn get_reminder_message(timer: &str, reminder_dir: &str) -> String {
    let unit = timer.trim_end_matches(".timer");
    fs::read_to_string(format!("{reminder_dir}/{unit}.message"))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn show_reminders() -> i32 {
    let now = unix_now();
    let reminder_dir = get_reminder_dir();
    let timers = active_reminder_timers();

    if timers.is_empty() {
        Command::new("omarchy-notification-send")
            .args(["-g", "󰢌", "Upcoming reminders", "No outstanding reminders"])
            .status().ok();
        return 0;
    }

    let mut body = String::new();
    for (timer, next_sec) in timers {
        let remaining = next_sec - now;
        let reminder = timer.trim_end_matches(".timer").trim_start_matches("omarchy-reminder-").to_string();
        let minutes = reminder.split('m').next().unwrap_or("0").parse::<u32>().unwrap_or(0);
        let msg = get_reminder_message(&timer, &reminder_dir);
        let at = format_time(next_sec);

        if !msg.is_empty() {
            body.push_str(&format!("{msg} in {} ({at})\n", format_remaining(remaining)));
        } else {
            body.push_str(&format!("{minutes}-min reminder in {} ({at})\n", format_remaining(remaining)));
        }
    }

    Command::new("omarchy-notification-send")
        .args(["-g", "󰢌", "Upcoming reminders", body.trim_end_matches('\n')])
        .status().ok();
    0
}

fn show_json() -> i32 {
    let now = unix_now();
    let reminder_dir = get_reminder_dir();
    let timers = active_reminder_timers();
    let count = timers.len();

    let tooltip = match count {
        0 => "Set Reminder".to_string(),
        1 => "1 reminder".to_string(),
        n => format!("{n} reminders"),
    };

    let reminders: Vec<serde_json::Value> = timers.iter().map(|(timer, next_sec)| {
        let unit = timer.trim_end_matches(".timer");
        let reminder = unit.trim_start_matches("omarchy-reminder-");
        let minutes: u32 = reminder.split('m').next().unwrap_or("0").parse().unwrap_or(0);
        let remaining = next_sec - now;
        let msg = get_reminder_message(timer, &reminder_dir);
        let label = if !msg.is_empty() { msg.clone() } else { format!("{minutes}-min reminder") };
        let at = format_time(*next_sec);

        serde_json::json!({
            "unit": unit,
            "timer": timer,
            "minutes": minutes,
            "message": msg,
            "label": label,
            "remaining": format_remaining(remaining),
            "remainingSeconds": remaining,
            "at": next_sec,
            "atTime": at,
        })
    }).collect();

    let result = serde_json::json!({
        "count": count,
        "active": count > 0,
        "tooltip": tooltip,
        "reminders": reminders,
    });

    println!("{result}");
    0
}

fn clear_reminders() -> i32 {
    let reminder_dir = get_reminder_dir();

    let units_out = Command::new("systemctl")
        .args(["--user", "list-timers", "--all", "--no-legend", "--no-pager", "omarchy-reminder-*.timer"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    if let Ok(o) = units_out {
        let s = String::from_utf8_lossy(&o.stdout);
        let units: Vec<&str> = s.lines()
            .filter_map(|l| l.split_whitespace().rev().take(2).last())
            .filter(|s| !s.is_empty())
            .collect();

        if !units.is_empty() {
            let _ = Command::new("systemctl")
                .args(["--user", "stop"])
                .args(&units)
                .status();
        }
    }

    // Remove message files
    if let Ok(entries) = fs::read_dir(&reminder_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("omarchy-reminder-") && name.ends_with(".message") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    let _ = Command::new("omarchy-shell")
        .args(["-q", "omarchy.indicators", "refresh"])
        .stdout(Stdio::null()).stderr(Stdio::null()).status();

    Command::new("omarchy-notification-send")
        .args(["-g", "󰢌", "All reminders have been cleared"])
        .status().ok();
    0
}

fn set_reminder(minutes: u32, message: &str) -> i32 {
    let set_at = unix_now();
    let unit = format!("omarchy-reminder-{minutes}m-{set_at}");
    let reminder_dir = get_reminder_dir();
    let _ = fs::create_dir_all(&reminder_dir);

    let display_message = if message.is_empty() {
        format!("Your {minutes} minutes are up")
    } else {
        message.to_string()
    };

    let message_file = format!("{reminder_dir}/{unit}.message");

    let remind_at = Command::new("date")
        .args(["-d", &format!("+{minutes} minutes"), "+%H:%M"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let (confirmation_title, _) = if !message.is_empty() {
        let _ = fs::write(&message_file, message);
        (format!("{message} in {minutes} minutes"), format!("You'll be reminded at {remind_at}"))
    } else {
        (format!("Reminder set for {minutes} minutes"), format!("You'll be reminded at {remind_at}"))
    };

    let confirmation = format!("You'll be reminded at {remind_at}");

    let exec_str = format!(
        "omarchy-notification-send -g 󰢌 \"Reminder\" \"{}\"",
        display_message.replace('"', "\\\"")
    );
    let cleanup_str = if !message.is_empty() {
        format!("{exec_str}; rm -f \"{message_file}\"")
    } else {
        exec_str
    };
    let refresh_str = format!("{cleanup_str}; omarchy-shell -q omarchy.indicators refresh >/dev/null 2>&1 || true");

    let ok = Command::new("systemd-run")
        .args(["--user", "--quiet", "--collect",
            &format!("--on-active={minutes}m"),
            &format!("--unit={unit}"),
            "bash", "-c", &refresh_str])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok {
        eprintln!("Failed to schedule reminder");
        return 1;
    }

    let _ = Command::new("omarchy-shell")
        .args(["-q", "omarchy.indicators", "refresh"])
        .stdout(Stdio::null()).stderr(Stdio::null()).status();

    Command::new("omarchy-notification-send")
        .args(["-g", "󰢌", &confirmation_title, &confirmation])
        .status().ok();

    0
}

fn get_reminder_dir() -> String {
    std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string())
        + "/omarchy-reminders"
}

fn format_time(unix_sec: i64) -> String {
    Command::new("date")
        .args(["-d", &format!("@{unix_sec}"), "+%-H:%M"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}
