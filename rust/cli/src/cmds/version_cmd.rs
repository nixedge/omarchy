use std::fs;
use std::process::{Command, Stdio};

pub fn version() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH")
        .unwrap_or_else(|_| "/usr/share/omarchy".to_string());

    if omarchy_path != "/usr/share/omarchy" {
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
        // Read the version file shipped in the package
        let ver_file = format!("{omarchy_path}/version");
        match fs::read_to_string(&ver_file) {
            Ok(v) => {
                println!("{}", v.trim());
                0
            }
            Err(_) => {
                eprintln!("Could not determine omarchy version");
                1
            }
        }
    }
}

pub fn pkgs() -> i32 {
    // Show the NixOS system version and current generation
    let nixos_ver = Command::new("nixos-version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let current = fs::read_link("/run/current-system")
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    if let Some(ver) = nixos_ver {
        println!("NixOS {ver}");
    }
    if let Some(sys) = current {
        // Extract the short hash from the store path
        let short = sys.split('-').next().unwrap_or(&sys);
        println!("system: {short}");
    }
    0
}

pub fn channel() -> i32 {
    // NixOS Cinque always tracks nixos-unstable — there is no stable/rc/edge
    // channel distinction as in the Arch-based Quattro. Show the booted system
    // generation hash so users can correlate changelogs to their install.
    let gen_hash = fs::read_link("/run/booted-system")
        .ok()
        .and_then(|p| {
            p.to_str().and_then(|s| {
                // Store paths look like /nix/store/<hash>-nixos-system-...; extract hash.
                s.split('/').nth(3).map(|part| {
                    let hash = part.split('-').next().unwrap_or(part);
                    hash[..hash.len().min(8)].to_string()
                })
            })
        });

    match gen_hash {
        Some(h) => println!("nixos-unstable ({h})"),
        None => println!("nixos-unstable"),
    }
    0
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
