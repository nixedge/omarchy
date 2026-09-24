use std::os::unix::process::CommandExt;
use std::process::Command;

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

// ─── default agent ────────────────────────────────────────────────────────────

pub fn agent(install_flag: bool, name: Option<&str>) -> i32 {
    let home_dir = home();
    let agent_file = format!("{}/.config/omarchy/defaults/agent", home_dir);

    match name {
        None => {
            // Read and print current agent
            if std::path::Path::new(&agent_file).exists() {
                if let Ok(content) = std::fs::read_to_string(&agent_file) {
                    let agent = content.trim();
                    if !agent.is_empty() {
                        println!("{}", agent);
                    }
                }
            }
            0
        }
        Some(agent_name) => {
            // Validate the agent name
            let (agent, _name_display, agent_package, agent_installer) = match agent_name {
                "pi" => ("pi", "Pi", "pi", None),
                "omp" | "oh-my-pi" => ("omp", "Oh My Pi", "github:can1357/oh-my-pi", None),
                "opencode" | "open-code" => ("opencode", "OpenCode", "opencode", None),
                "ori" | "openrouter" => ("ori", "Ori", "github:OpenRouterLabs/ori-releases", None),
                "claude" | "claude-code" => ("claude", "Claude Code", "claude", None),
                "codex" => ("codex", "Codex", "codex", None),
                "crush" => ("crush", "Crush", "crush", None),
                "grok" => ("grok", "Grok", "npm:@xai-official/grok", None),
                "openclaw" => ("openclaw", "OpenClaw", "openclaw", Some("omarchy-install-openclaw-cli")),
                "agy" | "antigravity" | "antigravity-cli" | "gemini" | "gemini-cli" => {
                    ("agy", "Antigravity", "antigravity-cli", None)
                }
                "hermes" => ("hermes", "Hermes", "hermes", Some("omarchy-install-hermes-cli")),
                "copilot" | "github-copilot" => ("copilot", "GitHub Copilot", "copilot", None),
                "muse" | "muse-code" | "musecode" => (
                    "muse",
                    "Muse Code",
                    "http:muse[url=https://api.meta.ai/muse-launcher.sh,bin=muse,version_list_url=https://api.meta.ai/muse-code/channels/muse-stable,version_json_path=.version]",
                    None,
                ),
                "cursor" | "cursor-agent" => ("cursor-agent", "Cursor CLI", "cursor-agent", None),
                _ => {
                    eprintln!("Usage: omarchy-default-agent <pi|omp|opencode|ori|claude|codex|grok|openclaw|agy|hermes|copilot|crush|cursor-agent|muse>");
                    return 1;
                }
            };

            // Check if agent is present and potentially install
            let agent_present = if let Some(installer) = agent_installer {
                Command::new(installer)
                    .arg("--check")
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            } else {
                // Check user install or mise
                let user_bin = format!("{}/.local/bin/{}", home_dir, agent);
                let user_installed = if std::path::Path::new(&user_bin).exists() {
                    // Check if it's a symlink or not a mise wrapper
                    std::path::Path::new(&user_bin).is_symlink() || {
                        std::fs::read_to_string(&user_bin)
                            .map(|c| !c.contains("mise use -g"))
                            .unwrap_or(false)
                    }
                } else {
                    false
                };
                user_installed || {
                    Command::new("mise")
                        .args(["where", agent_package])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                }
            };

            if !install_flag && !agent_present {
                let err = Command::new("omarchy-launch-floating-terminal-with-presentation")
                    .args(["omarchy-default-agent", "--install", agent])
                    .exec();
                eprintln!("exec omarchy-launch-floating-terminal-with-presentation: {}", err);
                return 1;
            }

            // Install the agent
            let install_ok = if let Some(installer) = agent_installer {
                Command::new(installer)
                    .arg("--now")
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            } else {
                // user_install or mise use -g
                let user_bin = format!("{}/.local/bin/{}", home_dir, agent);
                let user_installed = std::path::Path::new(&user_bin).exists();
                if user_installed {
                    true
                } else {
                    Command::new("mise")
                        .args(["use", "-g", agent_package])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                }
            };

            if !install_ok {
                if install_flag {
                    eprintln!("Could not install {} with mise", _name_display);
                } else {
                    eprintln!("Could not set {} as the default coding agent", _name_display);
                }
                return 1;
            }

            // Install Claude chromium extension if applicable
            if agent == "claude" {
                let _ = Command::new("omarchy-install-chromium-claude")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
            }

            // Write agent file
            if let Some(parent) = std::path::Path::new(&agent_file).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&agent_file, format!("{}\n", agent)) {
                eprintln!("Failed to write agent file: {}", e);
                return 1;
            }

            if install_flag {
                // Clear screen and exec agent inline
                print!("\x1b[2J\x1b[3J\x1b[H");
                let err = Command::new("omarchy-agent").arg("--inline").exec();
                eprintln!("exec omarchy-agent: {}", err);
                1
            } else {
                let err = Command::new("omarchy-agent").exec();
                eprintln!("exec omarchy-agent: {}", err);
                1
            }
        }
    }
}

