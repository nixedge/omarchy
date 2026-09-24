use std::process::{Command, Stdio};

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_default()
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn exec_delegate(script_name: &str, args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/{}", omarchy_path, script_name);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

fn cmd_present(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn focus_window_by_class(pattern: &str) -> Option<String> {
    let out = Command::new("hyprctl")
        .args(["clients", "-j"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let json = String::from_utf8(out.stdout).ok()?;
    let jq_filter = format!(
        ".[]|select((.class|test(\"\\\\b{}\\\\b\";\"i\")) or (.title|test(\"\\\\b{}\\\\b\";\"i\")))|.address",
        pattern.replace('"', "\\\""),
        pattern.replace('"', "\\\"")
    );
    let result = Command::new("jq")
        .args(["-r", &jq_filter])
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
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .unwrap_or_default();
    if result.is_empty() { None } else { Some(result) }
}

fn focus_address(address: &str) {
    let lua = format!("hl.dsp.focus({{ window = \"address:{}\" }})", address);
    let ok = Command::new("hyprctl")
        .args(["dispatch", &lua])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        let _ = Command::new("hyprctl")
            .args(["dispatch", "focuswindow", &format!("address:{}", address)])
            .stdout(Stdio::null())
            .status();
    }
}

// ─── 1password ───────────────────────────────────────────────────────────────

pub fn onepassword() -> i32 {
    use std::os::unix::process::CommandExt;
    if cmd_present("1password") {
        let err = Command::new("setsid")
            .args(["uwsm-app", "--", "1password", "--force-device-scale-factor=1"])
            .exec();
        eprintln!("exec setsid: {}", err);
        1
    } else {
        exec_delegate(
            "omarchy-launch-floating-terminal-with-presentation",
            &["omarchy-install-service-1password".to_string()],
        )
    }
}

// ─── about ───────────────────────────────────────────────────────────────────

pub fn about(args: &[String]) -> i32 {
    exec_delegate("omarchy-launch-about", args)
}

// ─── battlenet ───────────────────────────────────────────────────────────────

pub fn battlenet(args: &[String]) -> i32 {
    let prefix = format!("{}/Games/battlenet", home());
    let launcher = format!(
        "{}/drive_c/Program Files (x86)/Battle.net/Battle.net Launcher.exe",
        prefix
    );

    let mut with_mangohud = false;
    for arg in args {
        match arg.as_str() {
            "--with-mangohud" => with_mangohud = true,
            "-h" | "--help" => {
                println!("Usage: omarchy-launch-battlenet [--with-mangohud]");
                return 0;
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                return 1;
            }
        }
    }

    if !std::path::Path::new(&launcher).exists() {
        eprintln!("Battle.net is not installed. Run omarchy-install-gaming-battlenet first.");
        return 1;
    }

    use std::os::unix::process::CommandExt;
    let mut cmd = Command::new("env");
    cmd.env("WINEPREFIX", &prefix)
        .env("PROTONPATH", "GE-Proton")
        .env("GAMEID", "umu-battlenet")
        .env("PROTON_VERB", "run");
    if with_mangohud {
        cmd.env("MANGOHUD", "1");
    }
    let err = cmd.arg("umu-run").arg(&launcher).exec();
    eprintln!("exec umu-run: {}", err);
    1
}

// ─── browser ─────────────────────────────────────────────────────────────────

pub fn browser(args: &[String]) -> i32 {
    exec_delegate("omarchy-launch-browser", args)
}

// ─── config-editor ───────────────────────────────────────────────────────────

pub fn config_editor(args: &[String]) -> i32 {
    let path = match args.first() {
        Some(p) if !p.is_empty() => p.clone(),
        _ => {
            eprintln!("Usage: omarchy-launch-config-editor <path>");
            return 1;
        }
    };

    let _ = Command::new("omarchy-notification-send")
        .args(["-u", "low", "Editing config file", &path])
        .status();

    use std::os::unix::process::CommandExt;
    let err = Command::new("omarchy-launch-editor").arg(&path).exec();
    eprintln!("exec omarchy-launch-editor: {}", err);
    1
}

// ─── discord-community ───────────────────────────────────────────────────────

pub fn discord_community() -> i32 {
    let invite = "https://discord.gg/tXFUdasqhY";
    use std::os::unix::process::CommandExt;
    if cmd_present("discord") {
        let err = Command::new("setsid")
            .args([
                "uwsm-app",
                "--",
                "discord",
                "--url",
                "--",
                &format!("discord://-/invite/{}", &invite[invite.rfind('/').map(|i| i + 1).unwrap_or(0)..]),
            ])
            .exec();
        eprintln!("exec setsid discord: {}", err);
        1
    } else {
        exec_delegate("omarchy-launch-webapp", &[invite.to_string()])
    }
}

// ─── docker-tui ──────────────────────────────────────────────────────────────

pub fn docker_tui() -> i32 {
    use std::os::unix::process::CommandExt;
    let docker_ok = Command::new("omarchy-sudo-docker")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if docker_ok {
        let term = std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
        let err = Command::new("pkexec")
            .args(["/usr/bin/env", &format!("TERM={}", term), "lazydocker"])
            .exec();
        eprintln!("exec pkexec lazydocker: {}", err);
        1
    } else {
        let err = Command::new("lazydocker").exec();
        eprintln!("exec lazydocker: {}", err);
        1
    }
}

// ─── editor ──────────────────────────────────────────────────────────────────

pub fn editor(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;

    let inline = args.first().map(|s| s.as_str()) == Some("--inline");
    let file_args: Vec<&String> = if inline { args[1..].iter().collect() } else { args.iter().collect() };

    let default_editor_path = PathBuf::from(home()).join(".local/state/omarchy/defaults/editor");
    let mut editor = if default_editor_path.exists() {
        std::fs::read_to_string(&default_editor_path)
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        "nvim".to_string()
    };

    if !cmd_present(&editor) {
        editor = "nvim".to_string();
    }

    let editor_base = std::path::Path::new(&editor)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&editor)
        .to_string();

    match editor_base.as_str() {
        "nvim" | "vim" | "nano" | "micro" | "hx" | "helix" | "fresh" => {
            if inline {
                let err = Command::new(&editor).args(file_args).exec();
                eprintln!("exec {}: {}", editor, err);
                1
            } else {
                let mut launch_args = vec!["omarchy-launch-tui".to_string(), editor.clone()];
                launch_args.extend(file_args.iter().map(|s| s.to_string()));
                exec_delegate("omarchy-launch-tui", &launch_args[1..])
            }
        }
        _ => {
            let mut cmd = Command::new("setsid");
            cmd.args(["uwsm-app", "--", &editor]);
            cmd.args(file_args);
            let err = cmd.exec();
            eprintln!("exec setsid {} : {}", editor, err);
            1
        }
    }
}

use std::path::PathBuf;

// ─── floating-terminal-with-presentation ─────────────────────────────────────

pub fn floating_terminal_with_presentation(args: &[String]) -> i32 {
    exec_delegate("omarchy-launch-floating-terminal-with-presentation", args)
}

// ─── nautilus ────────────────────────────────────────────────────────────────

pub fn nautilus() -> i32 {
    use std::os::unix::process::CommandExt;
    let err = Command::new("setsid")
        .args(["uwsm-app", "--", "nautilus", "--new-window"])
        .exec();
    eprintln!("exec setsid nautilus: {}", err);
    1
}

// ─── nautilus-cwd ────────────────────────────────────────────────────────────

pub fn nautilus_cwd() -> i32 {
    use std::os::unix::process::CommandExt;
    let cwd = Command::new("omarchy-cmd-terminal-cwd")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let err = Command::new("setsid")
        .args(["uwsm-app", "--", "nautilus", "--new-window", &cwd])
        .exec();
    eprintln!("exec setsid nautilus --new-window: {}", err);
    1
}

// ─── openclaw ────────────────────────────────────────────────────────────────

pub fn openclaw(args: &[String]) -> i32 {
    exec_delegate("omarchy-launch-openclaw", args)
}

// ─── or-focus ────────────────────────────────────────────────────────────────

pub fn or_focus(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: omarchy-launch-or-focus [window-pattern] [launch-command]");
        return 1;
    }

    let pattern = &args[0];
    let launch_cmd = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        format!("uwsm-app -- {}", pattern)
    };

    if let Some(address) = focus_window_by_class(pattern) {
        focus_address(&address);
        0
    } else {
        use std::os::unix::process::CommandExt;
        // Parse launch_cmd into shell words and exec
        let parts: Vec<&str> = launch_cmd.split_whitespace().collect();
        if parts.is_empty() {
            return 1;
        }
        let err = Command::new("setsid").args(&parts).exec();
        eprintln!("exec setsid {}: {}", launch_cmd, err);
        1
    }
}

