use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;

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

// ─── brightness display ───────────────────────────────────────────────────

pub fn display(no_osd: bool, monitor: Option<&str>, step: Option<&str>) -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());

    // Resolve monitor
    let monitor_name: String = match monitor {
        Some(m) => m.to_string(),
        None => Command::new("omarchy-hyprland-monitor-focused")
            .stderr(Stdio::null())
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default(),
    };

    let monitor_is_internal = monitor_name.starts_with("eDP-")
        || monitor_name.starts_with("LVDS-")
        || monitor_name.starts_with("DSI-");

    let use_apple = !monitor_name.is_empty()
        && run_ok("omarchy-hyprland-monitor-focused-apple", &[&monitor_name]);
    let use_ddc = !monitor_name.is_empty() && !monitor_is_internal;

    // Query current brightness
    if step.is_none() {
        if use_apple {
            return run_delegate("omarchy-brightness-display-apple", &[]);
        } else if use_ddc {
            return run_delegate("omarchy-brightness-display-ddc", &[&monitor_name]);
        } else {
            let device = match run_output("omarchy-hw-display", &[]) {
                Some(d) if !d.is_empty() => d,
                _ => return 1,
            };
            if let Some(b) = backlight_brightness(&device) {
                println!("{b}");
            }
            return 0;
        }
    }

    let step = step.unwrap();

    if step == "off" {
        let _ = Command::new("hyprctl")
            .args(["dispatch", "hl.dsp.dpms({ action = \"disable\" })"])
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
        return 0;
    }
    if step == "on" {
        // Check if all monitors already dpms-on
        let already_on = Command::new("hyprctl")
            .args(["monitors", "-j"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                // Use jq to check
                let out = Command::new("jq")
                    .args(["-e", "[.[] | select(.disabled == false)] | length > 0 and all(.dpmsStatus)"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .ok()?;
                use std::io::Write;
                let mut child = out;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(s.as_bytes());
                }
                child.wait_with_output().ok().map(|o| o.status.success())
            })
            .unwrap_or(false);
        if !already_on {
            let _ = Command::new("hyprctl")
                .args(["dispatch", "hl.dsp.dpms({ action = \"enable\" })"])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
        }
        return 0;
    }

    // Acquire lock
    let lock_path = PathBuf::from(&runtime_dir).join("omarchy-brightness-display.lock");
    let lock_file = match std::fs::OpenOptions::new().create(true).write(true).open(&lock_path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    use std::os::unix::io::AsRawFd;
    let fd = lock_file.as_raw_fd();
    let locked = unsafe { libc_flock(fd, 6) }; // LOCK_EX | LOCK_NB = 6
    if locked != 0 {
        return 0;
    }

    if use_apple {
        let mut apple_args = vec![];
        if no_osd { apple_args.push("--no-osd"); }
        apple_args.push(step);
        return run_delegate("omarchy-brightness-display-apple", &apple_args);
    } else if use_ddc {
        let brightness = run_output("omarchy-brightness-display-ddc", &[&monitor_name, step]);
        match brightness {
            Some(b) if !b.is_empty() => {
                if !no_osd {
                    let _ = Command::new("omarchy-osd").args(["-i", "brightness", "-p", &b]).status();
                }
                return 0;
            }
            _ => return 1,
        }
    } else {
        let device = match run_output("omarchy-hw-display", &[]) {
            Some(d) if !d.is_empty() => d,
            _ => return 1,
        };
        let current = match backlight_brightness(&device) {
            Some(c) => c,
            None => return 1,
        };

        let resolved_step = match step {
            "+5%" => {
                let target = if current < 5 { current + 1 } else { current + 5 }.min(100);
                format!("{target}%")
            }
            "5%-" => {
                let target = if current <= 5 {
                    current.saturating_sub(1)
                } else {
                    current - 5
                }.max(1);
                format!("{target}%")
            }
            other => other.to_string(),
        };

        let _ = Command::new("brightnessctl")
            .args(["-d", &device, "set", &resolved_step])
            .stdout(Stdio::null()).status();

        if !no_osd {
            if let Some(new_b) = backlight_brightness(&device) {
                let _ = Command::new("omarchy-osd")
                    .args(["-i", "brightness", "-p", &new_b.to_string()])
                    .status();
            }
        }
    }
    0
}

fn run_delegate(cmd: &str, args: &[&str]) -> i32 {
    Command::new(cmd).args(args).status()
        .map(|s| s.code().unwrap_or(1))
        .unwrap_or(1)
}

fn backlight_brightness(device: &str) -> Option<u32> {
    let out = Command::new("brightnessctl")
        .args(["-d", device, "-m"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())?;
    out.split(',').nth(3)
        .map(|s| s.trim_end_matches('%').trim().to_string())
        .and_then(|s| s.parse().ok())
}

#[link(name = "c")]
extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

fn libc_flock(fd: i32, operation: i32) -> i32 {
    unsafe { flock(fd, operation) }
}

// ─── brightness display-apple ─────────────────────────────────────────────

pub fn display_apple(no_osd: bool, step: Option<&str>) -> i32 {
    let device_cache = std::env::var("XDG_RUNTIME_DIR").ok()
        .map(|d| PathBuf::from(d).join("omarchy-brightness-display-apple.device"));

    let device = match find_apple_display_device(&device_cache) {
        Some(d) => d,
        None => {
            eprintln!("No Apple Display HID device found");
            return 1;
        }
    };

    match step {
        None => {
            match current_apple_brightness(&device) {
                Some(b) => { println!("{b}"); 0 }
                None => {
                    // Retry with fresh device
                    let device2 = retry_fresh_device(&device_cache);
                    match device2.and_then(|d| current_apple_brightness(&d)) {
                        Some(b) => { println!("{b}"); 0 }
                        None => 1,
                    }
                }
            }
        }
        Some(s) => {
            // Convert N%- to -N%
            let s = if s.ends_with("%-") {
                let trimmed = s.trim_end_matches("%-");
                if let Ok(n) = trimmed.parse::<u32>() {
                    format!("-{n}%")
                } else {
                    s.to_string()
                }
            } else {
                s.to_string()
            };

            let ok = run_ok("sudo", &["asdcontrol", &device, "--", &s]);
            if !ok {
                let device2 = retry_fresh_device(&device_cache);
                if let Some(ref d2) = device2 {
                    let _ = run_ok("sudo", &["asdcontrol", d2, "--", &s]);
                }
            }

            if !no_osd {
                let b = current_apple_brightness(&device).unwrap_or(0);
                let _ = Command::new("omarchy-osd")
                    .args(["-i", "brightness", "-p", &b.to_string()])
                    .status();
            }
            0
        }
    }
}

fn find_apple_display_device(cache: &Option<PathBuf>) -> Option<String> {
    if let Some(ref cache_path) = cache {
        if let Ok(cached) = fs::read_to_string(cache_path) {
            let cached = cached.trim();
            if (cached.starts_with("/dev/hiddev") || cached.starts_with("/dev/usb/hiddev"))
                && std::path::Path::new(cached).exists()
            {
                return Some(cached.to_string());
            }
        }
    }
    detect_apple_display_device(cache)
}

fn detect_apple_display_device(cache: &Option<PathBuf>) -> Option<String> {
    let mut devices = Vec::new();
    for dir_path in &["/dev/usb", "/dev"] {
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("hiddev") {
                    devices.push(entry.path().to_string_lossy().into_owned());
                }
            }
        }
    }
    if devices.is_empty() { return None; }

    let mut args = vec!["asdcontrol", "--detect"];
    for d in &devices { args.push(d.as_str()); }

    let out = Command::new("sudo")
        .args(&args)
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())?;

    let device = out.lines()
        .find(|l| l.starts_with("/dev/usb/hiddev") || l.starts_with("/dev/hiddev"))
        .and_then(|l| l.split(':').next())
        .map(|s| s.trim().to_string())?;

    if let Some(ref cache_path) = cache {
        let _ = fs::write(cache_path, format!("{device}\n"));
    }
    Some(device)
}

