use std::os::unix::process::CommandExt;
use std::process::Command;

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_default()
}

fn exec_script(script_name: &str, args: &[String]) -> i32 {
    let path = omarchy_path();
    let script = format!("{}/bin/omarchy-{}", path, script_name);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── omarchy-agent ────────────────────────────────────────────────────────────

pub fn run(inline: bool, pick: bool, prompt: Option<&str>) -> i32 {
    // Change to ~/Work if launched from $HOME and ~/Work exists
    let home_dir = home();
    if let Ok(cwd) = std::env::current_dir() {
        if cwd == std::path::Path::new(&home_dir) {
            let work = format!("{}/Work", home_dir);
            if std::path::Path::new(&work).is_dir() {
                let _ = std::env::set_current_dir(&work);
            }
        }
    }

    // Read the default agent
    let agent_file = format!("{}/.config/omarchy/defaults/agent", home_dir);
    let agent = if std::path::Path::new(&agent_file).exists() {
        std::fs::read_to_string(&agent_file)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    };

    let agent = match agent {
        Some(a) => a,
        None => {
            if pick {
                let err = Command::new("omarchy-menu")
                    .args(["summon", "setup.default.agent"])
                    .exec();
                eprintln!("exec omarchy-menu: {}", err);
                return 1;
            }
            eprintln!("Choose default agent with: omarchy default agent <name>");
            return 1;
        }
    };

    // Check if the agent command is installed
    let missing = Command::new("omarchy-cmd-missing")
        .arg(&agent)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if missing {
        eprintln!("{} is not installed. Choose an installed agent with: omarchy default agent <name>", agent);
        return 1;
    }

    // Build the agent command
    let mut command: Vec<String> = match agent.as_str() {
        "opencode" => {
            let mut c = vec!["opencode".to_string(), "--auto".to_string()];
            if let Some(p) = prompt {
                c.push("--prompt".to_string());
                c.push(p.to_string());
            }
            c
        }
        "agy" => {
            let mut c = vec!["agy".to_string(), "--dangerously-skip-permissions".to_string()];
            if let Some(p) = prompt {
                c.push("--prompt-interactive".to_string());
                c.push(p.to_string());
            }
            c
        }
        "copilot" => {
            let mut c = vec!["copilot".to_string(), "--allow-all".to_string()];
            if let Some(p) = prompt {
                c.push("--interactive".to_string());
                c.push(p.to_string());
            }
            c
        }
        "crush" => {
            if let Some(p) = prompt {
                vec!["crush".to_string(), "run".to_string(), p.to_string()]
            } else {
                vec!["crush".to_string(), "--yolo".to_string()]
            }
        }
        "claude" => {
            let mut c = vec!["claude".to_string(), "--permission-mode".to_string(), "auto".to_string()];
            if let Some(p) = prompt {
                c.push("--".to_string());
                c.push(p.to_string());
            }
            c
        }
        "grok" => {
            let mut c = vec!["grok".to_string(), "--permission-mode".to_string(), "bypassPermissions".to_string()];
            if let Some(p) = prompt {
                c.push("--".to_string());
                c.push(p.to_string());
            }
            c
        }
        "openclaw" => {
            let mut c = vec!["omarchy-launch-openclaw".to_string(), "--tui".to_string()];
            if let Some(p) = prompt {
                c.push("--message".to_string());
                c.push(p.to_string());
            }
            c
        }
        "codex" => {
            let mut c = vec!["codex".to_string(), "--approve-for-me".to_string()];
            if let Some(p) = prompt {
                c.push("--".to_string());
                c.push(p.to_string());
            }
            c
        }
        "cursor-agent" => {
            let mut c = vec!["cursor-agent".to_string(), "--yolo".to_string(), "--trust".to_string()];
            if let Some(p) = prompt {
                c.push("agent".to_string());
                c.push("--".to_string());
                c.push(p.to_string());
            }
            c
        }
        "hermes" => {
            if let Some(p) = prompt {
                vec![
                    "env".to_string(),
                    "-u".to_string(),
                    "HERMES_SESSION_SOURCE".to_string(),
                    "hermes".to_string(),
                    "chat".to_string(),
                    "--yolo".to_string(),
                    "--tui".to_string(),
                    format!("--query={}", p),
                ]
            } else {
                vec!["hermes".to_string(), "--yolo".to_string()]
            }
        }
        "muse" => {
            let mut c = vec!["muse".to_string(), "--approval-mode".to_string(), "never".to_string()];
            if let Some(p) = prompt {
                c.push("--".to_string());
                c.push(p.to_string());
            }
            c
        }
        "omp" => {
            let mut c = vec!["omp".to_string(), "--auto-approve".to_string()];
            if let Some(p) = prompt {
                c.push("--".to_string());
                c.push(p.to_string());
            }
            c
        }
        "ori" => {
            let mut c = vec!["ori".to_string(), "code".to_string()];
            if let Some(p) = prompt {
                c.push("--interactive".to_string());
                c.push("--prompt".to_string());
                c.push(p.to_string());
            }
            c
        }
        "pi" => {
            let mut c = vec!["pi".to_string()];
            if let Some(p) = prompt {
                c.push(p.to_string());
            }
            c
        }
        _ => {
            eprintln!("Unsupported default agent: {}", agent);
            return 1;
        }
    };

    if inline {
        let prog = command.remove(0);
        let err = Command::new(&prog).args(&command).exec();
        eprintln!("exec {}: {}", prog, err);
        1
    } else {
        let mut args = vec!["--app-id=org.omarchy.agent".to_string()];
        args.extend(command);
        let err = Command::new("omarchy-launch-tui").args(&args).exec();
        eprintln!("exec omarchy-launch-tui: {}", err);
        1
    }
}

// ─── omarchy-agent-crash ──────────────────────────────────────────────────────

pub fn crash(pid: &str, comm: Option<&str>, exe: Option<&str>, signal: Option<&str>) -> i32 {
    // Validate PID
    if !pid.chars().all(|c| c.is_ascii_digit()) {
        eprintln!("Not a PID: {}", pid);
        eprintln!("Usage: omarchy agent crash <pid>   (see: coredumpctl list)");
        return 1;
    }

    let comm = comm.unwrap_or("unknown");
    let exe = exe.unwrap_or("unknown");
    let signal = signal.unwrap_or("unknown");
    let omarchy_path = omarchy_path();
    let skill = format!("{}/default/agents/skills/diagnose-crash/SKILL.md", omarchy_path);

    // Try to get the crash timestamp
    let when = Command::new("coredumpctl")
        .args(["list", pid, "--no-pager", "--no-legend"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().last().map(|l| {
            l.split_whitespace().take(4).collect::<Vec<_>>().join(" ")
        }))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let prompt = format!(
        "A process crashed on this Omarchy machine and I want to know why.\n\
\n\
What systemd-coredump recorded:\n\
  process:  {}\n\
  PID:      {}\n\
  binary:   {}\n\
  signal:   {}\n\
  time:     {}\n\
\n\
Use the diagnose-crash skill: it covers how to investigate, what to report, and\n\
when a crash is worth reporting upstream to Omarchy. If your harness has no skill\n\
mechanism, read the skill files directly and follow them instead:\n\
\n\
  {}",
        comm, pid, exe, signal, when, skill
    );

    let err = Command::new("omarchy-agent")
        .args(["--prompt", &prompt])
        .exec();
    eprintln!("exec omarchy-agent: {}", err);
    1
}

// ─── omarchy-agent-prompt ─────────────────────────────────────────────────────

pub fn agent_prompt(inline: bool, prompt_words: &[String]) -> i32 {
    if prompt_words.is_empty() {
        eprintln!("Usage: omarchy agent prompt [--inline] <prompt...>");
        return 1;
    }
    let prompt = prompt_words.join(" ");
    let mut cmd = Command::new("omarchy-agent");
    if inline {
        cmd.arg("--inline");
    }
    cmd.args(["--prompt", &prompt]);
    let err = cmd.exec();
    eprintln!("exec omarchy-agent: {}", err);
    1
}

// ─── omarchy-agent-usage-* (exec delegates) ───────────────────────────────────

pub fn usage_claude(args: &[String]) -> i32 {
    exec_script("agent-usage-claude", args)
}

pub fn usage_codex(args: &[String]) -> i32 {
    exec_script("agent-usage-codex", args)
}

pub fn usage_fireworks(args: &[String]) -> i32 {
    exec_script("agent-usage-fireworks", args)
}

pub fn usage_grok(args: &[String]) -> i32 {
    exec_script("agent-usage-grok", args)
}

// ─── omarchy-agent-account-* (exec delegates) ────────────────────────────────

pub fn account_add(args: &[String]) -> i32 {
    exec_script("agent-account-add", args)
}

pub fn account_home(args: &[String]) -> i32 {
    exec_script("agent-account-home", args)
}

pub fn account_list(args: &[String]) -> i32 {
    exec_script("agent-account-list", args)
}

pub fn account_mode(args: &[String]) -> i32 {
    exec_script("agent-account-mode", args)
}

pub fn account_remove(args: &[String]) -> i32 {
    exec_script("agent-account-remove", args)
}

pub fn account_rename(args: &[String]) -> i32 {
    exec_script("agent-account-rename", args)
}

pub fn account_state(args: &[String]) -> i32 {
    exec_script("agent-account-state", args)
}

pub fn account_use(args: &[String]) -> i32 {
    exec_script("agent-account-use", args)
}

// ─── omarchy-agent-usage-update ───────────────────────────────────────────────

pub fn usage_update(force: bool, limits_only: bool, except: &[String], agents: &[String]) -> i32 {
    let home_dir = home();
    let xdg_state = std::env::var("XDG_STATE_HOME")
        .unwrap_or_else(|_| format!("{}/.local/state", home_dir));
    let usage_dir = format!("{}/omarchy/agents/usage", xdg_state);
    let _ = std::fs::create_dir_all(&usage_dir);

    let omarchy_path = omarchy_path();

    // Build flags list
    let mut flags: Vec<String> = Vec::new();
    if force {
        flags.push("--force".to_string());
    }
    if limits_only {
        flags.push("--limits-only".to_string());
    }

    // Find all omarchy-agent-usage-* collectors
    let bin_dir = format!("{}/bin", omarchy_path);
    let entries = match std::fs::read_dir(&bin_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Could not read {}: {}", bin_dir, e);
            return 1;
        }
    };

    let prefix = "omarchy-agent-usage-";
    let mut pids = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(prefix) {
            continue;
        }
        let agent_name = &name_str[prefix.len()..];
        if agent_name == "update" {
            continue;
        }
        // Check if executable
        let path = entry.path();
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = path.metadata() {
            if meta.permissions().mode() & 0o111 == 0 {
                continue;
            }
        } else {
            continue;
        }

        let agent_name = agent_name.to_string();

        // Check exclusions
        if except.iter().any(|e| e == &agent_name) {
            continue;
        }
        // Check only list
        if !agents.is_empty() && !agents.iter().any(|a| a == &agent_name) {
            continue;
        }

        let collector = path.to_string_lossy().to_string();
        let flags_clone = flags.clone();
        let usage_dir_clone = usage_dir.clone();
        let agent_name_clone = agent_name.clone();

        // Spawn a child process for each collector
        match Command::new(&collector).args(&flags_clone).output() {
            Ok(output) if output.status.success() => {
                let record = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if record.is_empty() {
                    eprintln!("omarchy-agent-usage-update: {} collector failed (empty output)", agent_name_clone);
                } else {
                    // Write to usage dir
                    let tmp = format!("{}/.{}.tmp", usage_dir_clone, agent_name_clone);
                    let dest = format!("{}/{}.json", usage_dir_clone, agent_name_clone);
                    if std::fs::write(&tmp, format!("{}\n", record)).is_ok() {
                        let _ = std::fs::rename(&tmp, &dest);
                    }
                }
                pids.push(0i32);
            }
            _ => {
                eprintln!("omarchy-agent-usage-update: {} collector failed", agent_name_clone);
                pids.push(1i32);
            }
        }
    }

    if pids.iter().any(|&c| c != 0) { 1 } else { 0 }
}
