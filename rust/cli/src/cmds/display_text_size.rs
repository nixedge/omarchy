use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

const MIN: u32 = 9;
const MAX: u32 = 20;
const SHELL_DEFAULT_PX: u32 = 12;
const TERM_DEFAULT_PT: u32 = 9;
const GKEY_SCHEMA: &str = "org.gnome.desktop.interface";
const GKEY_NAME: &str = "text-scaling-factor";

pub fn run(arg: Option<&str>) -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let shell_config = format!("{home}/.config/omarchy/shell.toml");

    match arg {
        None | Some("") => {
            // Print current state
            let size = current_base_size(&shell_config)
                .map(|s| format!("{s}"))
                .unwrap_or_else(|| "12 (default)".to_string());
            let factor = Command::new("gsettings")
                .args(["get", GKEY_SCHEMA, GKEY_NAME])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            let term = term_current_pt();
            println!("text size: {size} px");
            println!("gtk text-scaling-factor: {factor}");
            println!("terminal font: {} pt", term.as_deref().unwrap_or("n/a"));
            0
        }
        Some("reset") | Some("default") => {
            reset_base_size(&shell_config);
            let _ = Command::new("gsettings")
                .args(["reset", GKEY_SCHEMA, GKEY_NAME])
                .status();
            set_terminal_size(TERM_DEFAULT_PT);
            0
        }
        Some(size_str) => {
            let size: u32 = match size_str.parse() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Size must be an integer between {MIN} and {MAX} (px).");
                    return 1;
                }
            };
            if size < MIN || size > MAX {
                eprintln!("Size must be an integer between {MIN} and {MAX} (px).");
                return 1;
            }

            set_base_size(&shell_config, size);

            // GTK scaling factor
            let gtk_pt = gtk_font_pt();
            let factor = format!("{:.4}", (gtk_pt as f64 * size as f64 / SHELL_DEFAULT_PX as f64 + 0.5).floor() / gtk_pt as f64);
            let _ = Command::new("gsettings")
                .args(["set", GKEY_SCHEMA, GKEY_NAME, &factor])
                .status();

            // Terminal
            let term_pt = (size * TERM_DEFAULT_PT + SHELL_DEFAULT_PX / 2) / SHELL_DEFAULT_PX;
            set_terminal_size(term_pt);

            0
        }
    }
}

fn current_base_size(config: &str) -> Option<String> {
    let content = fs::read_to_string(config).ok()?;
    let mut in_font = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            in_font = trimmed.starts_with("[font]");
            continue;
        }
        if in_font && trimmed.starts_with("base-size") {
            if let Some((_, val)) = trimmed.split_once('=') {
                let val = val.trim().split('#').next().unwrap_or("").trim();
                return Some(val.to_string());
            }
        }
    }
    None
}

fn set_base_size(config: &str, size: u32) {
    let _ = fs::create_dir_all(Path::new(config).parent().unwrap_or(Path::new(".")));

    if !Path::new(config).exists() {
        let _ = fs::write(config, format!("[font]\nbase-size = {size}\n"));
        return;
    }

    let content = fs::read_to_string(config).unwrap_or_default();
    let mut result = String::new();
    let mut in_font = false;
    let mut done = false;
    let mut inserted_into_section = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            if in_font && !done {
                result.push_str(&format!("base-size = {size}\n"));
                done = true;
            }
            in_font = trimmed.starts_with("[font]");
            result.push_str(line);
            result.push('\n');
            continue;
        }
        if in_font && trimmed.starts_with("base-size") {
            if !done {
                result.push_str(&format!("base-size = {size}\n"));
                done = true;
            }
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }

    if in_font && !done {
        result.push_str(&format!("base-size = {size}\n"));
        done = true;
    }

    if !done {
        if !result.ends_with('\n') { result.push('\n'); }
        result.push_str(&format!("[font]\nbase-size = {size}\n"));
    }

    let _ = fs::write(config, result);
}

fn reset_base_size(config: &str) {
    let Ok(content) = fs::read_to_string(config) else { return; };
    let mut result = String::new();
    let mut in_font = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            in_font = trimmed.starts_with("[font]");
        }
        if in_font && trimmed.starts_with("base-size") {
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }

    let _ = fs::write(config, result);
}

