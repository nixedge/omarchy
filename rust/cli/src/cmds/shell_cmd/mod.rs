use std::process::{Command, Stdio};

/// Forward IPC calls to the running Omarchy shell via `qs ipc call`.
pub fn run(args: &[String]) -> i32 {
    let (quiet, rest) = if args.first().map(|s| s.as_str()) == Some("-q") {
        (true, &args[1..])
    } else {
        (false, &args[..])
    };

    let fail = |msg: &str| -> i32 {
        if quiet {
            return 0;
        }
        eprintln!("{}", msg);
        1
    };

    if rest.is_empty() || rest[0] == "-h" || rest[0] == "--help" {
        println!(
            "Usage: omarchy-shell [-q] <target> <method> [args...]\n\n\
             Forwards an IPC call to the running Omarchy shell."
        );
        return 0;
    }

    if rest.len() < 2 {
        return fail("Usage: omarchy-shell <target> <method> [args...]");
    }

    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    if omarchy_path.is_empty() {
        return fail("OMARCHY_PATH is not set");
    }

    let shell_qml = format!("{}/shell/shell.qml", omarchy_path);
    if !std::path::Path::new(&shell_qml).exists() {
        return fail(&format!("omarchy-shell config not found: {}", shell_qml));
    }

    // Recover WAYLAND_DISPLAY if missing (e.g. SSH sessions)
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| {
                // Fallback: /run/user/<uid> via id -u
                let uid = Command::new("id").arg("-u").output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                format!("/run/user/{}", uid)
            });
        if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
            let mut sockets: Vec<String> = entries
                .flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with("wayland-")
                        && !name.ends_with(".lock")
                        && name["wayland-".len()..].chars().all(|c| c.is_ascii_digit())
                    {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect();
            sockets.sort_by(|a, b| {
                let ma = std::fs::metadata(format!("{}/{}", runtime_dir, a))
                    .and_then(|m| m.modified())
                    .ok();
                let mb = std::fs::metadata(format!("{}/{}", runtime_dir, b))
                    .and_then(|m| m.modified())
                    .ok();
                mb.cmp(&ma)
            });
            if let Some(socket) = sockets.first() {
                std::env::set_var("WAYLAND_DISPLAY", socket);
            }
        }
    }

    // Handle shell summon/toggle with implicit "{}" arg
    let mut call_args: Vec<String> = rest.to_vec();
    if call_args.len() == 3
        && call_args[0] == "shell"
        && (call_args[1] == "summon" || call_args[1] == "toggle")
    {
        call_args.push("{}".to_string());
    }

    let ipc_timeout = std::env::var("OMARCHY_SHELL_IPC_TIMEOUT")
        .unwrap_or_else(|_| "2s".to_string());

    let mut cmd = Command::new("timeout");
    cmd.args(["--kill-after=1s", &ipc_timeout, "qs", "ipc", "-n", "-p", &omarchy_path, "call", "--"]);
    cmd.args(&call_args);
    cmd.stderr(Stdio::null());

    let result = cmd.output();

    let (output, status_code) = match result {
        Ok(o) => {
            let code = o.status.code().unwrap_or(1);
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            (stdout, code)
        }
        Err(_) => return fail("omarchy-shell is not running"),
    };

    // 124 = timeout, 137 = kill-after timeout
    if status_code == 124 || status_code == 137 {
        return fail("omarchy-shell is not responding");
    }
    if status_code != 0 {
        return fail("omarchy-shell is not running");
    }

    let output = output.trim_end_matches('\n');
    match output {
        "Target not found." | "Function not found." => return fail(output),
        s if s.starts_with("Too few arguments provided")
            || s.starts_with("Too many arguments provided") =>
        {
            return fail(s)
        }
        s if s.starts_with("Not ready to accept queries yet") => {
            return fail("omarchy-shell is not ready")
        }
        _ => {}
    }

    if !quiet && !output.is_empty() {
        println!("{}", output);
    }
    0
}
