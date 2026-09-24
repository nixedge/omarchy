use std::process::{Command, Stdio};

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_default()
}

fn exec_delegate(script_name: &str, args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/{}", omarchy_path, script_name);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── factory-reset ───────────────────────────────────────────────────────────

pub fn factory_reset(args: &[String]) -> i32 {
    exec_delegate("omarchy-system-factory-reset", args)
}

// ─── factory-reset-finish ────────────────────────────────────────────────────

pub fn factory_reset_finish(args: &[String]) -> i32 {
    exec_delegate("omarchy-system-factory-reset-finish", args)
}

// ─── lid-close ───────────────────────────────────────────────────────────────

pub fn lid_close() -> i32 {
    // If lid is closed and no external monitors, lock
    let closed = Command::new("omarchy-hw-laptop-closed")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let external = Command::new("omarchy-hw-external-monitors")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if closed && !external {
        let _ = Command::new("omarchy-system-lock")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    let _ = Command::new("omarchy-hyprland-monitor-clamshell")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    0
}

// ─── lock ────────────────────────────────────────────────────────────────────

pub fn lock() -> i32 {
    let _ = Command::new("omarchy-shell")
        .args(["lock", "lock"])
        .stdout(Stdio::null())
        .status();

    // Set keyboard layout to default (first layout)
    let _ = Command::new("hyprctl")
        .args(["switchxkblayout", "all", "0"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // Lock 1password if running
    let onepassword_running = Command::new("pgrep")
        .args(["-x", "1password"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let onepassword_present = Command::new("omarchy-cmd-present")
        .arg("1password")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if onepassword_running && onepassword_present {
        std::thread::spawn(|| {
            let runtime_dir =
                std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
            let lock_path = format!("{}/omarchy-1password-lock.lock", runtime_dir);
            let lock_file = std::fs::File::create(&lock_path);
            if let Ok(_lf) = lock_file {
                let _ = Command::new("timeout")
                    .args(["--kill-after=1s", "3s", "1password", "--lock"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        });
    }

    // Kill screensaver processes
    let _ = Command::new("pkill")
        .args(["-x", "ttfx"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("timeout")
        .args(["1s", "pidwait", "-x", "ttfx"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("pkill")
        .args(["-f", "[o]rg.omarchy.screensaver"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    0
}

// ─── logout ──────────────────────────────────────────────────────────────────

pub fn logout() -> i32 {
    // Schedule logout in background
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let _ = Command::new("uwsm").arg("stop").status();
    });

    let _ = Command::new("omarchy-osd")
        .args(["-i", "logout", "-m", "Logging out", "-d", "5000"])
        .status();

    let _ = Command::new("omarchy-hyprland-window-close-all").status();
    std::thread::sleep(std::time::Duration::from_secs(1));
    0
}

// ─── reboot ──────────────────────────────────────────────────────────────────

pub fn reboot() -> i32 {
    let ok = Command::new("systemd-run")
        .args([
            "--user",
            "--collect",
            "--quiet",
            "--on-active=2s",
            "--timer-property=AccuracySec=100ms",
            "systemctl",
            "reboot",
            "--no-wall",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok {
        return 1;
    }

    let _ = Command::new("omarchy-osd")
        .args(["-i", "reboot", "-m", "Rebooting", "-d", "5000"])
        .status();

    let _ = Command::new("omarchy-state")
        .args(["clear", "re*-required"])
        .status();

    let _ = Command::new("omarchy-hyprland-window-close-all").status();
    std::thread::sleep(std::time::Duration::from_secs(1));
    0
}

// ─── shutdown ────────────────────────────────────────────────────────────────

pub fn shutdown() -> i32 {
    let ok = Command::new("systemd-run")
        .args([
            "--user",
            "--collect",
            "--quiet",
            "--on-active=2s",
            "--timer-property=AccuracySec=100ms",
            "systemctl",
            "poweroff",
            "--no-wall",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok {
        return 1;
    }

    let _ = Command::new("omarchy-osd")
        .args(["-i", "shutdown", "-m", "Shutting down", "-d", "5000"])
        .status();

    let _ = Command::new("omarchy-state")
        .args(["clear", "re*-required"])
        .status();

    let _ = Command::new("omarchy-hyprland-window-close-all").status();
    std::thread::sleep(std::time::Duration::from_secs(1));
    0
}

// ─── sleep-lock ──────────────────────────────────────────────────────────────

pub fn sleep_lock(args: &[String]) -> i32 {
    exec_delegate("omarchy-system-sleep-lock", args)
}

// ─── sleep-monitor ───────────────────────────────────────────────────────────

pub fn sleep_monitor(args: &[String]) -> i32 {
    exec_delegate("omarchy-system-sleep-monitor", args)
}

// ─── stats ───────────────────────────────────────────────────────────────────

pub fn stats(args: &[String]) -> i32 {
    match args.first().map(|s| s.as_str()) {
        Some("--bar-widget") => {
            bar_widget_stats();
            0
        }
        None | Some("") => {
            run_tui_stats()
        }
        _ => {
            eprintln!("Usage: omarchy-system-stats [--bar-widget]");
            1
        }
    }
}

fn bar_widget_stats() {
    // Read CPU stats from /proc/stat
    if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
        if let Some(line) = stat.lines().next() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() > 5 {
                let idle = fields[5];
                let total: u64 = fields[1..].iter()
                    .filter_map(|v| v.parse::<u64>().ok())
                    .sum();
                println!("cpu\t{}\t{}", idle, total);
            }
        }
    }

    // Read memory from /proc/meminfo
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        let mut total = 0u64;
        let mut avail = 0u64;
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                avail = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
            }
        }
        if total > 0 {
            let used_pct = ((total - avail) as f64 / total as f64) * 100.0;
            println!("memory\t{:.2}", used_pct);
        }
    }

    // Read load from /proc/loadavg
    if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(load) = loadavg.split_whitespace().next() {
            println!("load\t{}", load);
        }
    }
}

fn run_tui_stats() -> i32 {
    // Try btop first, then htop, then top
    for cmd in &["btop", "htop", "top"] {
        let present = Command::new("which")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if present {
            use std::os::unix::process::CommandExt;
            let err = Command::new(cmd).exec();
            eprintln!("exec {}: {}", cmd, err);
            return 1;
        }
    }
    eprintln!("No stats TUI found (btop, htop, or top required)");
    1
}

// ─── wake ────────────────────────────────────────────────────────────────────

pub fn wake() -> i32 {
    let _ = Command::new("omarchy-brightness-display").arg("on").status();
    let _ = Command::new("omarchy-brightness-keyboard").arg("restore").status();
    let _ = Command::new("omarchy-hyprland-monitor-clamshell")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    0
}
