use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_default()
}

fn hypr_toggle_flag_path(flag_name: &str) -> PathBuf {
    PathBuf::from(home())
        .join(".local/state/omarchy/toggles/hypr")
        .join(format!("{}.lua", flag_name))
}

fn run_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn hyprctl_dispatch(lua_dispatch: &str, fallback_cmd: &str, fallback_args: &[&str]) {
    let ok = Command::new("hyprctl")
        .args(["dispatch", lua_dispatch])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        let _ = Command::new("hyprctl")
            .args(std::iter::once(fallback_cmd).chain(fallback_args.iter().copied()))
            .stdout(Stdio::null())
            .status();
    }
}

// ─── focus-app ──────────────────────────────────────────────────────────────

pub fn focus_app(app: &str) -> i32 {
    if app.is_empty() {
        eprintln!("Usage: omarchy-hyprland-focus-app <app-name>");
        return 1;
    }

    let out = Command::new("hyprctl")
        .args(["clients", "-j"])
        .stderr(Stdio::null())
        .output();
    let json = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return 1,
    };

    // Use jq for the complex pattern matching
    let address = Command::new("jq")
        .args([
            "-r",
            &format!(
                "def matches($value): ($value // \"\") | test(\"{}\"; \"i\"); \
                 first( (.[] | select(matches(.class))), \
                        (.[] | select(.initialClass == \"org.omarchy.agent\" and matches(.initialTitle))) \
                      ).address // empty",
                app.replace('"', "\\\"")
            ),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(json.as_bytes());
            }
            child.wait_with_output()
        })
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if address.is_empty() {
        return 1;
    }

    let lua = format!(
        "hl.dsp.focus({{ window = \"address:{}\" }})",
        address
    );
    let focus_fallback = format!("address:{}", address);
    hyprctl_dispatch(&lua, "focuswindow", &[&focus_fallback]);
    0
}

// ─── monitor-clamshell ──────────────────────────────────────────────────────

pub fn monitor_clamshell() -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/omarchy-hyprland-monitor-clamshell", omarchy_path);
    let err = Command::new(&script).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── monitor-external-active ────────────────────────────────────────────────

pub fn monitor_external_active() -> i32 {
    let ok = Command::new("hyprctl")
        .args(["monitors", "all", "-j"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            Command::new("jq")
                .args(["-e", ".[] | select(.name | test(\"^(eDP|LVDS|DSI)-\") | not) | select(.disabled == false)"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        let _ = stdin.write_all(&o.stdout);
                    }
                    child.wait().ok()
                })
        })
        .map(|s| s.success())
        .unwrap_or(false);
    if ok { 0 } else { 1 }
}

// ─── monitor-focused ────────────────────────────────────────────────────────

pub fn monitor_focused() -> i32 {
    let out = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .stderr(Stdio::null())
        .output();
    if let Ok(o) = out {
        if let Ok(json) = String::from_utf8(o.stdout) {
            let result = Command::new("jq")
                .args(["-r", ".[] | select(.focused == true).name"])
                .stdin(Stdio::piped())
                .stdout(Stdio::inherit())
                .stderr(Stdio::null())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        let _ = stdin.write_all(json.as_bytes());
                    }
                    child.wait()
                });
            if result.map(|s| s.success()).unwrap_or(false) {
                return 0;
            }
        }
    }
    1
}

// ─── monitor-focused-apple ───────────────────────────────────────────────────

pub fn monitor_focused_apple(monitor: Option<&str>) -> i32 {
    let out = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .stderr(Stdio::null())
        .output();
    let json = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return 1,
    };

    let filter = match monitor {
        Some(m) if !m.is_empty() => format!(
            ".[] | select(.name == \"{}\") | select(.make == \"Apple Computer Inc\" and (.model | test(\"StudioDisplay|ProDisplayXDR|Studio XDR\")))",
            m.replace('"', "\\\"")
        ),
        _ => ".[] | select(.focused == true) | select(.make == \"Apple Computer Inc\" and (.model | test(\"StudioDisplay|ProDisplayXDR|Studio XDR\")))".to_string(),
    };

    let ok = Command::new("jq")
        .args(["-e", &filter])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(json.as_bytes());
            }
            child.wait()
        })
        .map(|s| s.success())
        .unwrap_or(false);
    if ok { 0 } else { 1 }
}

