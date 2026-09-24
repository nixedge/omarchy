use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

fn run_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn toggle_flag_path(flag_name: &str) -> PathBuf {
    PathBuf::from(home())
        .join(".local/state/omarchy/toggles")
        .join(flag_name)
}

// ─── toggle flag ──────────────────────────────────────────────────────────

pub fn flag(flag_name: &str, action: &str) -> i32 {
    let flag_path = toggle_flag_path(flag_name);

    let enable = || {
        if let Some(parent) = flag_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::File::create(&flag_path);
    };

    let disable = || {
        let _ = fs::remove_file(&flag_path);
    };

    match action {
        "toggle" | "" => {
            if flag_path.exists() {
                disable();
            } else {
                enable();
            }
        }
        "on" => enable(),
        "off" => disable(),
        _ => {
            eprintln!("Usage: omarchy-toggle <flag-name> [toggle|on|off]");
            return 1;
        }
    }
    0
}

// ─── toggle enabled ───────────────────────────────────────────────────────

pub fn enabled(flag_name: &str) -> i32 {
    if toggle_flag_path(flag_name).is_file() { 0 } else { 1 }
}

// ─── toggle bar ───────────────────────────────────────────────────────────

pub fn bar(action: &str) -> i32 {
    flag("bar-off", action);
    // Nudge shell to re-read the flag
    let _ = Command::new("omarchy-shell")
        .args(["-q", "omarchy.bar", "syncHidden"])
        .stdout(Stdio::null()).stderr(Stdio::null()).status();
    0
}

// ─── toggle crash-capture ─────────────────────────────────────────────────

pub fn crash_capture() -> i32 {
    let crash_glyph = "\u{f16a1}"; // nf-md-robot_dead
    flag("crash-capture-off", "toggle");
    if enabled("crash-capture-off") == 0 {
        let _ = Command::new("systemctl")
            .args(["--user", "stop", "omarchy-crash-watch.service"])
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", crash_glyph, "Crash capture disabled"])
            .status();
    } else {
        let _ = Command::new("systemctl")
            .args(["--user", "start", "omarchy-crash-watch.service"])
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", crash_glyph, "Crash capture enabled"])
            .status();
    }
    0
}

// ─── toggle fullscreen-desktop ────────────────────────────────────────────

pub fn fullscreen_desktop(action_arg: &str) -> i32 {
    let action = if action_arg == "toggle" || action_arg.is_empty() {
        // Only leave full screen when both halves are already in it
        let bar_off = enabled("bar-off") == 0;
        let gaps_off = run_ok("omarchy-hyprland-toggle-enabled", &["window-no-gaps"]);
        if bar_off && gaps_off { "off" } else { "on" }
    } else if action_arg == "on" || action_arg == "off" {
        action_arg
    } else {
        eprintln!("Usage: omarchy-toggle-fullscreen-desktop [toggle|on|off]");
        return 1;
    };

    bar(action);
    let _ = Command::new("omarchy-hyprland-toggle")
        .args(["window-no-gaps", action])
        .status();
    0
}

// ─── toggle hybrid-gpu ────────────────────────────────────────────────────