// ─── or-focus-tui ────────────────────────────────────────────────────────────

pub fn or_focus_tui(args: &[String]) -> i32 {
    let (app_id, remaining) = if args.first().map(|s| s.starts_with("--app-id=")).unwrap_or(false) {
        let id = args[0]["--app-id=".len()..].to_string();
        (id, &args[1..])
    } else {
        let base = args.first()
            .map(|s| {
                std::path::Path::new(s)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(s)
                    .to_string()
            })
            .unwrap_or_default();
        (format!("org.omarchy.{}", base), &args[..])
    };

    let mut launch_args = vec!["omarchy-launch-tui".to_string()];
    launch_args.extend(remaining.iter().cloned());

    let or_focus_args = vec![app_id, launch_args.join(" ")];
    or_focus(&or_focus_args)
}

// ─── or-focus-webapp ─────────────────────────────────────────────────────────

pub fn or_focus_webapp(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: omarchy-launch-or-focus-webapp [window-pattern] [url-and-flags...]");
        return 1;
    }

    let pattern = args[0].clone();
    let mut launch_args = vec!["omarchy-launch-webapp".to_string()];
    launch_args.extend(args[1..].iter().cloned());

    let or_focus_args = vec![pattern, launch_args.join(" ")];
    or_focus(&or_focus_args)
}