fn current_apple_brightness(device: &str) -> Option<u32> {
    let out = Command::new("sudo")
        .args(["asdcontrol", device])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())?;

    for line in out.lines() {
        if let Some(rest) = line.strip_prefix("BRIGHTNESS=") {
            if let Ok(val) = rest.trim().parse::<f64>() {
                return Some((val * 100.0 / 60000.0) as u32);
            }
        }
    }
    None
}

fn retry_fresh_device(cache: &Option<PathBuf>) -> Option<String> {
    if let Some(ref p) = cache { let _ = fs::remove_file(p); }
    detect_apple_display_device(cache)
}

// ─── brightness display-ddc ───────────────────────────────────────────────

pub fn display_ddc(monitor: &str, step: Option<&str>) -> i32 {
    if monitor.is_empty() {
        return 1;
    }

    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    let cache_dir = PathBuf::from(&runtime_dir).join("omarchy-brightness-display-ddc");
    let cache_name = monitor.chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '.' || c == '-' { c } else { '_' }).collect::<String>();
    let cache_file = cache_dir.join(format!("{cache_name}.bus"));

    let unavailable_cache_seconds: u64 = 60;
    let range_cache_seconds: u64 = 10;

    match step {
        None => {
            // Read and print current brightness
            match read_brightness_ddc(&cache_file, &cache_dir) {
                Some((_bus, current, maximum)) if maximum > 0 => {
                    let percent = (current * 100 + maximum / 2) / maximum;
                    println!("{percent}");
                    0
                }
                _ => 1,
            }
        }
        Some(s) => {
            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Parse step
            let (target, bus, maximum) = if let Some(pct) = s.strip_suffix('%').and_then(|p| if p.contains('+') || p.contains('-') { None } else { p.parse::<u32>().ok() }) {
                // Absolute percentage - try to use cached range
                let (bus_cached, max_cached, cached_at) = read_ddc_range_cache(&cache_file);
                if !bus_cached.is_empty() && max_cached > 0 && cached_at > 0
                    && now.saturating_sub(cached_at) < range_cache_seconds
                {
                    (pct.min(100).max(1), bus_cached, max_cached)
                } else {
                    match read_brightness_ddc(&cache_file, &cache_dir) {
                        Some((b, _, mx)) => (pct.min(100).max(1), b, mx),
                        None => return 1,
                    }
                }
            } else if let Some(pct) = s.strip_prefix('+').and_then(|p| p.strip_suffix('%')).and_then(|p| p.parse::<u32>().ok()) {
                match read_brightness_ddc(&cache_file, &cache_dir) {
                    Some((b, cur, mx)) if mx > 0 => {
                        let percent = (cur * 100 + mx / 2) / mx;
                        let t = if pct == 5 && percent < 5 { percent + 1 } else { percent + pct };
                        (t.min(100).max(1), b, mx)
                    }
                    _ => return 1,
                }
            } else if let Some(pct) = s.strip_suffix("%-").and_then(|p| p.parse::<u32>().ok()) {
                match read_brightness_ddc(&cache_file, &cache_dir) {
                    Some((b, cur, mx)) if mx > 0 => {
                        let percent = (cur * 100 + mx / 2) / mx;
                        let t = if pct == 5 && percent <= 5 { percent.saturating_sub(1) } else { percent.saturating_sub(pct) };
                        (t.min(100).max(1), b, mx)
                    }
                    _ => return 1,
                }
            } else {
                return 1;
            };

            let raw_target = (target * maximum + 50) / 100;
            let ok = run_ok("ddcutil", &["--bus", &bus, "--skip-ddc-checks", "--noverify", "setvcp", "10", &raw_target.to_string()]);
            if !ok {
                let _ = fs::remove_file(&cache_file);
                return 1;
            }
            println!("{target}");
            0
        }
    }
}

