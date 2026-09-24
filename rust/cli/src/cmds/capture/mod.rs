use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

// ─── capture qr ───────────────────────────────────────────────────────────────

pub fn qr() -> i32 {
    // Start hyprpicker freeze
    let mut freeze = match Command::new("hyprpicker")
        .args(["-r", "-z"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Failed to start hyprpicker");
            return 1;
        }
    };

    thread::sleep(Duration::from_millis(100));

    // Run slurp
    let selection = Command::new("slurp")
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Kill the freeze
    let _ = freeze.kill();

    if selection.is_empty() {
        return 0;
    }

    // Capture and decode QR
    let grim = Command::new("grim")
        .args(["-g", &selection, "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let grim = match grim {
        Ok(c) => c,
        Err(_) => return 1,
    };

    let result = Command::new("zbarimg")
        .args(["-q", "--raw", "-Sdisable", "-Sqrcode.enable", "-"])
        .stdin(grim.stdout.unwrap())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if result.is_empty() {
        let _ = Command::new("omarchy-notification-send")
            .args(["-g", "\u{f04b2}", "-u", "critical", "No QR code found", "Select a region containing a QR code"])
            .status();
        return 1;
    }

    // Copy to clipboard (sensitive)
    use std::io::Write;
    let mut wl = match Command::new("wl-copy")
        .arg("--sensitive")
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return 1,
    };
    if let Some(ref mut stdin) = wl.stdin {
        let _ = stdin.write_all(result.as_bytes());
    }
    let _ = wl.wait();

    let _ = Command::new("omarchy-notification-send")
        .args(["-g", "\u{f04b2}", "QR code copied to clipboard"])
        .status();
    0
}

// ─── capture region ───────────────────────────────────────────────────────────

pub fn region(mode: Option<&str>, keep_freeze: bool, match_monitor: bool) -> i32 {
    let mut args = vec!["capture".to_string(), "region".to_string()];
    if let Some(m) = mode {
        args.push(m.to_string());
    }
    if keep_freeze {
        args.push("--keep-freeze".to_string());
    }
    if match_monitor {
        args.push("--match-monitor".to_string());
    }
    // Exec the bash script since the logic is very complex (warp probes, etc.)
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-capture-region", omarchy_path);
    let mut cmd = Command::new(&script);
    if let Some(m) = mode {
        cmd.arg(m);
    }
    if keep_freeze {
        cmd.arg("--keep-freeze");
    }
    if match_monitor {
        cmd.arg("--match-monitor");
    }
    let err = cmd.exec();
    eprintln!("exec {}: {}", script, err);
    1
}

pub fn region_take_fullscreen() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-capture-region", omarchy_path);
    let err = Command::new(&script).arg("--take-fullscreen").exec();
    eprintln!("exec {}: {}", script, err);
    1
}

pub fn region_take_window() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-capture-region", omarchy_path);
    let err = Command::new(&script).arg("--take-window").exec();
    eprintln!("exec {}: {}", script, err);
    1
}

pub fn region_select_window(direction: &str) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-capture-region", omarchy_path);
    let err = Command::new(&script)
        .args(["--select-window", direction])
        .exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── capture screenrecording ─────────────────────────────────────────────────

pub fn screenrecording(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-capture-screenrecording", omarchy_path);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── capture screenrecording-with-webcam ─────────────────────────────────────

pub fn screenrecording_with_webcam() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-capture-screenrecording-with-webcam", omarchy_path);
    let err = Command::new(&script).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── capture screenshot ──────────────────────────────────────────────────────

pub fn screenshot(args: &[String]) -> i32 {
    // Forward OMARCHY_SCREENSHOT_DIR → OMASNAP_SCREENSHOT_DIR if set
    if let Ok(dir) = std::env::var("OMARCHY_SCREENSHOT_DIR") {
        if std::env::var("OMASNAP_SCREENSHOT_DIR").is_err() {
            std::env::set_var("OMASNAP_SCREENSHOT_DIR", &dir);
        }
    }

    // Map args: copy→--copy, save→--save, slurp→skip, others→pass through
    let mut omasnap_args: Vec<String> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "copy" => omasnap_args.push("--copy".to_string()),
            "save" => omasnap_args.push("--save".to_string()),
            "slurp" => {} // skip
            _ => omasnap_args.push(arg.clone()),
        }
    }

    let err = Command::new("omasnap").args(&omasnap_args).exec();
    eprintln!("exec omasnap: {}", err);
    1
}

// ─── capture text ────────────────────────────────────────────────────────────

pub fn text() -> i32 {
    // Start hyprpicker freeze
    let mut freeze = match Command::new("hyprpicker")
        .args(["-r", "-z"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return 1,
    };

    thread::sleep(Duration::from_millis(100));

    let selection = Command::new("slurp")
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let _ = freeze.kill();

    if selection.is_empty() {
        return 0;
    }

    let ocr_langs = std::env::var("OMARCHY_OCR_LANGS").unwrap_or_else(|_| "eng".to_string());

    // grim -g "$SELECTION" - | tesseract stdin stdout ...
    let grim = Command::new("grim")
        .args(["-g", &selection, "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let grim = match grim {
        Ok(c) => c,
        Err(_) => return 1,
    };

    let result = Command::new("tesseract")
        .args([
            "stdin", "stdout",
            "--oem", "1",
            "--psm", "6",
            "-l", &ocr_langs,
            "--dpi", "300",
            "-c", "preserve_interword_spaces=1",
        ])
        .stdin(grim.stdout.unwrap())
        .stderr(Stdio::null())
        .output();

    let text = match result {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return 1,
    };

    if text.is_empty() {
        return 1;
    }

    // wl-copy
    use std::io::Write;
    let mut wl = match Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return 1,
    };
    if let Some(ref mut stdin) = wl.stdin {
        let _ = write!(stdin, "{}", text);
    }
    let _ = wl.wait();

    let _ = Command::new("omarchy-notification-send")
        .args(["-g", "\u{f0d11}", "Copied text from selection to clipboard"])
        .status();
    0
}

// ─── capture webcam-list ─────────────────────────────────────────────────────

pub fn webcam_list() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-capture-webcam-list", omarchy_path);
    let err = Command::new(&script).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── capture webcam-resize ───────────────────────────────────────────────────

pub fn webcam_resize(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-capture-webcam-resize", omarchy_path);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}