// ─── monitor-internal ────────────────────────────────────────────────────────

pub fn monitor_internal(action: &str) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/omarchy-hyprland-monitor-internal", omarchy_path);
    let err = Command::new(&script).arg(action).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── monitor-internal-mirror ─────────────────────────────────────────────────

pub fn monitor_internal_mirror(action: &str) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/omarchy-hyprland-monitor-internal-mirror", omarchy_path);
    let err = Command::new(&script).arg(action).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── monitor-laptop ──────────────────────────────────────────────────────────

pub fn monitor_laptop() -> i32 {
    let out = Command::new("hyprctl")
        .args(["monitors", "all", "-j"])
        .stderr(Stdio::null())
        .output();
    let json = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return 1,
    };

    let ok = Command::new("jq")
        .args(["-r", ".[] | select(.name | test(\"^(eDP|LVDS|DSI)-\")).name"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(json.as_bytes());
            }
            child.wait()
        })
        .map(|s| s.success())
        .unwrap_or(false);
    if ok { 0 } else { 1 }
}

// ─── monitor-modeless ────────────────────────────────────────────────────────

pub fn monitor_modeless() -> i32 {
    let out = Command::new("hyprctl")
        .args(["monitors", "all", "-j"])
        .stderr(Stdio::null())
        .output();
    let json = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return 2,
    };

    let result = Command::new("jq")
        .args(["if any(.[]; .disabled != true and (.width == 0 or .height == 0)) then 0 else 1 end"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(json.as_bytes());
            }
            child.wait_with_output()
        });

    match result {
        Ok(o) => {
            let state = String::from_utf8_lossy(&o.stdout).trim().to_string();
            match state.as_str() {
                "0" => 0,
                "1" => 1,
                _ => 2,
            }
        }
        _ => 2,
    }
}

// ─── monitor-scaling ─────────────────────────────────────────────────────────

pub fn monitor_scaling(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/omarchy-hyprland-monitor-scaling", omarchy_path);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── monitor-watch ───────────────────────────────────────────────────────────

pub fn monitor_watch() -> i32 {
    let socket = {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
        let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default();
        format!("{}/hypr/{}/.socket2.sock", runtime_dir, sig)
    };

    let lock_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let lock_path = format!("{}/omarchy-monitor-clamshell.lock", lock_dir);
    let modeless_lock_path = format!("{}/omarchy-monitor-modeless.lock", lock_dir);

    let sync_clamshell = || {
        let _ = Command::new("omarchy-hyprland-monitor-clamshell")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    };

    let lock_path_clone = lock_path.clone();
    let modeless_lock_path_clone = modeless_lock_path.clone();

    let sync_clamshell_after_monitor_change = move || {
        let _ = Command::new("omarchy-hyprland-monitor-clamshell")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        let lp = lock_path_clone.clone();
        let mlp = modeless_lock_path_clone.clone();
        std::thread::spawn(move || {
            for delay in [1u64, 3, 7] {
                std::thread::sleep(std::time::Duration::from_secs(delay));
                let _ = Command::new("omarchy-hyprland-monitor-clamshell")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                let _ = lp.as_str(); // keep in scope
                let _ = mlp.as_str();
            }
        });
    };

    // Initial sync
    sync_clamshell();
    sync_clamshell_after_monitor_change();

    // Start socat to read events
    let mut socat = match Command::new("socat")
        .args(["-U", "-", &format!("UNIX-CONNECT:{}", socket)])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to start socat: {}", e);
            return 1;
        }
    };

    let stdout = socat.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let event = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if event.starts_with("monitoradded>>")
            || event.starts_with("monitoraddedv2>>")
            || event.starts_with("monitorremoved>>")
            || event.starts_with("monitorremovedv2>>")
        {
            let _ = Command::new("omarchy-hyprland-monitor-clamshell")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            // spawn delayed syncs
            let lp = lock_path.clone();
            let mlp = modeless_lock_path.clone();
            std::thread::spawn(move || {
                for delay in [1u64, 3, 7] {
                    std::thread::sleep(std::time::Duration::from_secs(delay));
                    let _ = Command::new("omarchy-hyprland-monitor-clamshell")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                    let _ = lp.as_str();
                    let _ = mlp.as_str();
                }
            });
        }
    }

    let _ = socat.wait();
    0
}