// ─── default browser ─────────────────────────────────────────────────────────

pub fn browser(install_flag: bool, name: Option<&str>) -> i32 {
    match name {
        None => {
            // Read current default browser
            let out = Command::new("env")
                .args(["-u", "BROWSER", "xdg-settings", "get", "default-web-browser"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let mapped = match out.as_str() {
                "chromium.desktop" => "chromium",
                "google-chrome.desktop" => "chrome",
                "brave-browser.desktop" => "brave",
                "brave-origin.desktop" => "brave-origin",
                "microsoft-edge.desktop" => "edge",
                "firefox.desktop" => "firefox",
                "zen.desktop" => "zen",
                _ => &out,
            };
            println!("{}", mapped);
            0
        }
        Some(browser_name) => {
            let (command, desktop_id, display_name, glyph) = match browser_name {
                "chromium" => ("chromium", "chromium.desktop", "Chromium", ""),
                "chrome" => ("google-chrome-stable", "google-chrome.desktop", "Chrome", "\u{ea87}"),
                "brave" => ("brave", "brave-browser.desktop", "Brave", "\u{f0598}"),
                "brave-origin" => ("brave-origin", "brave-origin.desktop", "Brave Origin", "\u{f0598}"),
                "edge" => ("microsoft-edge-stable", "microsoft-edge.desktop", "Edge", "\u{f01e9}"),
                "firefox" => ("firefox", "firefox.desktop", "Firefox", "\u{f0239}"),
                "zen" => ("zen-browser", "zen.desktop", "Zen", "\u{f0598}"),
                _ => {
                    eprintln!("Usage: omarchy-default-browser <chromium|chrome|brave|brave-origin|edge|firefox|zen>");
                    return 1;
                }
            };

            // Check if command is missing
            let missing = Command::new("omarchy-cmd-missing")
                .arg(command)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if missing {
                if !install_flag {
                    let err = Command::new("omarchy-launch-floating-terminal-with-presentation")
                        .args(["omarchy-default-browser", "--install", browser_name])
                        .exec();
                    eprintln!("exec omarchy-launch-floating-terminal-with-presentation: {}", err);
                    return 1;
                } else {
                    let status = Command::new("omarchy-install-browser")
                        .arg(browser_name)
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if !status {
                        return 1;
                    }
                }
            }

            let status = Command::new("env")
                .args(["-u", "BROWSER", "xdg-settings", "set", "default-web-browser", desktop_id])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if !status {
                return 1;
            }

            let _ = Command::new("omarchy-notification-send")
                .args(["-g", glyph, &format!("{} is now the default browser", display_name)])
                .status();

            0
        }
    }
}

// ─── default editor ───────────────────────────────────────────────────────────

pub fn editor(install_flag: bool, name: Option<&str>) -> i32 {
    let home_dir = home();
    let editor_file = format!("{}/.local/state/omarchy/defaults/editor", home_dir);

    match name {
        None => {
            if std::path::Path::new(&editor_file).exists() {
                if let Ok(content) = std::fs::read_to_string(&editor_file) {
                    let e = content.trim();
                    if !e.is_empty() {
                        println!("{}", e);
                        return 0;
                    }
                }
            }
            println!("nvim");
            0
        }
        Some(editor_name) => {
            let (selection, editor_cmd, display_name, glyph) = match editor_name {
                "code" => ("code", "code", "VSCode", "\u{e70c}"),
                "cursor" => ("cursor", "cursor", "Cursor", "\u{e70c}"),
                "zed" | "zeditor" => ("zed", "zeditor", "Zed", "\u{e70c}"),
                "sublime_text" => ("sublime_text", "sublime_text", "Sublime Text", "\u{e70c}"),
                "helix" => ("helix", "helix", "Helix", "\u{e70c}"),
                "vim" => ("vim", "vim", "Vim", "\u{e70c}"),
                "emacs" => ("emacs", "emacs", "Emacs", "\u{e70c}"),
                "nvim" => ("nvim", "nvim", "Neovim", "\u{e70c}"),
                _ => {
                    eprintln!("Usage: omarchy-default-editor <code|cursor|zed|sublime_text|helix|vim|emacs|nvim>");
                    return 1;
                }
            };

            // Check if command is missing
            let missing = Command::new("omarchy-cmd-missing")
                .arg(editor_cmd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if missing {
                if !install_flag {
                    let err = Command::new("omarchy-launch-floating-terminal-with-presentation")
                        .args(["omarchy-default-editor", "--install", selection])
                        .exec();
                    eprintln!("exec omarchy-launch-floating-terminal-with-presentation: {}", err);
                    return 1;
                } else {
                    let install_ok = match selection {
                        "code" => run_cmd("omarchy-install-editor-vscode", &[]),
                        "cursor" => run_cmd("omarchy-pkg-add", &["cursor-bin"]),
                        "zed" => run_cmd("omarchy-install-editor-zed", &[]),
                        "sublime_text" => run_cmd("omarchy-pkg-add", &["sublime-text-4"]),
                        "helix" => run_cmd("omarchy-install-editor-helix", &[]),
                        "vim" => run_cmd("omarchy-pkg-add", &["vim"]),
                        "emacs" => run_cmd("omarchy-install-editor-emacs", &[]),
                        "nvim" => run_cmd("omarchy-pkg-add", &["neovim"]),
                        _ => false,
                    };
                    if !install_ok {
                        return 1;
                    }
                }
            }

            if let Some(parent) = std::path::Path::new(&editor_file).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&editor_file, format!("{}\n", selection)) {
                eprintln!("Failed to write editor file: {}", e);
                return 1;
            }

            let _ = Command::new("omarchy-notification-send")
                .args(["-g", glyph, &format!("{} is now the default editor", display_name)])
                .status();

            0
        }
    }
}

// ─── default terminal ─────────────────────────────────────────────────────────

pub fn terminal(install_flag: bool, name: Option<&str>) -> i32 {
    match name {
        None => {
            let out = Command::new("xdg-terminal-exec")
                .arg("--print-id")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| {
                    let id = s.trim();
                    // strip after colon
                    let id = id.split(':').next().unwrap_or(id);
                    match id {
                        "Alacritty.desktop" => "alacritty".to_string(),
                        "foot.desktop" => "foot".to_string(),
                        "com.mitchellh.ghostty.desktop" => "ghostty".to_string(),
                        "kitty.desktop" => "kitty".to_string(),
                        _ => id.to_string(),
                    }
                })
                .unwrap_or_default();
            println!("{}", out);
            0
        }
        Some(terminal_name) => {
            let (cmd, desktop_id, display_name, glyph) = match terminal_name {
                "alacritty" => ("alacritty", "Alacritty.desktop", "Alacritty", "\u{e70c}"),
                "foot" => ("foot", "foot.desktop", "Foot", "\u{e70c}"),
                "ghostty" => ("ghostty", "com.mitchellh.ghostty.desktop", "Ghostty", "\u{e70c}"),
                "kitty" => ("kitty", "kitty.desktop", "Kitty", "\u{e70c}"),
                _ => {
                    eprintln!("Usage: omarchy-default-terminal <alacritty|foot|ghostty|kitty>");
                    return 1;
                }
            };

            // Check if missing
            let missing = Command::new("omarchy-cmd-missing")
                .arg(cmd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if missing {
                if !install_flag {
                    let err = Command::new("omarchy-launch-floating-terminal-with-presentation")
                        .args(["omarchy-default-terminal", "--install", terminal_name])
                        .exec();
                    eprintln!("exec omarchy-launch-floating-terminal-with-presentation: {}", err);
                    return 1;
                } else {
                    let ok = run_cmd("omarchy-install-terminal", &[terminal_name]);
                    if !ok {
                        return 1;
                    }
                }
            }

            // Write xdg-terminals.list
            let home_dir = home();
            let list_file = format!("{}/.config/xdg-terminals.list", home_dir);
            let content = format!(
                "# Terminal emulator preference order for xdg-terminal-exec\n\
# The first found and valid terminal will be used\n\
{}\n",
                desktop_id
            );
            if let Err(e) = std::fs::write(&list_file, &content) {
                eprintln!("Failed to write {}: {}", list_file, e);
                return 1;
            }

            let _ = Command::new("omarchy-notification-send")
                .args(["-g", glyph, &format!("{} is now the default terminal", display_name)])
                .status();

            0
        }
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
