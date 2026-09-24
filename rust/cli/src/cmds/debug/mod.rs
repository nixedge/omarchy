use std::process::{Command, Stdio};

// ─── debug ─────────────────────────────────────────────────────────────────────

pub fn run(no_sudo: bool, print_only: bool) -> i32 {
    let log_file = "/tmp/omarchy-debug.log";

    let dmesg_output = if no_sudo {
        "(skipped - --no-sudo flag used)".to_string()
    } else {
        Command::new("sudo")
            .arg("dmesg")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|| "(dmesg failed)".to_string())
    };

    let date_out = Command::new("date")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let hostname_out = Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Try to get package info
    let pkg_out = Command::new("pacman")
        .args(["-Q", "omarchy-dev"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .or_else(|| {
            Command::new("pacman")
                .args(["-Q", "omarchy"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let inxi_out = Command::new("inxi")
        .args(["-Farz"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "(inxi not available)".to_string());

    let journalctl_out = Command::new("journalctl")
        .args(["-b", "-p", "4..1"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    // Installed packages - skip the expac/pacman complex commands on NixOS
    let packages_out = "(package list not available on NixOS)".to_string();

    let log_content = format!(
        "Date: {}\n\
Hostname: {}\n\
Omarchy Package: {}\n\
\n\
=========================================\n\
SYSTEM INFORMATION\n\
=========================================\n\
{}\n\
\n\
=========================================\n\
DMESG\n\
=========================================\n\
{}\n\
\n\
=========================================\n\
JOURNALCTL (CURRENT BOOT, WARNINGS+ERRORS)\n\
=========================================\n\
{}\n\
\n\
=========================================\n\
INSTALLED PACKAGES\n\
=========================================\n\
{}\n",
        date_out, hostname_out, pkg_out, inxi_out, dmesg_output, journalctl_out, packages_out
    );

    if let Err(e) = std::fs::write(log_file, &log_content) {
        eprintln!("Failed to write log: {}", e);
        return 1;
    }

    if print_only {
        print!("{}", log_content);
        return 0;
    }

    // Check internet connectivity
    let has_internet = Command::new("ping")
        .args(["-c", "1", "8.8.8.8"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let mut options = vec!["View log", "Save in current directory"];
    if has_internet {
        options.insert(0, "Upload log");
    }

    let action = Command::new("gum")
        .arg("choose")
        .args(&options)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    match action.as_str() {
        "Upload log" => {
            println!("Uploading debug log to logs.omarchy.org...");
            let out = Command::new("curl")
                .args(["-sf", "-F", &format!("file=@{}", log_file), "-Fexpires=24", "https://logs.omarchy.org/"])
                .output();
            match out {
                Ok(o) if o.status.success() => {
                    let url = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !url.is_empty() {
                        println!("✓ Log uploaded successfully!");
                        println!("Share this URL:");
                        println!();
                        println!("  {}", url);
                    } else {
                        eprintln!("Error: Failed to upload log file");
                        return 1;
                    }
                }
                _ => {
                    eprintln!("Error: Failed to upload log file");
                    return 1;
                }
            }
        }
        "View log" => {
            let _ = Command::new("less").arg(log_file).status();
        }
        "Save in current directory" => {
            let dest = "./omarchy-debug.log";
            if let Err(e) = std::fs::copy(log_file, dest) {
                eprintln!("Failed to save: {}", e);
                return 1;
            }
            if let Ok(cwd) = std::env::current_dir() {
                println!("✓ Log saved to {}/omarchy-debug.log", cwd.display());
            }
        }
        _ => {}
    }

    0
}

// ─── debug idle ────────────────────────────────────────────────────────────────

pub fn idle(log_lines: u64) -> i32 {
    fn section(title: &str) {
        println!("\n== {} ==", title);
    }

    section("Time");
    let _ = Command::new("date").arg("-Is").status();

    section("Idle IPC status");
    let out = Command::new("omarchy-shell")
        .args(["idle", "status"])
        .output();
    match out {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout).to_string()
                + &String::from_utf8_lossy(&o.stderr);
            // Try to pretty-print as JSON
            let jq = Command::new("jq")
                .arg(".")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn();
            let pretty = jq.ok().and_then(|mut c| {
                use std::io::Write;
                if let Some(ref mut stdin) = c.stdin {
                    let _ = stdin.write_all(text.as_bytes());
                }
                c.wait_with_output().ok()
            }).filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok());
            if let Some(p) = pretty {
                print!("{}", p);
            } else {
                print!("{}", text);
            }
        }
        Err(_) => {}
    }

    section("Quickshell instances");
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let _ = Command::new("quickshell")
        .args(["list", "-p", &format!("{}/shell", omarchy_path), "--any-display"])
        .status();

    section("Recent idle logs");
    let _ = Command::new("quickshell")
        .args([
            "--no-color", "log",
            "-p", &format!("{}/shell", omarchy_path),
            "--any-display",
            "--tail", &log_lines.to_string(),
            "--log-times",
            "-r", "quickshell.wayland.idle_notify=true",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .ok()
        .and_then(|c| {
            Command::new("grep")
                .args(["-Ei", "omarchy idle|idle_notify|screensaver|lock|error|warn|failed"])
                .stdin(c.stdout.unwrap())
                .status()
                .ok()
        });

    section("Persisted shell log");
    let _ = Command::new("journalctl")
        .args(["-t", "omarchy-shell", "-n", &log_lines.to_string(), "--no-pager", "--quiet"])
        .status();

    section("Relevant processes");
    let _ = Command::new("sh")
        .args([
            "-c",
            r#"ps -eo pid=,args= | grep -E 'quickshell -n -p|omarchy-system-sleep-monitor|systemd-inhibit.*Lock screen before suspend|org\.omarchy\.screensaver|omarchy-screensaver|(^|/| )ttfx( |$)' | grep -v grep || true"#,
        ])
        .status();

    section("Sleep lock service");
    let _ = Command::new("systemctl")
        .args(["--user", "status", "omarchy-sleep-lock.service", "--no-pager"])
        .status();

    section("Hyprland screensaver clients");
    let _ = Command::new("sh")
        .args([
            "-c",
            r#"hyprctl clients -j 2>/dev/null | jq -r '.[] | select(.class == "org.omarchy.screensaver" or .initialClass == "org.omarchy.screensaver") | [.pid,.class,.initialClass,.title,.focusHistoryID] | @tsv' || true"#,
        ])
        .status();

    section("Idle inhibitors");
    let _ = Command::new("sh")
        .args([
            "-c",
            r#"hyprctl clients -j 2>/dev/null | jq -r '.[] | select(.inhibitingIdle == true or ((.tags // []) | index("noidle"))) | [.pid,.class,.title,((.tags // []) | join(",")), .inhibitingIdle] | @tsv' || true"#,
        ])
        .status();

    section("Screensaver detector");
    let screensaver_client = Command::new("sh")
        .args(["-c", r#"hyprctl clients -j 2>/dev/null | jq -e '.[] | select(.class == "org.omarchy.screensaver" or .initialClass == "org.omarchy.screensaver")' >/dev/null 2>&1 && echo running-window"#])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if screensaver_client {
        println!("running-window");
    } else {
        let running_proc = Command::new("pgrep")
            .args(["-f", "[o]rg.omarchy.screensaver"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if running_proc {
            println!("running-process");
        } else {
            let disabled = Command::new("omarchy-toggle-enabled")
                .arg("screensaver-off")
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if disabled {
                println!("disabled");
            } else {
                println!("stopped");
            }
        }
    }

    section("Lock detector");
    let lock_out = Command::new("omarchy-shell")
        .args(["lock", "status"])
        .output();
    match lock_out {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout).to_string()
                + &String::from_utf8_lossy(&o.stderr);
            let jq = Command::new("jq")
                .arg(".")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn();
            let pretty = jq.ok().and_then(|mut c| {
                use std::io::Write;
                if let Some(ref mut stdin) = c.stdin {
                    let _ = stdin.write_all(text.as_bytes());
                }
                c.wait_with_output().ok()
            }).filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok());
            if let Some(p) = pretty {
                print!("{}", p);
            } else {
                print!("{}", text);
            }
        }
        Err(_) => {}
    }

    0
}
