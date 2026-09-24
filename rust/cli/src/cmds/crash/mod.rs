use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn mutes_dir() -> PathBuf {
    PathBuf::from(home()).join(".local/state/omarchy/toggles/crash-ignore")
}

fn toggle_flag_path(flag_name: &str) -> PathBuf {
    PathBuf::from(home())
        .join(".local/state/omarchy/toggles")
        .join(flag_name)
}

fn flag_enabled(flag_name: &str) -> bool {
    toggle_flag_path(flag_name).is_file()
}

// ─── crash mute ───────────────────────────────────────────────────────────────

pub fn mute(program: Option<&str>, action: &str) -> i32 {
    match program {
        None => {
            // List muted programs
            let dir = mutes_dir();
            let mut found = false;
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if entry.path().is_file() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        println!("{}", name_str);
                        found = true;
                    }
                }
            }
            // Also check dotfiles
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with('.') && name_str != "." && name_str != ".."
                        && entry.path().is_file()
                    {
                        println!("{}", name_str);
                        found = true;
                    }
                }
            }
            if !found {
                println!("No programs muted. Crashes all notify.");
            }
            0
        }
        Some(prog) => {
            // Strip path prefix (basename)
            let program_name = prog.rfind('/').map(|i| &prog[i+1..]).unwrap_or(prog);

            if program_name.is_empty() || program_name == "." || program_name == ".." {
                eprintln!("Not a program name: {}", prog);
                eprintln!("Usage: omarchy crash mute [--] [<program>] [on|off|toggle]");
                return 1;
            }

            match action {
                "on" | "off" | "toggle" => {}
                _ => {
                    eprintln!("Not an action: {}", action);
                    eprintln!("Usage: omarchy crash mute [--] [<program>] [on|off|toggle]");
                    return 1;
                }
            }

            let flag_name = format!("crash-ignore/{}", program_name);
            let flag_path = toggle_flag_path(&flag_name);

            match action {
                "on" => {
                    if let Some(parent) = flag_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::File::create(&flag_path);
                }
                "off" => {
                    let _ = std::fs::remove_file(&flag_path);
                }
                "toggle" => {
                    if flag_path.exists() {
                        let _ = std::fs::remove_file(&flag_path);
                    } else {
                        if let Some(parent) = flag_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::File::create(&flag_path);
                    }
                }
                _ => {}
            }

            if flag_enabled(&flag_name) {
                println!("Muted crash notifications for {}.", program_name);
            } else {
                println!("Crash notifications for {} are back on.", program_name);
            }
            0
        }
    }
}

// ─── crash watch ─────────────────────────────────────────────────────────────

pub fn watch() -> i32 {
    const COREDUMP_MESSAGE_ID: &str = "fc2e22bc6ee647b6b90729ab34a250b1";
    const CRASH_GLYPH: &str = "\u{f16a1}";

    let dedupe_seconds: u64 = std::env::var("OMARCHY_CRASH_DEDUPE_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    let ignore_pattern = std::env::var("OMARCHY_CRASH_IGNORE").unwrap_or_default();

    let uid = unsafe { libc_getuid() };

    let mut last_notified: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

    let mut journalctl = match Command::new("journalctl")
        .args(["-f", "-n", "0", "-o", "json", &format!("MESSAGE_ID={}", COREDUMP_MESSAGE_ID)])
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to start journalctl: {}", e);
            return 1;
        }
    };

    let stdout = journalctl.stdout.take().unwrap();
    let reader = std::io::BufReader::new(stdout);

    for line in reader.lines() {
        let entry = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        // Parse fields from JSON using jq
        let fields = Command::new("jq")
            .args([
                "-r",
                r#"def field: if . == null or . == "" then "-" else . end;
                   [(._UID | field),
                    (.COREDUMP_COMM | field),
                    (.COREDUMP_PID | field),
                    (.COREDUMP_EXE | field),
                    (.COREDUMP_SIGNAL_NAME | field)] | @tsv"#,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()
            .and_then(|mut c| {
                use std::io::Write;
                if let Some(ref mut stdin) = c.stdin {
                    let _ = stdin.write_all(entry.as_bytes());
                }
                c.wait_with_output().ok()
            })
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let parts: Vec<&str> = fields.split('\t').collect();
        if parts.len() < 5 {
            continue;
        }

        let (entry_uid, comm, pid, exe, signal) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

        // Validate PID
        if !pid.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        // Check if a default agent is set
        let agent = Command::new("omarchy-default-agent")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if agent.is_empty() {
            continue;
        }

        // Only this user's crashes
        if !entry_uid.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let entry_uid_num: u32 = entry_uid.parse().unwrap_or(u32::MAX);
        if entry_uid_num != uid {
            continue;
        }

        // Compute name (prefer exe basename)
        let mut name = comm.to_string();
        if exe.starts_with('/') {
            if let Some(base) = exe.rfind('/') {
                name = exe[base + 1..].to_string();
            }
        }
        // Keep only the basename part
        if let Some(base) = name.rfind('/') {
            name = name[base + 1..].to_string();
        }

        // Sanitize name
        if name.is_empty() || name == "-" || name == "." || name == ".." {
            name = "unknown".to_string();
        }

        // Check ignore pattern
        if !ignore_pattern.is_empty() {
            // Simple regex check via grep
            let matched = Command::new("grep")
                .args(["-qE", &ignore_pattern])
                .stdin(Stdio::piped())
                .spawn()
                .ok()
                .and_then(|mut c| {
                    use std::io::Write;
                    if let Some(ref mut stdin) = c.stdin {
                        let _ = stdin.write_all(name.as_bytes());
                    }
                    c.wait().ok()
                })
                .map(|s| s.success())
                .unwrap_or(false);
            if matched {
                continue;
            }
        }

        // Skip our own machinery
        if name.starts_with("omarchy-crash-") || name.starts_with("omarchy-agent-") {
            continue;
        }

        // Check if muted
        let flag_name = format!("crash-ignore/{}", name);
        if flag_enabled(&flag_name) {
            continue;
        }

        // Dedupe check
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(&last) = last_notified.get(&name) {
            if now - last < dedupe_seconds {
                continue;
            }
        }

        // Wait for notification server
        let wait_ok = Command::new("omarchy-notification-wait")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !wait_ok {
            continue;
        }

        let pid_str = pid.to_string();
        let comm_str = comm.to_string();
        let exe_str = exe.to_string();
        let signal_str = signal.to_string();

        let status = Command::new("omarchy-notification-send")
            .args([
                "--urgency", "critical",
                "--glyph", CRASH_GLYPH,
                &format!("Process crashed: {}", name),
                "Click to diagnose with AI",
                "--exec", "omarchy-agent-crash", &pid_str, &comm_str, &exe_str, &signal_str,
            ])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if status {
            last_notified.insert(name, now);
        }
    }

    let _ = journalctl.wait();
    0
}

fn libc_getuid() -> u32 {
    unsafe {
        extern "C" {
            fn getuid() -> u32;
        }
        getuid()
    }
}
