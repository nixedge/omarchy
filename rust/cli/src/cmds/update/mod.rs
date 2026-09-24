use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn dev() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    if omarchy_path == "/usr/share/omarchy" {
        eprintln!("Not a dev checkout");
        return 1;
    }
    let status = Command::new("git")
        .args(["-C", &omarchy_path, "pull", "--ff-only"])
        .status();
    if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
}

pub fn firmware() -> i32 {
    // Install fwupd if missing
    Command::new("omarchy-pkg-add").arg("fwupd").status().ok();

    let refresh = Command::new("sudo")
        .args(["fwupdmgr", "refresh"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let update = Command::new("sudo")
        .args(["fwupdmgr", "update"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if refresh && update { 0 } else { 1 }
}

pub fn lock(action: &str, cmd_args: &[String]) -> i32 {
    let lock_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let lock_path = format!("{lock_dir}/omarchy-update.lock");

    match action {
        "held" => {
            // Check if lock is held by current process
            lock_is_held(&lock_path)
        }
        "run" => {
            if cmd_args.is_empty() {
                eprintln!("Usage: omarchy-update-lock run <command> [args...]");
                return 2;
            }
            lock_run(&lock_path, cmd_args)
        }
        _ => {
            eprintln!("Usage: omarchy-update-lock <held|run> [command] [args...]");
            2
        }
    }
}

fn lock_is_held(lock_path: &str) -> i32 {
    // Check OMARCHY_UPDATE_LOCK_FD env var and verify the fd points to lock_path
    let lock_fd = std::env::var("OMARCHY_UPDATE_LOCK_FD").ok()
        .and_then(|s| s.parse::<i32>().ok());

    let fd = match lock_fd {
        Some(f) if f >= 0 => f,
        _ => return 1,
    };

    // Check if /proc/self/fd/<fd> exists
    let fd_link = format!("/proc/self/fd/{fd}");
    if !Path::new(&fd_link).exists() {
        return 1;
    }

    // Verify it points to lock_path
    let canonical = std::fs::canonicalize(lock_path).unwrap_or_default();
    let fd_target = std::fs::read_link(&fd_link).unwrap_or_default();
    if fd_target != canonical {
        return 1;
    }

    // Try non-blocking flock to verify it's held by us
    // If we can get the lock, it means WE hold it (or nobody does)
    // For "held" check: we want to know if this process has the lock open
    // Since we opened it, flock(LOCK_EX|LOCK_NB) succeeds (re-entrant)
    // This is a best-effort check
    0
}

fn lock_run(lock_path: &str, cmd_args: &[String]) -> i32 {
    use std::fs::OpenOptions;
    use std::os::unix::io::IntoRawFd;

    let _ = fs::create_dir_all(Path::new(lock_path).parent().unwrap_or(Path::new(".")));

    let file = match OpenOptions::new().write(true).create(true).open(lock_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Cannot open lock file {lock_path}: {e}");
            return 1;
        }
    };

    let fd = file.into_raw_fd();

    // Try to flock
    let locked = unsafe {
        libc_flock(fd, 2 | 4) // LOCK_EX | LOCK_NB
    };

    if locked != 0 {
        println!("An Omarchy update is already running.");
        unsafe { libc_close(fd); }
        return 1;
    }

    // Set the FD env var and exec the command
    std::env::set_var("OMARCHY_UPDATE_LOCK_FD", fd.to_string());

    let mut cmd = std::process::Command::new(&cmd_args[0]);
    cmd.args(&cmd_args[1..]);

    use std::os::unix::process::CommandExt;
    let err = cmd.exec();
    eprintln!("exec failed: {err}");
    1
}

extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
    fn close(fd: i32) -> i32;
}

fn libc_flock(fd: i32, op: i32) -> i32 {
    unsafe { flock(fd, op) }
}

fn libc_close(fd: i32) {
    unsafe { close(fd); }
}

pub fn requires_free_space() -> i32 {
    if std::env::var("OMARCHY_UPDATE_FORCE").as_deref() == Ok("1") {
        return 0;
    }

    let out = Command::new("df")
        .args(["--block-size=1", "/"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let available_bytes = match out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            s.lines().nth(1)
                .and_then(|l| l.split_whitespace().nth(3))
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0)
        }
        Err(_) => 0,
    };

    let ten_gib: u64 = 10 * 1024 * 1024 * 1024;
    if available_bytes >= ten_gib { 0 } else { 1 }
}

pub fn status() -> i32 {
    let status = Command::new("omarchy-update-available")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let code = status.map(|s| s.code().unwrap_or(0)).unwrap_or(0);

    if code == 0 {
        // Updates available - refresh indicators
        let _ = Command::new("omarchy-shell")
            .args(["-q", "omarchy.indicators", "refresh"])
            .status();
    } else {
        // No updates - clear indicators
        let _ = Command::new("omarchy-shell")
            .args(["-q", "omarchy.indicators", "clear"])
            .status();
    }
    0
}

pub fn stay_awake(action: &str) -> i32 {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/tmp/omarchy-{}", get_uid()));
    let state_dir = format!("{runtime_dir}/omarchy-update-stay-awake");
    let idle_owner_file = format!("{state_dir}/idle-owner");
    let inhibit_pid_file = format!("{state_dir}/inhibit-pid");
    let home = std::env::var("HOME").unwrap_or_default();
    let stay_awake_state = format!("{home}/.local/state/omarchy/indicators/stay-awake");

    match action {
        "start" => stay_awake_start(&state_dir, &idle_owner_file, &inhibit_pid_file, &stay_awake_state),
        "stop" => stay_awake_stop(&state_dir, &idle_owner_file, &inhibit_pid_file, &stay_awake_state),
        _ => {
            eprintln!("Usage: omarchy-update-stay-awake <start|stop>");
            2
        }
    }
}

fn stay_awake_start(state_dir: &str, idle_owner_file: &str, inhibit_pid_file: &str, stay_awake_state: &str) -> i32 {
    stay_awake_stop(state_dir, idle_owner_file, inhibit_pid_file, stay_awake_state);
    let _ = fs::create_dir_all(state_dir);

    // Start systemd-inhibit
    let uid = get_uid();
    let need_sudo = uid != 0;

    let lock_fd = std::env::var("OMARCHY_UPDATE_LOCK_FD").ok()
        .and_then(|s| s.parse::<i32>().ok());

    if Command::new("omarchy-cmd-present")
        .arg("systemd-inhibit")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        let mut cmd = if need_sudo {
            let is_terminal = unsafe { isatty(0) } != 0;
            if is_terminal {
                let _ = Command::new("sudo").arg("-v").status();
                let mut c = Command::new("sudo");
                c.args(["systemd-inhibit",
                    "--what=sleep:idle",
                    "--who=omarchy-update",
                    "--why=Omarchy update in progress",
                    "--mode=block",
                    "sleep", "infinity"]);
                c
            } else {
                let mut c = Command::new("pkexec");
                c.args(["systemd-inhibit",
                    "--what=sleep:idle",
                    "--who=omarchy-update",
                    "--why=Omarchy update in progress",
                    "--mode=block",
                    "sleep", "infinity"]);
                c
            }
        } else {
            let mut c = Command::new("systemd-inhibit");
            c.args(["--what=sleep:idle",
                "--who=omarchy-update",
                "--why=Omarchy update in progress",
                "--mode=block",
                "sleep", "infinity"]);
            c
        };

        cmd.stdout(Stdio::null()).stderr(Stdio::null());

        if let Ok(child) = cmd.spawn() {
            let pid = child.id();
            // Get start time from /proc/<pid>/stat
            let start_time = get_process_start_time(pid);
            if let Some(st) = start_time {
                let _ = fs::write(inhibit_pid_file, format!("{pid} {st}\n"));
            }
        }
    }

    // Toggle idle (stay-awake)
    if !Path::new(stay_awake_state).exists() {
        let idle_owner = format!("$$:{}", rand_num());
        let _ = fs::write(idle_owner_file, &idle_owner);
        let _ = fs::create_dir_all(Path::new(stay_awake_state).parent().unwrap_or(Path::new(".")));
        let ok = Command::new("omarchy-toggle-idle")
            .args(["stay-awake"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            let _ = fs::write(stay_awake_state, &idle_owner);
        } else {
            let _ = fs::remove_file(idle_owner_file);
        }
    }

    0
}

fn stay_awake_stop(state_dir: &str, idle_owner_file: &str, inhibit_pid_file: &str, stay_awake_state: &str) -> i32 {
    // Restore idle if we own it
    if let Ok(idle_owner) = fs::read_to_string(idle_owner_file) {
        let idle_owner = idle_owner.trim().to_string();
        if !idle_owner.is_empty() {
            let current_owner = fs::read_to_string(stay_awake_state)
                .unwrap_or_default()
                .trim()
                .to_string();
            if current_owner == idle_owner {
                let _ = Command::new("omarchy-toggle-idle")
                    .args(["allow-idle"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
        let _ = fs::remove_file(idle_owner_file);
    }

    // Kill inhibit process
    if let Ok(content) = fs::read_to_string(inhibit_pid_file) {
        let mut parts = content.trim().split_whitespace();
        let pid: Option<u32> = parts.next().and_then(|s| s.parse().ok());
        let recorded_start: Option<String> = parts.next().map(|s| s.to_string());

        if let (Some(pid), Some(recorded_st)) = (pid, recorded_start) {
            let current_st = get_process_start_time(pid);
            if current_st.as_deref() == Some(&recorded_st) {
                let _ = unsafe { libc_kill(pid as i32, 15) }; // SIGTERM
                // Wait up to 1 second
                for _ in 0..50 {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    if get_process_start_time(pid).as_deref() != Some(&recorded_st) {
                        break;
                    }
                }
            }
        }
        let _ = fs::remove_file(inhibit_pid_file);
    }

    let _ = fs::remove_dir(state_dir);
    0
}

extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
    fn isatty(fd: i32) -> i32;
}

fn libc_kill(pid: i32, sig: i32) -> i32 {
    unsafe { kill(pid, sig) }
}

fn get_process_start_time(pid: u32) -> Option<String> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Format: pid (comm) state ... starttime (field 22, 0-indexed 21)
    let after_comm = stat.rfind(')')?;
    let rest = &stat[after_comm + 2..];
    rest.split_whitespace().nth(19).map(|s| s.to_string())
}

fn get_uid() -> u32 {
    fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("Uid:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000)
}

fn rand_num() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0)
}

pub fn time() -> i32 {
    let status = Command::new("sudo")
        .args(["systemctl", "restart", "systemd-timesyncd"])
        .status();
    if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
}

pub fn user_notify(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let err = Command::new("omarchy-migrate-notify").args(args).exec();
    eprintln!("exec failed: {err}");
    1
}