fn find_ddc_bus(monitor: &str, cache_file: &PathBuf, cache_dir: &PathBuf, unavailable_cache_seconds: u64) -> Option<String> {
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if let Ok(content) = fs::read_to_string(cache_file) {
        let mut parts = content.split_whitespace();
        let first = parts.next().unwrap_or("");
        if first == "unavailable" {
            let ts: u64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            if now.saturating_sub(ts) < unavailable_cache_seconds {
                return None;
            }
            let _ = fs::remove_file(cache_file);
        } else if !first.is_empty() && first.chars().all(|c| c.is_ascii_digit()) {
            return Some(first.to_string());
        }
    }

    // Detect bus
    let out = Command::new("ddcutil")
        .args(["--skip-ddc-checks", "detect", "--brief"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let bus = detect_ddc_bus_from_output(&out, monitor);
    if bus.is_empty() {
        let _ = fs::create_dir_all(cache_dir);
        let ts = now;
        let _ = fs::write(cache_file, format!("unavailable {ts}\n"));
        return None;
    }

    let _ = fs::create_dir_all(cache_dir);
    let _ = fs::write(cache_file, format!("{bus}\n"));
    Some(bus)
}

fn detect_ddc_bus_from_output(out: &str, monitor: &str) -> String {
    let mut bus = String::new();
    for line in out.lines() {
        if line.contains("I2C bus:") {
            if let Some(b) = line.split('/').last().and_then(|s| s.strip_prefix("i2c-")) {
                bus = b.trim().to_string();
            }
        }
        if line.contains("DRM connector:") {
            if let Some(conn) = line.split_whitespace().last() {
                let conn = conn.splitn(2, '-').nth(1).unwrap_or(conn);
                // Remove "card0-" prefix
                let conn = conn.splitn(2, '-').collect::<Vec<_>>();
                let connector = if conn.len() > 1 { conn[1..].join("-") } else { conn[0].to_string() };
                if connector == monitor && !bus.is_empty() {
                    return bus;
                }
                bus.clear();
            }
        }
    }
    String::new()
}

fn read_vcp(bus: &str) -> Option<(u32, u32)> {
    let out = Command::new("ddcutil")
        .args(["--bus", bus, "--skip-ddc-checks", "getvcp", "10", "--brief"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())?;

    for line in out.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5
            && parts[0] == "VCP"
            && parts[1].to_uppercase() == "10"
            && parts[2] == "C"
        {
            let cur: u32 = parts[3].parse().ok()?;
            let max: u32 = parts[4].parse().ok()?;
            if max > 0 { return Some((cur, max)); }
        }
    }
    None
}

fn read_brightness_ddc(cache_file: &PathBuf, cache_dir: &PathBuf) -> Option<(String, u32, u32)> {
    // We need the monitor name from the cache_file path to detect bus
    // Since we don't have it here, re-derive from cache_file name
    let cache_name = cache_file.file_stem()?.to_str()?.to_string();
    // Reconstruct monitor name (approximate - replace underscores back to what was there)
    // Actually we just need to call find_ddc_bus with the correct monitor - but we don't
    // have it here. Let's use a different approach: read the bus from cache first, or detect.
    let bus = find_ddc_bus_from_cache_or_detect(cache_file, cache_dir, &cache_name, 60)?;

    let (current, maximum) = read_vcp(&bus)?;

    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let _ = fs::create_dir_all(cache_dir);
    let _ = fs::write(cache_file, format!("{bus} {maximum} {now}\n"));
    Some((bus, current, maximum))
}

fn find_ddc_bus_from_cache_or_detect(cache_file: &PathBuf, cache_dir: &PathBuf, monitor: &str, unavailable_cache_seconds: u64) -> Option<String> {
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if let Ok(content) = fs::read_to_string(cache_file) {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if !parts.is_empty() {
            if parts[0] == "unavailable" {
                let ts: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                if now.saturating_sub(ts) < unavailable_cache_seconds {
                    return None;
                }
                let _ = fs::remove_file(cache_file);
            } else if parts[0].chars().all(|c| c.is_ascii_digit()) {
                return Some(parts[0].to_string());
            }
        }
    }

    let out = Command::new("ddcutil")
        .args(["--skip-ddc-checks", "detect", "--brief"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let bus = detect_ddc_bus_from_output(&out, monitor);
    if bus.is_empty() {
        let _ = fs::create_dir_all(cache_dir);
        let _ = fs::write(cache_file, format!("unavailable {now}\n"));
        return None;
    }

    let _ = fs::create_dir_all(cache_dir);
    let _ = fs::write(cache_file, format!("{bus}\n"));
    Some(bus)
}

fn read_ddc_range_cache(cache_file: &PathBuf) -> (String, u32, u64) {
    let content = fs::read_to_string(cache_file).unwrap_or_default();
    let parts: Vec<&str> = content.split_whitespace().collect();
    if parts.len() >= 3 {
        let bus = parts[0].to_string();
        let max: u32 = parts[1].parse().unwrap_or(0);
        let ts: u64 = parts[2].parse().unwrap_or(0);
        (bus, max, ts)
    } else {
        (String::new(), 0, 0)
    }
}

// ─── brightness keyboard ──────────────────────────────────────────────────

pub fn keyboard(no_osd: bool, direction: &str) -> i32 {
    // Find keyboard backlight device
    let device = find_kbd_backlight();
    let device = match device {
        Some(d) => d,
        None => {
            eprintln!("No keyboard backlight device found");
            return 1;
        }
    };

    if direction == "off" {
        let _ = Command::new("brightnessctl").args(["-sd", &device, "set", "0"]).stdout(Stdio::null()).status();
        return 0;
    }
    if direction == "restore" {
        let _ = Command::new("brightnessctl").args(["-rd", &device]).stdout(Stdio::null()).status();
        return 0;
    }

    let max_brightness: u32 = run_output("brightnessctl", &["-d", &device, "max"])
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let current_brightness: u32 = run_output("brightnessctl", &["-d", &device, "get"])
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let step = (max_brightness / 10).max(1);

    let new_brightness = match direction {
        "cycle" => {
            let next = current_brightness + step;
            if next > max_brightness { 0 } else { next }
        }
        "up" => (current_brightness + step).min(max_brightness),
        _ => current_brightness.saturating_sub(step),
    };

    let _ = Command::new("brightnessctl")
        .args(["-d", &device, "set", &new_brightness.to_string()])
        .stdout(Stdio::null()).status();

    if !no_osd {
        let pct = if max_brightness > 0 { new_brightness * 100 / max_brightness } else { 0 };
        let _ = Command::new("omarchy-osd")
            .args(["-i", "keyboard", "-p", &pct.to_string()])
            .status();
    }
    0
}

fn find_kbd_backlight() -> Option<String> {
    let entries = fs::read_dir("/sys/class/leds").ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains("kbd_backlight") {
            return Some(name);
        }
    }
    None
}

// ─── brightness keyboard-mute ─────────────────────────────────────────────

pub fn keyboard_mute(state: &str) -> i32 {
    let led_path = "/sys/class/leds/platform::micmute/brightness";
    if !std::path::Path::new(led_path).exists() {
        return 0;
    }

    let value = match state {
        "on" => "1",
        "off" => "0",
        _ => {
            eprintln!("Usage: omarchy-brightness-keyboard-mute <on|off>");
            return 1;
        }
    };

    let _ = Command::new("brightnessctl")
        .args(["--device=platform::micmute", "set", value])
        .stdout(Stdio::null()).stderr(Stdio::null()).status();
    0
}
