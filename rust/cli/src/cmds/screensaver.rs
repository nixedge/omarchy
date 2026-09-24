use std::process::{Command, Stdio};
use std::time::Duration;

pub fn run(force: bool) -> i32 {
    // Set background to black
    print!("\x1b]11;rgb:00/00/00\x07");

    // Hide cursor
    let _ = Command::new("hyprctl")
        .args(["eval", "hl.config({ cursor = { invisible = true } })"])
        .stdout(Stdio::null()).stderr(Stdio::null()).status()
        .or_else(|_| Command::new("hyprctl")
            .args(["keyword", "cursor:invisible", "true"])
            .stdout(Stdio::null()).stderr(Stdio::null()).status());

    let home = std::env::var("HOME").unwrap_or_default();
    let input_file = format!("{home}/.config/omarchy/branding/screensaver.txt");

    loop {
        let mut child = Command::new("ttfx")
            .args(["-i", &input_file,
                "--frame-rate", "120",
                "--canvas-width", "0",
                "--canvas-height", "0",
                "--reuse-canvas",
                "--anchor-canvas", "c",
                "--anchor-text", "c",
                "--random-effect",
                "--no-eol",
                "--no-restore-cursor"])
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to start ttfx: {e}");
                break;
            }
        };

        // Poll: wait for ttfx to exit or user input
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break, // ttfx exited
                Ok(None) => {
                    // Check focus
                    let in_focus = screensaver_in_focus();
                    if !in_focus {
                        exit_screensaver();
                        return 0;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(_) => break,
            }
        }
    }

    exit_screensaver();
    0
}

fn screensaver_in_focus() -> bool {
    Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok())
        .and_then(|v| v.get("class").and_then(|c| c.as_str()).map(|s| s == "org.omarchy.screensaver"))
        .unwrap_or(false)
}

fn exit_screensaver() {
    let _ = Command::new("hyprctl")
        .args(["eval", "hl.config({ cursor = { invisible = false } })"])
        .stdout(Stdio::null()).stderr(Stdio::null()).status()
        .or_else(|_| Command::new("hyprctl")
            .args(["keyword", "cursor:invisible", "false"])
            .stdout(Stdio::null()).stderr(Stdio::null()).status());
    let _ = Command::new("pkill").args(["-x", "ttfx"]).status();
    let _ = Command::new("pkill").args(["-f", "[o]rg.omarchy.screensaver"]).status();
}
