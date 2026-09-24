use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

// ─── font current ──────────────────────────────────────────────────────────

pub fn current() -> i32 {
    // fc-match monospace -f '%{family}\n' | head -n1 | cut -d, -f1
    let out = Command::new("fc-match")
        .args(["monospace", "-f", "%{family}\\n"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    if let Some(first_line) = out.lines().next() {
        let family = first_line.split(',').next().unwrap_or(first_line);
        println!("{family}");
        0
    } else {
        1
    }
}

// ─── font list ─────────────────────────────────────────────────────────────

pub fn list() -> i32 {
    // fc-list :spacing=100 -f "%{family[0]}\n" | grep -v -i -E 'emoji|signwriting|omarchy' | sort -u
    let out = Command::new("fc-list")
        .args([":spacing=100", "-f", "%{family[0]}\\n"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut fonts: Vec<String> = out
        .lines()
        .filter(|l| {
            let lower = l.to_lowercase();
            !lower.contains("emoji") && !lower.contains("signwriting") && !lower.contains("omarchy")
        })
        .map(|l| l.to_string())
        .collect();

    fonts.sort();
    fonts.dedup();

    for font in fonts {
        println!("{font}");
    }
    0
}

// ─── font set ──────────────────────────────────────────────────────────────

pub fn set_font(font_name: &str) -> i32 {
    let home = home();

    // Check font exists
    let fc_out = Command::new("fc-list")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    if !fc_out.to_lowercase().contains(&font_name.to_lowercase()) {
        eprintln!("Font '{font_name}' not found.");
        return 1;
    }

    // Alacritty
    let alacritty = PathBuf::from(&home).join(".config/alacritty/alacritty.toml");
    if alacritty.exists() {
        sed_replace_in_file(
            &alacritty,
            r#"family = ".*""#,
            &format!(r#"family = "{font_name}""#),
        );
    }

    // Kitty
    let kitty_conf = PathBuf::from(&home).join(".config/kitty/kitty.conf");
    if kitty_conf.exists() || run_ok("kitty", &["--version"]) {
        let _ = fs::create_dir_all(kitty_conf.parent().unwrap());
        if kitty_conf.exists() {
            let content = fs::read_to_string(&kitty_conf).unwrap_or_default();
            let has_font_family = content.lines().any(|l| {
                let trimmed = l.trim_start();
                trimmed.starts_with("font_family") && trimmed[11..].trim_start().len() > 0
            });
            if has_font_family {
                sed_replace_in_file_pattern(
                    &kitty_conf,
                    r"^\s*font_family\s+.*",
                    &format!("font_family {font_name}"),
                );
            } else {
                let mut f = fs::OpenOptions::new().append(true).open(&kitty_conf).unwrap();
                use std::io::Write;
                let _ = writeln!(f, "\nfont_family {font_name}");
            }
            let _ = Command::new("pkill").args(["-USR1", "kitty"]).status();
        }
    }

    // Ghostty
    let ghostty_config = PathBuf::from(&home).join(".config/ghostty/config");
    if ghostty_config.exists() {
        sed_replace_in_file(
            &ghostty_config,
            r#"font-family = ".*""#,
            &format!(r#"font-family = "{font_name}""#),
        );
        let _ = Command::new("pkill").args(["-SIGUSR2", "ghostty"]).status();
    }

    // Foot
    let foot_ini = PathBuf::from(&home).join(".config/foot/foot.ini");
    if foot_ini.exists() {
        sed_replace_in_file_pattern(
            &foot_ini,
            r"^font=.*",
            &format!("font={font_name}:size=9"),
        );
    }

    // Fontconfig (canonical source of truth)
    let fontconfig_file = PathBuf::from(&home).join(".config/fontconfig/fonts.conf");
    let _ = fs::create_dir_all(fontconfig_file.parent().unwrap());
    let xml = format!(
        r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
  <match target="pattern">
    <test name="family" qual="any">
      <string>monospace</string>
    </test>
    <edit name="family" mode="prepend_first" binding="strong">
      <string>{font_name}</string>
    </edit>
  </match>
</fontconfig>
"#
    );
    if let Err(e) = fs::write(&fontconfig_file, xml) {
        eprintln!("Cannot write fontconfig: {e}");
        return 1;
    }

    let _ = Command::new("omarchy-restart-shell").status();

    if is_running("ghostty") {
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", "\u{f0635}", "You must restart Ghostty to see font change"])
            .status();
    }

    if is_running("foot") {
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", "\u{f0635}", "You must restart Foot to see font change"])
            .status();
    }

    let _ = Command::new("omarchy-hook")
        .args(["font-set", font_name])
        .status();

    0
}

fn sed_replace_in_file(path: &PathBuf, pattern: &str, replacement: &str) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let new_content = replace_pattern(&content, pattern, replacement, false);
    let _ = fs::write(path, new_content);
}

fn sed_replace_in_file_pattern(path: &PathBuf, pattern: &str, replacement: &str) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let new_content = replace_pattern(&content, pattern, replacement, true);
    let _ = fs::write(path, new_content);
}

fn replace_pattern(content: &str, pattern: &str, replacement: &str, is_line_pattern: bool) -> String {
    // Use sed subprocess for regex replacement
    let sed_pattern = if is_line_pattern {
        format!("s|{pattern}|{replacement}|g")
    } else {
        format!("s|{pattern}|{replacement}|g")
    };

    let out = Command::new("sed")
        .args([&sed_pattern])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    if let Ok(mut child) = out {
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(content.as_bytes());
        }
        if let Ok(output) = child.wait_with_output() {
            if output.status.success() {
                if let Ok(s) = String::from_utf8(output.stdout) {
                    return s;
                }
            }
        }
    }
    content.to_string()
}

fn is_running(process: &str) -> bool {
    Command::new("pgrep")
        .args(["-x", process])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