// ─── screensaver ─────────────────────────────────────────────────────────────

pub fn screensaver(args: &[String]) -> i32 {
    exec_delegate("omarchy-launch-screensaver", args)
}

// ─── shell ───────────────────────────────────────────────────────────────────

pub fn shell(args: &[String]) -> i32 {
    exec_delegate("omarchy-launch-shell", args)
}

// ─── signal ──────────────────────────────────────────────────────────────────

pub fn signal() -> i32 {
    if let Some(address) = focus_window_by_class("signal") {
        focus_address(&address);
        return 0;
    }

    use std::os::unix::process::CommandExt;
    if std::path::Path::new("/usr/bin/signal-desktop").exists() {
        let err = Command::new("setsid")
            .args(["uwsm-app", "--", "/usr/bin/signal-desktop"])
            .exec();
        eprintln!("exec setsid signal-desktop: {}", err);
        1
    } else {
        exec_delegate(
            "omarchy-launch-floating-terminal-with-presentation",
            &["omarchy-install-service-signal".to_string()],
        )
    }
}

// ─── spotify ─────────────────────────────────────────────────────────────────

pub fn spotify() -> i32 {
    if let Some(address) = focus_window_by_class("spotify") {
        focus_address(&address);
        return 0;
    }

    use std::os::unix::process::CommandExt;
    if std::path::Path::new("/usr/bin/spotify").exists() {
        let err = Command::new("setsid")
            .args(["uwsm-app", "--", "/usr/bin/spotify"])
            .exec();
        eprintln!("exec setsid spotify: {}", err);
        1
    } else {
        exec_delegate(
            "omarchy-launch-floating-terminal-with-presentation",
            &["omarchy-install-service-spotify".to_string()],
        )
    }
}

// ─── terminal ────────────────────────────────────────────────────────────────

