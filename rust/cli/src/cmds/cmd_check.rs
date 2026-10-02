use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

pub fn missing(cmds: &[String]) -> i32 {
    // Return 0 if ANY command is missing
    for cmd in cmds {
        if !command_in_path(cmd) {
            return 0;
        }
    }
    1
}

pub fn present(cmds: &[String]) -> i32 {
    // Return 0 if ALL commands are present
    for cmd in cmds {
        if !command_in_path(cmd) {
            return 1;
        }
    }
    0
}

pub fn terminal_cwd() -> i32 {
    // Get CWD of active terminal via hyprctl + kitty socket or /proc
    let terminal_pid = get_active_window_pid();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

    let cwd = if let Some(pid) = terminal_pid {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/tmp".to_string());
        let kitty_socket = format!("{runtime_dir}/omarchy-kitty-{pid}");

        if std::path::Path::new(&kitty_socket).exists() {
            // Try kitty IPC
            kitty_cwd(&kitty_socket)
                .unwrap_or_else(|| proc_cwd(pid, &home))
        } else {
            proc_cwd(pid, &home)
        }
    } else {
        home.clone()
    };

    if std::path::Path::new(&cwd).is_dir() {
        println!("{cwd}");
    } else {
        println!("{home}");
    }
    0
}

fn command_in_path(cmd: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }
    // Absolute or relative path — check executability directly
    if cmd.contains('/') {
        let p = std::path::Path::new(cmd);
        return p.is_file()
            && std::fs::metadata(p)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false);
    }
    // Search PATH directories for an executable named exactly `cmd`
    let path_env = std::env::var("PATH").unwrap_or_default();
    for dir in path_env.split(':') {
        if dir.is_empty() {
            continue;
        }
        let candidate = std::path::Path::new(dir).join(cmd);
        if candidate.is_file()
            && std::fs::metadata(&candidate)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

fn get_active_window_pid() -> Option<String> {
    let out = Command::new("hyprctl")
        .arg("activewindow")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pid:") {
            return trimmed.split_whitespace().nth(1).map(|s| s.to_string());
        }
    }
    None
}

fn kitty_cwd(socket: &str) -> Option<String> {
    let out = Command::new("kitten")
        .args(["@", "--to", &format!("unix:{socket}"), "ls", "--match", "state:focused"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let val: serde_json::Value = serde_json::from_str(&stdout).ok()?;

    // .[].tabs[].windows[].cwd
    let arr = val.as_array()?;
    for item in arr {
        if let Some(tabs) = item.get("tabs").and_then(|t| t.as_array()) {
            for tab in tabs {
                if let Some(windows) = tab.get("windows").and_then(|w| w.as_array()) {
                    for win in windows {
                        if let Some(cwd) = win.get("cwd").and_then(|c| c.as_str()) {
                            if !cwd.is_empty() {
                                return Some(cwd.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn proc_cwd(pid: String, home: &str) -> String {
    // Find child shell and check /proc/<pid>/cwd
    let shell_pid = Command::new("pgrep")
        .args(["-P", &pid])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.lines().last().map(|l| l.trim().to_string())
        });

    if let Some(spid) = shell_pid {
        if spid.is_empty() {
            return home.to_string();
        }
        let cwd_link = format!("/proc/{spid}/cwd");
        if let Ok(cwd) = std::fs::read_link(&cwd_link) {
            let cwd_str = cwd.to_string_lossy().to_string();
            // Check if the exe is a valid shell
            let exe_link = format!("/proc/{spid}/exe");
            if let Ok(exe) = std::fs::read_link(&exe_link) {
                let exe_str = exe.to_string_lossy().to_string();
                let shells = std::fs::read_to_string("/etc/shells").unwrap_or_default();
                if shells.lines().any(|l| l.trim() == exe_str) {
                    return cwd_str;
                }
            }
        }
    }

    home.to_string()
}

pub fn browser_handoff(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-cmd-browser-handoff", omarchy_path);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

pub fn default_browser() -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-cmd-default-browser", omarchy_path);
    let err = Command::new(&script).exec();
    eprintln!("exec {}: {}", script, err);
    1
}
