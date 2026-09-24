use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn loc_file() -> PathBuf {
    PathBuf::from(home()).join(".local/state/omarchy/settings/weather.json")
}

// ─── weather location ─────────────────────────────────────────────────────

pub fn location(action: Option<&str>, name_arg: Option<&str>, coords_arg: Option<&str>) -> i32 {
    match action {
        None => {
            // Show location
            let file = loc_file();
            let mut name = String::new();
            if file.exists() {
                if let Ok(content) = fs::read_to_string(&file) {
                    name = jq_scalar(&content, r#".name // "" | if type == "string" then . else "" end"#)
                        .unwrap_or_default();
                }
            }
            if name.is_empty() {
                // IP-detect
                name = curl_get("https://wttr.in/?format=%l", 4)
                    .map(|s| s.split(',').next().unwrap_or("").to_string())
                    .unwrap_or_default();
            }
            if !name.is_empty() {
                println!("{name}");
            }
            0
        }
        Some("--set") => {
            let name = match name_arg {
                Some(n) if !n.is_empty() => n,
                _ => {
                    eprintln!("Usage: omarchy-weather-location --set <name> [lat,lon]");
                    return 1;
                }
            };

            let json = if let Some(coords) = coords_arg {
                // Validate coords
                let parts: Vec<&str> = coords.splitn(2, ',').collect();
                if parts.len() != 2
                    || parts[0].parse::<f64>().is_err()
                    || parts[1].parse::<f64>().is_err()
                {
                    eprintln!("Invalid coordinates: {coords} (expected lat,lon)");
                    return 1;
                }
                let lat = parts[0];
                let lon = parts[1];
                format!(r#"{{"name":"{name}","latitude":{lat},"longitude":{lon}}}"#)
            } else {
                format!(r#"{{"name":"{name}"}}"#)
            };

            let file = loc_file();
            if let Some(parent) = file.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&file, format!("{json}\n"));
            0
        }
        Some("--clear") => {
            let _ = fs::remove_file(loc_file());
            0
        }
        Some(other) => {
            eprintln!("Usage: omarchy-weather-location [--set <name> [lat,lon]|--clear]");
            1
        }
    }
}

// ─── weather icon ──────────────────────────────────────────────────────────

pub fn icon() -> i32 {
    let file = loc_file();
    let query = if file.exists() && fs::metadata(&file).map(|m| m.len() > 0).unwrap_or(false) {
        // Get location name and URI encode
        let loc = Command::new("omarchy-weather-location")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if loc.is_empty() {
            String::new()
        } else {
            uri_encode(&loc)
        }
    } else {
        String::new()
    };

    let url = format!("https://wttr.in/{query}?format=j1");
    let weather_data = match curl_get(&url, 3) {
        Some(d) => d,
        None => return 1,
    };

    // Extract weather_code, sunrise, sunset via jq
    let tsv = jq_scalar(
        &weather_data,
        r#"[.current_condition[0].weatherCode, .weather[0].astronomy[0].sunrise, .weather[0].astronomy[0].sunset] | select(all(. != null and . != "")) | @tsv"#,
    );

    let tsv = match tsv {
        Some(t) if !t.is_empty() => t,
        _ => return 1,
    };

    let parts: Vec<&str> = tsv.split('\t').collect();
    if parts.len() != 3 { return 1; }
    let weather_code: u32 = match parts[0].parse() {
        Ok(n) => n, Err(_) => return 1,
    };
    let sunrise = parts[1];
    let sunset = parts[2];

    // Parse sunrise/sunset times
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let sunrise_ts = parse_time_today(sunrise).unwrap_or(0);
    let sunset_ts = parse_time_today(sunset).unwrap_or(0);

    let night = sunrise_ts > 0 && sunset_ts > 0
        && (now_ts < sunrise_ts || now_ts >= sunset_ts);

    let icon = match weather_code {
        113 => if night { "\u{e32b}" } else { "\u{e30d}" }, // moon / sun
        116 => if night { "\u{e379}" } else { "\u{e30c}" }, // partly cloudy night / day
        119 | 122 => "\u{e312}",                            // cloudy
        143 | 248 | 260 => "\u{e313}",                     // fog
        176 | 263 | 353 => if night { "\u{e325}" } else { "\u{e318}" }, // light rain night/day
        179 | 227 | 230 | 323 | 326 | 368 => if night { "\u{e327}" } else { "\u{e31a}" }, // snow night/day
        182 | 185 | 281 | 284 | 311 | 314 | 317 | 320 | 350 | 362 | 365 | 374 | 377 => "\u{e3ac}", // sleet
        200 | 386 | 389 | 392 | 395 => "\u{e31d}",         // thunderstorm
        266 | 293 | 296 | 299 | 302 | 305 | 308 | 356 | 359 => "\u{e319}", // rain
        329 | 332 | 335 | 338 | 371 => "\u{e31a}",         // heavy snow
        _ => "\u{e33d}",                                    // unknown
    };

    println!("{icon}");
    0
}

// ─── weather status ────────────────────────────────────────────────────────

pub fn status() -> i32 {
    let place = Command::new("omarchy-weather-location")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if place.is_empty() {
        println!("Weather unavailable");
        return 1;
    }

    let query = uri_encode(&place);
    let url = format!("https://wttr.in/{query}?format=%t|%w");
    let weather = match curl_get(&url, 4) {
        Some(w) if !w.is_empty() => w.trim().to_string(),
        _ => {
            println!("Weather unavailable");
            return 1;
        }
    };

    let parts: Vec<&str> = weather.splitn(2, '|').collect();
    if parts.len() != 2 {
        println!("Weather unavailable");
        return 1;
    }

    let temperature = parts[0].trim_start_matches('+');
    let wind = parts[1];

    // Capitalize first letter of place
    let place_cap = capitalize(&place);

    println!("{place_cap}  ·  Temp {temperature}  ·  Wind {wind}");
    0
}

// ─── helpers ────────────────────────────────────────────────────────────────

fn curl_get(url: &str, timeout: u32) -> Option<String> {
    let out = Command::new("curl")
        .args(["-fsS", "--max-time", &timeout.to_string(), url])
        .output()
        .ok()?;
    if out.status.success() {
        String::from_utf8(out.stdout).ok().map(|s| s.trim_end_matches('\n').to_string())
    } else {
        None
    }
}

fn jq_scalar(input: &str, filter: &str) -> Option<String> {
    let mut child = Command::new("jq")
        .args(["-r", filter])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }
    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn uri_encode(s: &str) -> String {
    s.bytes().map(|b| {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            (b as char).to_string()
        } else {
            format!("%{b:02X}")
        }
    }).collect()
}

fn parse_time_today(s: &str) -> Option<i64> {
    // Format: "6:30 AM" or "7:00 PM"
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 { return None; }
    let time_parts: Vec<&str> = parts[0].split(':').collect();
    if time_parts.len() != 2 { return None; }
    let mut hour: i64 = time_parts[0].parse().ok()?;
    let min: i64 = time_parts[1].parse().ok()?;
    let ampm = parts[1].to_uppercase();
    if ampm == "PM" && hour != 12 { hour += 12; }
    if ampm == "AM" && hour == 12 { hour = 0; }

    // Get today midnight in local time using date command
    let midnight = Command::new("date")
        .args(["+%s", "-d", "today 00:00:00"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<i64>().ok())
        .unwrap_or(0);

    Some(midnight + hour * 3600 + min * 60)
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
    }
}