pub fn hybrid_gpu() -> i32 {
    if run_ok("omarchy-cmd-missing", &["supergfxctl"]) {
        let _ = Command::new("omarchy-pkg-add").arg("supergfxctl").status();

        // Create config
        let conf = r#"{
  "mode": "Hybrid",
  "vfio_enable": true,
  "vfio_save": false,
  "always_reboot": false,
  "no_logind": false,
  "logout_timeout_s": 180,
  "hotplug_type": "None"
}
"#;
        let _ = {
            let mut child = Command::new("sudo")
                .args(["tee", "/etc/supergfxd.conf"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .spawn();
            if let Ok(ref mut c) = child {
                use std::io::Write;
                if let Some(ref mut stdin) = c.stdin {
                    let _ = stdin.write_all(conf.as_bytes());
                }
            }
            if let Ok(mut c) = child { let _ = c.wait(); }
        };

        let _ = Command::new("sudo")
            .args(["systemctl", "enable", "--now", "supergfxd"])
            .status();
    }

    // Get current GPU mode
    let mut gpu_mode = String::new();
    for attempt in 0..3 {
        let out = Command::new("timeout")
            .args(["--kill-after=1s", "3s", "supergfxctl", "-g"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if !out.is_empty() {
            gpu_mode = out;
            break;
        }
        if attempt < 2 {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    if gpu_mode.is_empty() {
        eprintln!("supergfxd is not responding. Try again, or check: systemctl status supergfxd");
        return 1;
    }

    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();

    match gpu_mode.as_str() {
        "Integrated" => {
            if run_ok("gum", &["confirm", "Enable dedicated GPU and reboot?"]) {
                let _ = Command::new("sudo")
                    .args(["sed", "-i", r#"s/"mode": ".*"/"mode": "Hybrid"/"#, "/etc/supergfxd.conf"])
                    .status();
                let _ = Command::new("sudo")
                    .args(["rm", "-rf", "/usr/lib/systemd/system-sleep/force-igpu"])
                    .status();
                let _ = Command::new("sudo")
                    .args(["rm", "-rf", "/etc/systemd/system/supergfxd.service.d/delay-start.conf"])
                    .status();
                let _ = Command::new("omarchy-system-reboot").status();
            }
        }
        "Hybrid" => {
            if run_ok("gum", &["confirm", "Use only integrated GPU and reboot?"]) {
                let _ = Command::new("sudo")
                    .args(["mkdir", "-p", "/etc/systemd/system/supergfxd.service.d"])
                    .status();

                let src = format!("{omarchy_path}/default/systemd/system/supergfxd.service.d/delay-start.conf");
                let ok = install_root_file(
                    &src,
                    "/etc/systemd/system/supergfxd.service.d/delay-start.conf",
                );
                if !ok {
                    eprintln!("Could not install the supergfxd startup-delay override");
                    return 1;
                }

                let _ = Command::new("sudo")
                    .args(["mkdir", "-p", "/usr/lib/systemd/system-sleep"])
                    .status();

                let src2 = format!("{omarchy_path}/default/systemd/system-sleep/force-igpu");
                let ok2 = install_root_file(&src2, "/usr/lib/systemd/system-sleep/force-igpu");
                if !ok2 {
                    eprintln!("Could not install the force-igpu system-sleep hook");
                    return 1;
                }

                let _ = Command::new("sudo")
                    .args([
                        "sed", "-i",
                        "-e", r#"s/"mode": ".*"/"mode": "Integrated"/"#,
                        "-e", r#"s/"vfio_enable": false/"vfio_enable": true/"#,
                        "/etc/supergfxd.conf",
                    ])
                    .status();

                let _ = Command::new("omarchy-system-reboot").status();
            }
        }
        _ => {
            eprintln!("Hybrid GPU not found or in unknown mode.");
            return 1;
        }
    }
    0
}

fn install_root_file(src: &str, dst: &str) -> bool {
    let stage = Command::new("sudo")
        .args(["/usr/bin/mktemp", "--", &format!("{}/.tmp.XXXXXX", std::path::Path::new(dst).parent().unwrap_or(std::path::Path::new("/tmp")).to_string_lossy())])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());
    let stage = match stage {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };
    let ok1 = Command::new("sudo")
        .args(["/usr/bin/install", "-m", "0644", "-o", "root", "-g", "root", "-T", src, &stage])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok1 {
        let _ = Command::new("sudo").args(["rm", "-f", "--", &stage]).status();
        return false;
    }
    let ok2 = Command::new("sudo")
        .args(["/usr/bin/mv", "-Tf", "--", &stage, dst])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok2 {
        let _ = Command::new("sudo").args(["rm", "-f", "--", &stage]).status();
    }
    ok2
}

// ─── toggle idle ──────────────────────────────────────────────────────────

pub fn idle(action: &str) -> i32 {
    let state_dir = PathBuf::from(home()).join(".local/state/omarchy/indicators");
    let state_file = state_dir.join("stay-awake");

    let stay_awake_enabled = || state_file.is_file();

    let apply_stay_awake = || {
        let _ = fs::create_dir_all(&state_dir);
        let _ = fs::File::create(&state_file);
    };

    let apply_allow_idle = || {
        let _ = fs::remove_file(&state_file);
    };

    let print_idle_state = || {
        if stay_awake_enabled() {
            println!("disabled");
        } else {
            println!("enabled");
        }
    };

    let print_status = || {
        if stay_awake_enabled() {
            println!(r#"{{"enabled":true,"class":"enabled","tooltip":"Allow Idle Lock & Screensaver"}}"#);
        } else {
            println!(r#"{{"enabled":false,"class":"disabled","tooltip":"Stay Awake"}}"#);
        }
    };

    match action {
        "toggle" => {
            if stay_awake_enabled() {
                apply_allow_idle();
            } else {
                apply_stay_awake();
            }
            print_idle_state();
        }
        "stay-awake" | "awake" | "on" => {
            apply_stay_awake();
            print_idle_state();
        }
        "allow-idle" | "idle" | "off" => {
            apply_allow_idle();
            print_idle_state();
        }
        "status" | "--status" => {
            print_status();
        }
        _ => {
            eprintln!("Usage: omarchy-toggle-idle [toggle|stay-awake|allow-idle|status]");
            return 1;
        }
    }
    0
}

// ─── toggle input-device ──────────────────────────────────────────────────

pub fn input_device(kind: &str, action: &str) -> i32 {
    let (label, icon) = match kind {
        "touchpad" => ("Touchpad", "touchpad"),
        "touchscreen" => ("Touchscreen", "touch"),
        _ => {
            eprintln!("Usage: omarchy-toggle-input-device <touchpad|touchscreen> [on|off|toggle]");
            return 1;
        }
    };

    let name_file = PathBuf::from(home())
        .join(".local/state/omarchy/toggles/hypr")
        .join(format!("{kind}-disabled-name"));

    let device = Command::new(format!("omarchy-hw-{kind}"))
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let require_device = || -> bool {
        if device.is_empty() {
            eprintln!("No {kind} device found");
            return false;
        }
        if device.chars().any(|c| c.is_control()) {
            eprintln!("Invalid {kind} device name");
            return false;
        }
        true
    };

    let apply_device = |enabled: bool| {
        let quoted = device.replace('\\', "\\\\").replace('"', "\\\"");
        let _ = Command::new("hyprctl")
            .args(["eval", &format!("hl.device({{ name = \"{quoted}\", enabled = {enabled} }})")])
            .stdout(Stdio::null()).status();
    };

    let do_enable = || -> i32 {
        let _ = fs::remove_file(&name_file);
        if !require_device() { return 1; }
        apply_device(true);
        let _ = Command::new("omarchy-osd")
            .args(["-i", icon, "-m", &format!("{label} enabled")])
            .status();
        0
    };

    let do_disable = || -> i32 {
        if !require_device() { return 1; }
        apply_device(false);
        if let Some(parent) = name_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&name_file, format!("{device}\n"));
        let _ = Command::new("omarchy-osd")
            .args(["-i", icon, "-m", &format!("{label} disabled")])
            .status();
        0
    };

    match action {
        "on" => do_enable(),
        "off" => do_disable(),
        "toggle" => {
            if name_file.is_file() { do_enable() } else { do_disable() }
        }
        _ => {
            eprintln!("Usage: omarchy-toggle-input-device <touchpad|touchscreen> [on|off|toggle]");
            1
        }
    }
}

// ─── toggle nightlight ────────────────────────────────────────────────────

pub fn nightlight(status_only: bool) -> i32 {
    const ON_TEMP: u32 = 4000;
    const OFF_TEMP: u32 = 6500;
    const IDENTITY_TEMP: u32 = 6000;

    let current_temp = || -> Option<u32> {
        let out = Command::new("hyprctl")
            .args(["hyprsunset", "temperature"])
            .stderr(Stdio::null())
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())?;
        out.split_whitespace()
            .find(|s| s.chars().all(|c| c.is_ascii_digit()))
            .and_then(|s| s.parse().ok())
    };

    if status_only {
        let temp = current_temp();
        let enabled = temp.map(|t| t < IDENTITY_TEMP).unwrap_or(false);
        println!(
            r#"{{"enabled":{enabled},"temperature":{}}}"#,
            temp.map(|t| t.to_string()).unwrap_or_else(|| "null".into())
        );
        return 0;
    }

    // Ensure hyprsunset is running
    if !run_ok("pgrep", &["-x", "hyprsunset"]) {
        let _ = Command::new("setsid")
            .args(["uwsm-app", "--", "hyprsunset"])
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .spawn();
    }

    let cur = current_temp();
    let target = if cur.is_none() || cur == Some(OFF_TEMP) { ON_TEMP } else { OFF_TEMP };

    // Retry until it sticks
    for _ in 0..10 {
        let _ = Command::new("hyprctl")
            .args(["hyprsunset", "temperature", &target.to_string()])
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
        std::thread::sleep(Duration::from_millis(200));
        if current_temp() == Some(target) { break; }
    }

    let _ = Command::new("omarchy-shell")
        .args(["-q", "nightlight", "refresh"])
        .stdout(Stdio::null()).stderr(Stdio::null()).status();
    0
}

// ─── toggle notification-silencing ────────────────────────────────────────

pub fn notification_silencing() -> i32 {
    let _ = Command::new("omarchy-shell")
        .args(["notifications", "toggleDnd"])
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("omarchy-shell")
        .args(["-q", "omarchy.indicators", "refresh"])
        .stdout(Stdio::null()).stderr(Stdio::null()).status();
    0
}

// ─── toggle screensaver ───────────────────────────────────────────────────

pub fn screensaver() -> i32 {
    let glyph = "\u{f1104}"; // nf-md-monitor_screenshot
    flag("screensaver-off", "toggle");
    if enabled("screensaver-off") == 0 {
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", glyph, "Screensaver disabled"])
            .status();
    } else {
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", glyph, "Screensaver enabled"])
            .status();
    }
    0
}

// ─── toggle suspend ───────────────────────────────────────────────────────

pub fn suspend() -> i32 {
    let glyph = "\u{f00b2}"; // nf-md-power_sleep
    flag("suspend-off", "toggle");
    if enabled("suspend-off") == 0 {
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", glyph, "Suspend removed from system menu"])
            .status();
    } else {
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", glyph, "Suspend now available in system menu"])
            .status();
    }
    0
}

// ─── toggle touchpad ─────────────────────────────────────────────────────

pub fn touchpad(action: &str) -> i32 {
    input_device("touchpad", action)
}

// ─── toggle touchscreen ───────────────────────────────────────────────────

pub fn touchscreen(action: &str) -> i32 {
    input_device("touchscreen", action)
}