fn gtk_font_pt() -> u32 {
    let out = Command::new("gsettings")
        .args(["get", GKEY_SCHEMA, "font-name"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    // Parse last word as point size
    out.trim_matches('\'').split_whitespace().last()
        .and_then(|s| s.parse().ok())
        .unwrap_or(11)
}

fn set_terminal_size(pt: u32) {
    let home = std::env::var("HOME").unwrap_or_default();

    // Alacritty
    let alacritty = format!("{home}/.config/alacritty/alacritty.toml");
    if Path::new(&alacritty).exists() {
        let content = fs::read_to_string(&alacritty).unwrap_or_default();
        let new = content.lines()
            .map(|l| {
                if l.trim_start().starts_with("size") && l.contains('=') {
                    format!("size = {pt}")
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n") + "\n";
        let _ = fs::write(&alacritty, new);
    }

    // Kitty
    let kitty = format!("{home}/.config/kitty/kitty.conf");
    let has_kitty = Path::new(&kitty).exists() || command_present("kitty");
    if has_kitty {
        let _ = fs::create_dir_all(format!("{home}/.config/kitty"));
        if Path::new(&kitty).exists() {
            let content = fs::read_to_string(&kitty).unwrap_or_default();
            let has_size = content.lines().any(|l| l.trim_start().starts_with("font_size "));
            let new = if has_size {
                content.lines()
                    .map(|l| {
                        if l.trim_start().starts_with("font_size ") {
                            format!("font_size {pt}.0")
                        } else {
                            l.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n") + "\n"
            } else {
                format!("{content}\nfont_size {pt}.0\n")
            };
            let _ = fs::write(&kitty, new);
        } else {
            let _ = fs::write(&kitty, format!("font_size {pt}.0\n"));
        }
        let _ = Command::new("pkill").args(["-USR1", "kitty"]).status();
    }

    // Ghostty
    let ghostty = format!("{home}/.config/ghostty/config");
    if Path::new(&ghostty).exists() {
        let content = fs::read_to_string(&ghostty).unwrap_or_default();
        let new: String = content.lines()
            .map(|l| {
                if l.starts_with("font-size = ") {
                    format!("font-size = {pt}")
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n") + "\n";
        let _ = fs::write(&ghostty, new);
        let _ = Command::new("pkill").args(["-SIGUSR2", "ghostty"]).status();
    }

    // Foot
    let foot = format!("{home}/.config/foot/foot.ini");
    if Path::new(&foot).exists() {
        let content = fs::read_to_string(&foot).unwrap_or_default();
        let new: String = content.lines()
            .map(|l| {
                // Replace :size=N within font= lines
                if l.contains(":size=") {
                    let re = l.find(":size=").unwrap();
                    let after = &l[re + ":size=".len()..];
                    let end = after.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(after.len());
                    format!("{}:size={}{}", &l[..re], pt, &after[end..])
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n") + "\n";
        let _ = fs::write(&foot, new);
    }
}

fn term_current_pt() -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();

    let ghostty = format!("{home}/.config/ghostty/config");
    if Path::new(&ghostty).exists() {
        let content = fs::read_to_string(&ghostty).unwrap_or_default();
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("font-size = ") {
                return Some(val.trim().to_string());
            }
        }
    }

    let alacritty = format!("{home}/.config/alacritty/alacritty.toml");
    if Path::new(&alacritty).exists() {
        let content = fs::read_to_string(&alacritty).unwrap_or_default();
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("size") && trimmed.contains('=') {
                if let Some((_, val)) = trimmed.split_once('=') {
                    let v = val.trim();
                    if v.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        return Some(v.to_string());
                    }
                }
            }
        }
    }

    let kitty = format!("{home}/.config/kitty/kitty.conf");
    if Path::new(&kitty).exists() {
        let content = fs::read_to_string(&kitty).unwrap_or_default();
        for line in content.lines().rev() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("font_size ") {
                let val = trimmed.trim_start_matches("font_size ").trim().trim_end_matches(".0");
                return Some(val.to_string());
            }
        }
    } else if command_present("kitty") {
        return Some(TERM_DEFAULT_PT.to_string());
    }

    let foot = format!("{home}/.config/foot/foot.ini");
    if Path::new(&foot).exists() {
        let content = fs::read_to_string(&foot).unwrap_or_default();
        for line in content.lines() {
            if let Some(pos) = line.find(":size=") {
                let after = &line[pos + ":size=".len()..];
                let num: String = after.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                if !num.is_empty() {
                    return Some(num);
                }
            }
        }
    }

    None
}

fn command_present(cmd: &str) -> bool {
    Command::new("which").arg(cmd)
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false)
}
