use std::process::Command;

pub fn run(icon: Option<&str>, message: Option<&str>, progress: Option<&str>, duration: Option<&str>) -> i32 {
    let mut payload = serde_json::json!({});

    if let Some(i) = icon { payload["icon"] = serde_json::json!(i); }
    if let Some(m) = message { payload["message"] = serde_json::json!(m); }
    if let Some(p) = progress {
        if let Ok(f) = p.parse::<f64>() {
            payload["progress"] = serde_json::json!(f);
        }
    }
    if let Some(d) = duration {
        if let Ok(f) = d.parse::<f64>() {
            payload["duration"] = serde_json::json!(f);
        }
    }

    let payload_str = payload.to_string();
    let status = Command::new("omarchy-shell")
        .args(["shell", "osd", "show", &payload_str])
        .status();

    if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
}