// ─── reload-guard ────────────────────────────────────────────────────────────

pub fn reload_guard(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/omarchy-hyprland-reload-guard", omarchy_path);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── session-locked ──────────────────────────────────────────────────────────

pub fn session_locked() -> i32 {
    let out = Command::new("hyprctl")
        .args(["-j", "monitors"])
        .stderr(Stdio::null())
        .output();
    let json = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return 2,
    };

    let result = Command::new("jq")
        .args([
            "def blockers: .solitaryBlockedBy // []; \
             def readable: blockers | index(\"WORKSPACE\") | not; \
             if   any(.[]; blockers | index(\"LOCK\")) then 0 \
             elif any(.[]; readable)                  then 1 \
             else                                          2 \
             end",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(json.as_bytes());
            }
            child.wait_with_output()
        });

    match result {
        Ok(o) => {
            let state = String::from_utf8_lossy(&o.stdout).trim().to_string();
            match state.as_str() {
                "0" => 0,
                "1" => 1,
                _ => 2,
            }
        }
        _ => 2,
    }
}

// ─── toggle ──────────────────────────────────────────────────────────────────

pub fn toggle(flag_name: &str, action: &str) -> i32 {
    if flag_name.is_empty() {
        eprintln!("Usage: omarchy-hyprland-toggle <flag-name> [on|off|toggle]");
        return 1;
    }

    let flag_file = hypr_toggle_flag_path(flag_name);
    let omarchy_path = omarchy_path();
    let flag_source = PathBuf::from(&omarchy_path)
        .join("default/hypr/toggles")
        .join(format!("{}.lua", flag_name));

    let do_on = || -> i32 {
        if flag_source.exists() {
            if let Some(parent) = flag_file.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::copy(&flag_source, &flag_file) {
                eprintln!("Failed to copy flag: {}", e);
                return 1;
            }
            0
        } else {
            eprintln!("Flag not found: {}", flag_name);
            1
        }
    };

    let do_off = || -> i32 {
        let _ = fs::remove_file(&flag_file);
        0
    };

    let result = match action {
        "on" => do_on(),
        "off" => do_off(),
        "toggle" => {
            if flag_file.exists() {
                let r = do_off();
                println!("off");
                r
            } else {
                let r = do_on();
                if r == 0 {
                    println!("on");
                }
                r
            }
        }
        _ => {
            eprintln!("Usage: omarchy-hyprland-toggle <flag-name> [on|off|toggle]");
            return 1;
        }
    };

    let _ = Command::new("hyprctl")
        .arg("reload")
        .stdout(Stdio::null())
        .status();

    result
}

// ─── toggle-disabled ─────────────────────────────────────────────────────────

pub fn toggle_disabled(flag_name: &str) -> i32 {
    if !hypr_toggle_flag_path(flag_name).exists() { 0 } else { 1 }
}

// ─── toggle-enabled ──────────────────────────────────────────────────────────

pub fn toggle_enabled(flag_name: &str) -> i32 {
    if hypr_toggle_flag_path(flag_name).exists() { 0 } else { 1 }
}

// ─── window-close-all ────────────────────────────────────────────────────────

