use std::process::{Command, Stdio};

pub fn version() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();

    if omarchy_path != "/usr/share/omarchy" {
        // Dev checkout: git rev-parse --short HEAD
        let out = Command::new("git")
            .args(["-C", &omarchy_path, "rev-parse", "--short", "HEAD"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let hash = String::from_utf8_lossy(&o.stdout).trim().to_string();
                println!("dev ({hash})");
                0
            }
            _ => {
                println!("dev (unknown)");
                0
            }
        }
    } else {
        // Query pacman for omarchy-dev or omarchy package version
        for pkg in &["omarchy-dev", "omarchy"] {
            let out = Command::new("pacman")
                .args(["-Q", pkg])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output();
            if let Ok(o) = out {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout);
                    let ver = s.split_whitespace().nth(1).unwrap_or("unknown");
                    println!("{ver}");
                    return 0;
                }
            }
        }
        eprintln!("Could not determine omarchy version");
        1
    }
}

pub fn branch() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();

    if omarchy_path != "/usr/share/omarchy" {
        // Dev checkout
        let out = Command::new("git")
            .args(["-C", &omarchy_path, "branch", "--show-current"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        match out {
            Ok(o) if o.status.success() => {
                let branch = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !branch.is_empty() {
                    println!("{branch}");
                    return 0;
                }
                // Detached HEAD
                let hash_out = Command::new("git")
                    .args(["-C", &omarchy_path, "rev-parse", "--short", "HEAD"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output();
                let hash = hash_out
                    .ok()
                    .filter(|o| o.status.success())
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                println!("detached@{hash}");
                0
            }
            _ => {
                eprintln!("Could not determine branch");
                1
            }
        }
    } else {
        eprintln!("Not a dev checkout");
        1
    }
}