pub fn terminal(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let cwd = Command::new("omarchy-cmd-terminal-cwd")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let mut cmd = Command::new("setsid");
    cmd.args(["uwsm-app", "--", "xdg-terminal-exec", &format!("--dir={}", cwd)]);
    cmd.args(args);
    let err = cmd.exec();
    eprintln!("exec setsid xdg-terminal-exec: {}", err);
    1
}

// ─── terminal-herdr ──────────────────────────────────────────────────────────

pub fn terminal_herdr() -> i32 {
    use std::os::unix::process::CommandExt;
    let err = Command::new("omarchy-launch-terminal").arg("herdr").exec();
    eprintln!("exec omarchy-launch-terminal herdr: {}", err);
    1
}

// ─── terminal-tmux ───────────────────────────────────────────────────────────

pub fn terminal_tmux() -> i32 {
    use std::os::unix::process::CommandExt;
    let err = Command::new("omarchy-launch-terminal")
        .args(["bash", "-c", "tmux attach || tmux new -s Work"])
        .exec();
    eprintln!("exec omarchy-launch-terminal tmux: {}", err);
    1
}

// ─── tui ─────────────────────────────────────────────────────────────────────

pub fn tui(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;

    let (app_id, remaining) = if args.first().map(|s| s.starts_with("--app-id=")).unwrap_or(false) {
        let id = args[0]["--app-id=".len()..].to_string();
        (id, &args[1..])
    } else {
        let base = args.first()
            .map(|s| {
                std::path::Path::new(s)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(s)
                    .to_string()
            })
            .unwrap_or_default();
        (format!("org.omarchy.{}", base), &args[..])
    };

    if remaining.is_empty() {
        eprintln!("Usage: omarchy-launch-tui [--app-id=<id>] <command> [args...]");
        return 1;
    }

    let mut cmd = Command::new("setsid");
    cmd.args([
        "uwsm-app",
        "--",
        "xdg-terminal-exec",
        &format!("--app-id={}", app_id),
        "-e",
        &remaining[0],
    ]);
    if remaining.len() > 1 {
        cmd.args(&remaining[1..]);
    }
    let err = cmd.exec();
    eprintln!("exec setsid xdg-terminal-exec: {}", err);
    1
}

// ─── webapp ──────────────────────────────────────────────────────────────────

pub fn webapp(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;

    let url = match args.first() {
        Some(u) if !u.is_empty() => u.clone(),
        _ => {
            eprintln!("Usage: omarchy-launch-webapp <url>");
            return 1;
        }
    };
    let extra_args = if args.len() > 1 { &args[1..] } else { &[] as &[String] };

    let browser = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let browser_desktop = match browser.as_str() {
        b if b.starts_with("google-chrome")
            || b.starts_with("brave")
            || b.starts_with("microsoft-edge")
            || b.starts_with("opera")
            || b.starts_with("vivaldi")
            || b.starts_with("helium") => browser.clone(),
        _ => "chromium.desktop".to_string(),
    };

    // Find browser executable
    let browser_exec = find_browser_exec(&browser_desktop);

    if browser_exec.is_empty() {
        eprintln!("Could not find browser executable for {}", browser_desktop);
        return 1;
    }

    let mut cmd = Command::new("setsid");
    cmd.args(["uwsm-app", "--", &browser_exec, &format!("--app={}", url)]);
    cmd.args(extra_args);
    let err = cmd.exec();
    eprintln!("exec setsid {} --app: {}", browser_exec, err);
    1
}

fn find_browser_exec(desktop_file: &str) -> String {
    let search_dirs = [
        format!("{}/.local/share/applications/{}", home(), desktop_file),
        format!("{}/.nix-profile/share/applications/{}", home(), desktop_file),
        format!("/usr/share/applications/{}", desktop_file),
    ];

    for path in &search_dirs {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if line.starts_with("Exec=") {
                    let exec = line.trim_start_matches("Exec=");
                    let first_word = exec.split_whitespace().next().unwrap_or("");
                    if !first_word.is_empty() {
                        return first_word.to_string();
                    }
                }
            }
        }
    }
    String::new()
}