pub fn window_close_all() -> i32 {
    let out = Command::new("hyprctl")
        .args(["clients", "-j"])
        .stderr(Stdio::null())
        .output();
    let json = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return 1,
    };

    // Extract addresses with jq
    let addresses = Command::new("jq")
        .args(["-r", ".[].address"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(json.as_bytes());
            }
            child.wait_with_output()
        })
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    for addr in addresses.lines() {
        let addr = addr.trim();
        if addr.is_empty() {
            continue;
        }
        let lua = format!("hl.dsp.window.close({{ window = \"address:{}\" }})", addr);
        let _ = Command::new("hyprctl")
            .args(["dispatch", &lua])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    // Move to first workspace
    let ok = run_ok("hyprctl", &["dispatch", "hl.dsp.focus({ workspace = \"1\" })"]);
    if !ok {
        let _ = Command::new("hyprctl")
            .args(["dispatch", "workspace", "1"])
            .stdout(Stdio::null())
            .status();
    }
    0
}

// ─── window-gaps-toggle ──────────────────────────────────────────────────────

pub fn window_gaps_toggle() -> i32 {
    toggle("window-no-gaps", "toggle")
}

// ─── window-pop ──────────────────────────────────────────────────────────────

pub fn window_pop(args: &[String]) -> i32 {
    let width = args.first().map(|s| s.as_str()).unwrap_or("1300");
    let height = args.get(1).map(|s| s.as_str()).unwrap_or("900");
    let x = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let y = args.get(3).map(|s| s.as_str()).unwrap_or("");

    let active = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let pinned = Command::new("jq")
        .args([".pinned"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(active.as_bytes());
            }
            child.wait_with_output()
        })
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let addr = Command::new("jq")
        .args(["-r", ".address"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(active.as_bytes());
            }
            child.wait_with_output()
        })
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let window = format!("address:{}", addr);

    if pinned == "true" {
        hyprctl_dispatch(
            &format!("hl.dsp.window.pin({{ window = \"{}\" }})", window),
            "pin",
            &[&window],
        );
        hyprctl_dispatch(
            &format!("hl.dsp.window.float({{ window = \"{}\", action = \"toggle\" }})", window),
            "togglefloating",
            &[&window],
        );
        hyprctl_dispatch(
            &format!("hl.dsp.window.tag({{ window = \"{}\", tag = \"-pop\" }})", window),
            "tagwindow",
            &["-pop", &window],
        );
    } else if !addr.is_empty() {
        hyprctl_dispatch(
            &format!("hl.dsp.window.float({{ window = \"{}\", action = \"toggle\" }})", window),
            "togglefloating",
            &[&window],
        );
        hyprctl_dispatch(
            &format!("hl.dsp.window.resize({{ window = \"{}\", x = {}, y = {} }})", window, width, height),
            "resizeactive",
            &["exact", width, height, &window],
        );

        if !x.is_empty() && !y.is_empty() {
            hyprctl_dispatch(
                &format!("hl.dsp.window.move({{ window = \"{}\", x = {}, y = {} }})", window, x, y),
                "moveactive",
                &[x, y, &window],
            );
        } else {
            hyprctl_dispatch(
                &format!("hl.dsp.window.center({{ window = \"{}\" }})", window),
                "centerwindow",
                &[&window],
            );
        }

        hyprctl_dispatch(
            &format!("hl.dsp.window.pin({{ window = \"{}\" }})", window),
            "pin",
            &[&window],
        );
        hyprctl_dispatch(
            &format!("hl.dsp.window.alter_zorder({{ window = \"{}\", mode = \"top\" }})", window),
            "alterzorder",
            &["top", &window],
        );
        hyprctl_dispatch(
            &format!("hl.dsp.window.tag({{ window = \"{}\", tag = \"+pop\" }})", window),
            "tagwindow",
            &["+pop", &window],
        );
    }
    0
}

// ─── window-single-square-aspect-toggle ──────────────────────────────────────

