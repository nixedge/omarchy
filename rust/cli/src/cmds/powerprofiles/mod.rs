use std::fs;
use std::process::{Command, Stdio};

pub fn init() -> i32 {
    // Just delegate to powerprofiles-set autodetect
    let status = Command::new("omarchy-powerprofiles-set")
        .arg("autodetect")
        .status();
    if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
}

pub fn list(active_state: bool) -> i32 {
    let out = Command::new("powerprofilesctl")
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            parse_and_print_profiles(&stdout, active_state);
            0
        }
        Err(e) => {
            eprintln!("powerprofilesctl error: {e}");
            1
        }
    }
}

fn parse_and_print_profiles(output: &str, active_state: bool) {
    // Lines containing * are active; name is at the start of a line or after *
    // powerprofilesctl list output:
    //   * performance:
    //     Driver:     amd-pstate
    //   balanced:
    //   power-saver:
    let mut current_profile: Option<String> = None;
    let mut current_active = false;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        let (is_active, profile_name) = if let Some(rest) = trimmed.strip_prefix("* ") {
            (true, rest.trim_end_matches(':').trim().to_string())
        } else if trimmed.ends_with(':') && !trimmed.starts_with(' ') && !line.starts_with(' ') {
            (false, trimmed.trim_end_matches(':').to_string())
        } else {
            // Check for profile lines (lines starting with name:)
            if let Some(name) = trimmed.strip_suffix(':') {
                if !name.contains(' ') && !name.is_empty() {
                    (false, name.to_string())
                } else {
                    continue;
                }
            } else {
                continue;
            }
        };

        // Flush previous profile
        if let Some(prev) = current_profile.take() {
            if active_state {
                println!("{prev}\t{current_active}");
            } else {
                println!("{prev}");
            }
        }

        current_profile = Some(profile_name);
        current_active = is_active;
    }

    if let Some(prev) = current_profile {
        if active_state {
            println!("{prev}\t{current_active}");
        } else {
            println!("{prev}");
        }
    }
}

pub fn set(action: Option<&str>, profile: Option<&str>) -> i32 {
    let action = action.unwrap_or("autodetect");

    // Resolve autodetect to ac or battery
    let resolved_action = if action == "autodetect" {
        let out = Command::new("busctl")
            .args(["get-property",
                "org.freedesktop.UPower",
                "/org/freedesktop/UPower",
                "org.freedesktop.UPower",
                "OnBattery"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match out {
            Ok(o) if String::from_utf8_lossy(&o.stdout).contains("true") => "battery",
            _ => "ac",
        }
    } else {
        action
    };

    match resolved_action {
        "ac" | "battery" => {}
        other => {
            eprintln!("Unknown action: {other}. Use autodetect, ac, or battery.");
            return 1;
        }
    }

    // Get available profiles
    let profiles = get_available_profiles();

    let state_dir = std::env::var("OMARCHY_POWERPROFILES_STATE_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            let state_home = std::env::var("XDG_STATE_HOME")
                .unwrap_or_else(|_| format!("{home}/.local/state"));
            format!("{state_home}/omarchy/powerprofiles")
        });
    let state_file = format!("{state_dir}/{resolved_action}");

    let selected_profile = if let Some(p) = profile {
        if !profiles.contains(&p.to_string()) {
            eprintln!("Power profile is not available: {p}");
            return 1;
        }
        p.to_string()
    } else if let Ok(saved) = fs::read_to_string(&state_file) {
        let saved = saved.trim().to_string();
        if profiles.contains(&saved) {
            saved
        } else {
            default_profile(resolved_action, &profiles)
        }
    } else {
        default_profile(resolved_action, &profiles)
    };

    // Set the profile
    let status = Command::new("powerprofilesctl")
        .args(["set", &selected_profile])
        .status();

    if !status.map(|s| s.success()).unwrap_or(false) {
        eprintln!("Failed to set power profile: {selected_profile}");
        return 1;
    }

    // Save if explicitly requested
    if let Some(p) = profile {
        let _ = fs::create_dir_all(&state_dir);
        let _ = fs::write(&state_file, format!("{p}\n"));
    }

    0
}

fn get_available_profiles() -> Vec<String> {
    let out = Command::new("omarchy-powerprofiles-list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    match out {
        Ok(o) => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

fn default_profile(action: &str, profiles: &[String]) -> String {
    if action == "ac" && profiles.contains(&"performance".to_string()) {
        "performance".to_string()
    } else if profiles.contains(&"balanced".to_string()) {
        "balanced".to_string()
    } else {
        profiles.first().cloned().unwrap_or_else(|| "balanced".to_string())
    }
}
