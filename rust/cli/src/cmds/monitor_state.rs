use std::process::{Command, Stdio};

pub fn run() -> i32 {
    // Get monitors JSON
    let monitors_out = Command::new("hyprctl")
        .args(["monitors", "all", "-j"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let monitors_json = match monitors_out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            eprintln!("hyprctl error: {e}");
            return 1;
        }
    };

    let monitors: serde_json::Value = match serde_json::from_str(&monitors_json) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("JSON parse error: {e}");
            return 1;
        }
    };

    let monitors_arr = match monitors.as_array() {
        Some(a) => a,
        None => {
            eprintln!("Unexpected monitors format");
            return 1;
        }
    };

    // Get focused monitor name
    let focused_monitor = monitors_arr.iter()
        .find(|m| m.get("focused").and_then(|f| f.as_bool()).unwrap_or(false))
        .and_then(|m| m.get("name").and_then(|n| n.as_str()))
        .unwrap_or("")
        .to_string();

    // Get brightness for focused monitor
    let brightness = Command::new("omarchy-brightness-display")
        .args(["--monitor", &focused_monitor])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().next().map(|l| l.to_string()).unwrap_or_default())
        .unwrap_or_default();
    println!("{brightness}");

    // Internal monitor patterns
    let is_internal = |name: &str| {
        name.starts_with("eDP") || name.starts_with("LVDS") || name.starts_with("DSI")
    };

    // Internal monitor name
    let internal_name = monitors_arr.iter()
        .find(|m| m.get("name").and_then(|n| n.as_str()).map(is_internal).unwrap_or(false))
        .and_then(|m| m.get("name").and_then(|n| n.as_str()))
        .unwrap_or("")
        .to_string();
    println!("{internal_name}");

    // External monitor name
    let external_name = monitors_arr.iter()
        .find(|m| {
            let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("");
            !is_internal(name)
        })
        .and_then(|m| m.get("name").and_then(|n| n.as_str()))
        .unwrap_or("")
        .to_string();
    println!("{external_name}");

    // Internal enabled (not disabled)
    let internal_enabled = monitors_arr.iter()
        .find(|m| {
            let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("");
            is_internal(name) && !m.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false)
        })
        .and_then(|m| m.get("name").and_then(|n| n.as_str()))
        .unwrap_or("")
        .to_string();
    println!("{internal_enabled}");

    // Mirroring source
    let mirror_source = monitors_arr.iter()
        .find(|m| {
            let mirror_of = m.get("mirrorOf").and_then(|v| v.as_str()).unwrap_or("none");
            mirror_of != "none"
        })
        .map(|m| {
            let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let mirror_of = m.get("mirrorOf").and_then(|v| v.as_str()).unwrap_or("");
            if is_internal(name) { mirror_of.to_string() } else { name.to_string() }
        })
        .unwrap_or_default();
    println!("{mirror_source}");

    // Focused monitor
    println!("{focused_monitor}");

    // Scaling
    let scaling_out = Command::new("omarchy-hyprland-monitor-scaling")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    println!("{scaling_out}");

    // Monitors JSON array — build manually to preserve field order expected by consumers.
    let monitor_list: Vec<String> = monitors_arr.iter()
        .map(|m| {
            let name    = m.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let enabled = !m.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false);
            let focused = m.get("focused").and_then(|f| f.as_bool()).unwrap_or(false);
            let width   = m.get("width").and_then(|w| w.as_i64()).unwrap_or(0);
            let height  = m.get("height").and_then(|h| h.as_i64()).unwrap_or(0);
            format!(r#"{{"name":"{name}","enabled":{enabled},"focused":{focused},"width":{width},"height":{height}}}"#)
        })
        .collect();
    println!("[{}]", monitor_list.join(","));

    0
}
