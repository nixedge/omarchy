use super::rm_rf;
use crate::cmds::drop;
use crate::desktop;
use std::process::Command;

pub fn claude() -> i32 {
    let _ = Command::new("pkill").args(["-x", "claude-desktop"]).status();
    let rc = drop::run("claude-desktop");
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.config/Claude",
        "$HOME/.cache/Claude",
    ]);
    println!("\nClaude has been removed.");
    0
}

pub fn t3code() -> i32 {
    let rc = drop::run("t3code");
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.config/t3code",
        "$HOME/.t3",
        "$HOME/.config/t3code-flags.conf",
    ]);
    println!("\nT3 Code has been removed.");
    0
}

pub fn ollama() -> i32 {
    // Stop the service first
    let _ = Command::new("sudo")
        .args(["systemctl", "disable", "--now", "ollama.service"])
        .status();

    // Try to drop whichever variant is installed (best-effort)
    let _ = drop::run("ollama");

    let _ = Command::new("sudo").args(["rm", "-rf", "/var/lib/ollama"]).status();
    rm_rf(&["$HOME/.ollama"]);
    println!("\nOllama and its models have been removed.");
    0
}

pub fn chatgpt() -> i32 {
    let rc = drop::run("openai-codex-desktop");
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.config/Codex",
        "$HOME/.cache/Codex",
    ]);
    println!("\nChatGPT has been removed.");
    0
}

pub fn lm_studio() -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();

    // Read the home pointer before deletion
    let lmstudio_home = std::fs::read_to_string(format!("{home}/.lmstudio-home-pointer"))
        .ok()
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();

    let rc = drop::run("lmstudio");
    if rc != 0 { return rc; }

    rm_rf(&[
        "$HOME/.config/LM Studio",
        "$HOME/.config/LM-Studio",
        "$HOME/.lmstudio",
        "$HOME/.lmstudio-home-pointer",
    ]);

    // Remove the LM Studio home if it's a valid absolute path outside home root
    if !lmstudio_home.is_empty()
        && lmstudio_home.starts_with('/')
        && lmstudio_home != "/"
        && lmstudio_home != home
        && std::path::Path::new(&lmstudio_home).is_dir()
    {
        let _ = std::fs::remove_dir_all(&lmstudio_home);
    }

    println!("\nLM Studio and its models have been removed.");
    0
}

pub fn grok_bot() -> i32 {
    let rc = drop::run("grok-bot");
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.config/Grok Bot",
        "$HOME/.grokbot",
    ]);
    println!("\nGrok Bot has been removed.");
    0
}

pub fn perplexity() -> i32 {
    let rc = drop::run("perplexity");
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.cache/Perplexity",
        "$HOME/.cache/perplexity-rpc-server",
        "$HOME/.local/share/perplexity-rpc-server",
    ]);
    println!("\nPerplexity has been removed.");
    0
}

pub fn openclaw() -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let unit_dir = format!("{home}/.config/systemd/user");

    for role in &["gateway", "node"] {
        let unit = format!("openclaw-{role}.service");
        let unit_file = format!("{unit_dir}/{unit}");
        if !std::path::Path::new(&unit_file).exists() {
            continue;
        }

        let stopped = Command::new("openclaw")
            .args([role, "uninstall"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            || Command::new("systemctl")
                .args(["--user", "disable", "--now", &unit])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            || {
                let state = Command::new("systemctl")
                    .args(["--user", "is-active", &unit])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .unwrap_or_default();
                let state = state.trim();
                state == "inactive" || state == "failed"
            };

        if !stopped {
            eprintln!("Could not stop {unit}; OpenClaw was not removed.");
            return 1;
        }

        let _ = std::fs::remove_file(&unit_file);
        let _ = std::fs::remove_file(format!("{unit_file}.bak"));
        let _ = std::fs::remove_file(format!("{unit_dir}/default.target.wants/{unit}"));
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let _ = Command::new("systemctl")
            .args(["--user", "reset-failed", &unit])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    let rc = drop::run("openclaw");
    if rc != 0 { return rc; }

    let _ = std::fs::remove_file(format!("{home}/.local/share/applications/OpenClaw.desktop"));
    let _ = std::fs::remove_file(format!("{home}/.local/share/icons/hicolor/256x256/apps/openclaw.png"));
    let _ = Command::new("gtk-update-icon-cache")
        .arg(format!("{home}/.local/share/icons/hicolor"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    let openclaw_dir = format!("{home}/.openclaw");
    let state_removed = if std::path::Path::new(&openclaw_dir).is_dir() {
        let size = Command::new("du")
            .args(["-sh", &openclaw_dir])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or("?")
            .to_owned();
        let prompt = format!(
            "Also delete ~/.openclaw ({size}: chats, memories, credentials, and downloaded plugins)?"
        );
        if desktop::gum_confirm(&prompt) {
            let _ = std::fs::remove_dir_all(&openclaw_dir);
            true
        } else {
            false
        }
    } else {
        false
    };

    println!("\nOpenClaw has been removed.");
    if state_removed {
        println!("Its chats, memories, and settings in ~/.openclaw are gone too.");
    } else if std::path::Path::new(&openclaw_dir).is_dir() {
        println!("Your agent's chats, memories, and settings are still in ~/.openclaw.");
    }
    0
}

pub fn hermes() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script_path = format!("{omarchy_path}/bin/omarchy-remove-ai-hermes");

    // Delegate to the original bash script during transition
    if std::path::Path::new(&script_path).exists() {
        let err = std::os::unix::process::CommandExt::exec(
            Command::new("bash").arg(&script_path)
        );
        eprintln!("exec failed: {err}");
        return 1;
    }

    // Fallback: just drop the package
    let rc = drop::run("hermes-desktop");
    if rc != 0 { return rc; }
    println!("\nHermes Desktop has been removed.");
    0
}
