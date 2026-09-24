use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn receive(once: bool, dir: Option<&str>) -> i32 {
    let download_dir = dir
        .map(|s| s.to_string())
        .or_else(|| std::env::var("XDG_DOWNLOAD_DIR").ok())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{home}/Downloads")
        });

    let staging = format!("{download_dir}/.omarchy-taildrop");
    let _ = fs::create_dir_all(&staging);

    // Deliver any leftover files from interrupted runs
    deliver(&staging, &download_dir);

    loop {
        let status = Command::new("tailscale")
            .args(["file", "get", "--wait", "--conflict=rename", &staging])
            .status();

        match status {
            Ok(s) if s.success() => {
                deliver(&staging, &download_dir);
                if once { return 0; }
            }
            _ => {
                if once { return 1; }
                std::thread::sleep(Duration::from_secs(10));
            }
        }
    }
}

fn deliver(staging: &str, dir: &str) {
    let Ok(entries) = fs::read_dir(staging) else { return; };
    for entry in entries.flatten() {
        let staged = entry.path();
        if let Some(target) = claim_path(&staged, dir) {
            announce(&target, dir);
        }
    }
}

fn claim_path(staged: &Path, dir: &str) -> Option<String> {
    let name = staged.file_name()?.to_string_lossy().to_string();
    let (base, ext) = if let Some(dot_pos) = name.rfind('.') {
        let b = &name[..dot_pos];
        let e = &name[dot_pos..];
        (b.to_string(), e.to_string())
    } else {
        (name.clone(), String::new())
    };

    for index in 0..1000 {
        let candidate = if index == 0 {
            format!("{dir}/{name}")
        } else {
            format!("{dir}/{base}-{index}{ext}")
        };

        // Try hard link (atomic)
        match std::fs::hard_link(staged, &candidate) {
            Ok(_) => {
                let _ = fs::remove_file(staged);
                return Some(candidate);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return None,
        }
    }
    None
}

fn announce(path: &str, dir: &str) {
    let name = Path::new(path).file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let home = std::env::var("HOME").unwrap_or_default();
    let short_dir = dir.replace(&home, "~");
    let desc = format!("Saved to {short_dir}");

    let lower = name.to_lowercase();
    let is_image = lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg")
        || lower.ends_with(".gif") || lower.ends_with(".webp") || lower.ends_with(".avif")
        || lower.ends_with(".bmp") || lower.ends_with(".tif") || lower.ends_with(".tiff");

    let mut args: Vec<String> = vec![format!("Received {name}"), desc, "-u".to_string(), "critical".to_string()];

    if is_image {
        args.push("--image".to_string());
        args.push(path.to_string());
    } else {
        args.push("-g".to_string());
        args.push("󰒊".to_string());
    }
    args.push("--exec".to_string());
    args.push("xdg-open".to_string());
    args.push(path.to_string());

    let _ = Command::new("omarchy-notification-send").args(&args).status();
}

pub fn send(machine: &str, files: &[String]) -> i32 {
    let name = machine.split('.').next().unwrap_or(machine);

    let files_to_send: Vec<String> = if files.is_empty() {
        // Interactive file selection
        let out = Command::new("omarchy-file-select")
            .args(["--title", &format!("Send to {name}"), "--multiple"])
            .output();

        match out {
            Ok(o) => {
                if !o.status.success() && o.status.code() != Some(0) {
                    if o.status.code().map(|c| c > 1).unwrap_or(false) {
                        Command::new("omarchy-notification-send")
                            .args(["-g", "󰒊", "-u", "critical",
                                &format!("Could not send to {name}"),
                                "The file chooser did not open"])
                            .status().ok();
                        return 1;
                    }
                }
                let s = String::from_utf8_lossy(&o.stdout);
                if s.trim().is_empty() { return 0; }
                s.lines().map(|l| l.to_string()).collect()
            }
            Err(_) => {
                Command::new("omarchy-notification-send")
                    .args(["-g", "󰒊", "-u", "critical",
                        &format!("Could not send to {name}"),
                        "The file chooser did not open"])
                    .status().ok();
                return 1;
            }
        }
    } else {
        files.to_vec()
    };

    let what = if files_to_send.len() == 1 {
        Path::new(&files_to_send[0]).file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| files_to_send[0].clone())
    } else {
        format!("{} files", files_to_send.len())
    };

    let dest = format!("{machine}:");
    let out = Command::new("tailscale")
        .args(["file", "cp", "--update-interval=0", "--"])
        .args(&files_to_send)
        .arg(&dest)
        .output();

    match out {
        Ok(o) if o.status.success() => {
            Command::new("omarchy-notification-send")
                .args(["-g", "󰒊", &format!("Sent to {name}"), &what])
                .status().ok();
            0
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            let err = err.trim();
            let msg = if err.is_empty() { "Taildrop transfer failed".to_string() } else { err.to_string() };
            Command::new("omarchy-notification-send")
                .args(["-g", "󰒊", "-u", "critical",
                    &format!("Could not send to {name}"), &msg])
                .status().ok();
            1
        }
        Err(e) => {
            Command::new("omarchy-notification-send")
                .args(["-g", "󰒊", "-u", "critical",
                    &format!("Could not send to {name}"), &e.to_string()])
                .status().ok();
            1
        }
    }
}