pub fn window_single_square_aspect_toggle() -> i32 {
    let out = Command::new("omarchy-hyprland-toggle")
        .arg("single-window-aspect-ratio")
        .output();
    let state = out
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    match state.as_str() {
        "on" => {
            let _ = Command::new("omarchy-notification-send")
                .args(["-g", "\u{f02f8}", "Enable single-window square aspect ratio"])
                .status();
        }
        "off" => {
            let _ = Command::new("omarchy-notification-send")
                .args(["-g", "\u{f02f8}", "Disable single-window square aspect ratio"])
                .status();
        }
        _ => {}
    }
    0
}

// ─── window-tiled-fullscreen-toggle ──────────────────────────────────────────

pub fn window_tiled_fullscreen_toggle() -> i32 {
    let active = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let fs_client = Command::new("jq")
        .args(["-r", ".fullscreenClient // 0"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(active.as_bytes());
            }
            child.wait_with_output()
        })
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if fs_client == "2" {
        hyprctl_dispatch(
            "hl.dsp.window.fullscreen_state({ internal = 0, client = 0 })",
            "fullscreenstate",
            &["0", "0"],
        );
    } else {
        hyprctl_dispatch(
            "hl.dsp.window.fullscreen_state({ internal = 0, client = 2 })",
            "fullscreenstate",
            &["0", "2"],
        );
    }
    0
}

// ─── window-transparency-toggle ──────────────────────────────────────────────

pub fn window_transparency_toggle() -> i32 {
    let addr = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            Command::new("jq")
                .args(["-r", ".address"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .ok()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        let _ = stdin.write_all(&o.stdout);
                    }
                    child.wait_with_output().ok()
                })
        })
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let lua = format!(
        "hl.dsp.window.set_prop({{ window = \"address:{}\", prop = \"opaque\", value = \"toggle\" }})",
        addr
    );
    let fallback_addr = format!("address:{}", addr);
    hyprctl_dispatch(&lua, "setprop", &[&fallback_addr, "opaque", "toggle"]);
    0
}

// ─── window-width ────────────────────────────────────────────────────────────

pub fn window_width(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/omarchy-hyprland-window-width", omarchy_path);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── workspace-layout-toggle ─────────────────────────────────────────────────

pub fn workspace_layout_toggle() -> i32 {
    let workspace_json = Command::new("hyprctl")
        .args(["activeworkspace", "-j"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let workspace_id = Command::new("jq")
        .args(["-r", ".id"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(workspace_json.as_bytes());
            }
            child.wait_with_output()
        })
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if workspace_id.is_empty() || !workspace_id.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return 1;
    }

    let current_layout = Command::new("jq")
        .args(["-r", ".tiledLayout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(workspace_json.as_bytes());
            }
            child.wait_with_output()
        })
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let new_layout = if current_layout == "dwindle" { "scrolling" } else { "dwindle" };

    let layouts_dir = PathBuf::from(home()).join(".local/state/omarchy/workspace-layouts");
    let _ = fs::create_dir_all(&layouts_dir);
    let layout_file = layouts_dir.join(format!("{}.lua", workspace_id));
    let lua_content = format!(
        "hl.workspace_rule({{ workspace = \"{}\", layout = \"{}\" }})\n",
        workspace_id, new_layout
    );
    let _ = fs::write(&layout_file, &lua_content);

    let eval_lua = format!(
        "hl.workspace_rule({{ workspace = \"{}\", layout = \"{}\" }})",
        workspace_id, new_layout
    );
    let ok = run_ok("hyprctl", &["eval", &eval_lua]);
    if !ok {
        let _ = Command::new("hyprctl")
            .args(["keyword", "workspace", &format!("{}, layout:{}", workspace_id, new_layout)])
            .stdout(Stdio::null())
            .status();
    }

    let _ = Command::new("omarchy-notification-send")
        .args(["-g", "\u{f02ac}", &format!("Workspace layout set to {}", new_layout)])
        .status();

    0
}
