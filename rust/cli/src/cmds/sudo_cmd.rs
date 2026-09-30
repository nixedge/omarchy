use std::fs;
use std::process::{Command, Stdio};

pub fn docker(configured: bool) -> i32 {
    // Exit code convention (same as hw-* commands): 0 = condition is true ("needs sudo"),
    // 1 = condition is false ("no sudo needed").
    if configured {
        // Check if user is in docker group — if so, sudoless Docker is configured.
        let user = std::env::var("USER").unwrap_or_default();
        let groups = Command::new("id").arg("-Gn").arg(&user)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();
        // In docker group → no sudo needed → 1; not in group → needs sudo → 0.
        if groups.split_whitespace().any(|g| g == "docker") { 1 } else { 0 }
    } else {
        // Check if docker socket is writable this session.
        // Writable → no sudo needed → 1; not writable / missing → needs sudo → 0.
        let socket = std::env::var("OMARCHY_DOCKER_SOCKET")
            .unwrap_or_else(|_| "/var/run/docker.sock".to_string());
        Command::new("test").args(["-w", &socket])
            .status().map(|s| if s.success() { 1 } else { 0 }).unwrap_or(0)
    }
}

pub fn keepalive() -> i32 {
    // sudo -v once
    let init = Command::new("sudo").arg("-v").status();
    if !init.map(|s| s.success()).unwrap_or(false) {
        return 1;
    }

    // Spawn background loop: sudo -n true && sleep 60, repeated
    let _ = std::thread::Builder::new()
        .name("sudo-keepalive".to_string())
        .spawn(|| {
            loop {
                let ok = Command::new("sudo")
                    .args(["-n", "true"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if !ok { break; }
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        });
    0
}

pub fn passwordless(minutes: Option<u32>) -> i32 {
    let user = std::env::var("USER").unwrap_or_default();
    let nopasswd_file = format!("/etc/sudoers.d/99-omarchy-nopasswd-{user}");
    let timer_name = format!("omarchy-nopasswd-expire-{user}");
    let minutes = minutes.unwrap_or(15);

    println!("Toggle passwordless sudo...");

    // Safety: if file exists but timer doesn't (e.g. after reboot), clean up
    let file_exists = Command::new("sudo")
        .args(["test", "-f", &nopasswd_file])
        .status().map(|s| s.success()).unwrap_or(false);

    let timer_active = Command::new("systemctl")
        .args(["is-active", &format!("{timer_name}.timer")])
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);

    if file_exists && !timer_active {
        Command::new("sudo").args(["rm", &nopasswd_file]).status().ok();
    }

    let file_exists_now = Command::new("sudo")
        .args(["test", "-f", &nopasswd_file])
        .status().map(|s| s.success()).unwrap_or(false);

    if file_exists_now {
        // Already enabled
        // Update timer
        Command::new("sudo").args(["systemctl", "stop", &format!("{timer_name}.timer")])
            .status().ok();
        if !arm_expiry(&nopasswd_file, &timer_name, minutes) {
            return 1;
        }
        println!("Passwordless sudo timer updated. It will now automatically disable in {minutes} minutes.");
    } else {
        // Show warning
        println!();
        println!("WARNING: This will allow ANY process running as your user to");
        println!("execute ANY command as root WITHOUT a password for {minutes} minutes.");
        println!();
        println!("Passwordless sudo will automatically disable after {minutes} minutes.");
        println!("Run this command again to disable it early.");
        println!();

        if gum_confirm(&format!("Enable passwordless sudo for {minutes} minutes? This is a significant security risk!")) {
            let rule = format!("{user} ALL=(ALL) NOPASSWD: ALL\n");
            // Write via sudo tee
            use std::io::Write;
            let ok = Command::new("sudo")
                .args(["tee", &nopasswd_file])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .spawn()
                .map(|mut child| {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(rule.as_bytes());
                    }
                    child.wait().map(|s| s.success()).unwrap_or(false)
                })
                .unwrap_or(false);

            if !ok {
                eprintln!("Failed to write sudoers file");
                return 1;
            }

            Command::new("sudo").args(["chmod", "440", &nopasswd_file]).status().ok();

            if !arm_expiry(&nopasswd_file, &timer_name, minutes) {
                return 1;
            }

            println!();
            println!("Passwordless sudo has been ENABLED. It will automatically disable in {minutes} minutes.");
            println!("A restart removes the passwordless sudo rule as well.");
        } else {
            println!("Aborted. No changes made.");
        }
    }

    0
}

fn arm_expiry(nopasswd_file: &str, timer_name: &str, minutes: u32) -> bool {
    let unit_arg = format!("--on-active={minutes}m");
    let timer_prop = "--timer-property=AccuracySec=1s";
    let unit_flag = format!("--unit={timer_name}");

    let ok = Command::new("sudo")
        .args(["systemd-run", &unit_arg, timer_prop, &unit_flag,
            "rm", "-f", "--", nopasswd_file])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok {
        eprintln!("Failed to schedule passwordless sudo expiry. Revoking access now.");
        if !Command::new("sudo").args(["rm", "-f", "--", nopasswd_file])
            .status().map(|s| s.success()).unwrap_or(false)
        {
            eprintln!("CRITICAL: Could not remove {nopasswd_file}. Remove it as root immediately.");
        }
    }
    ok
}

fn gum_confirm(msg: &str) -> bool {
    Command::new("gum").args(["confirm", msg])
        .status().map(|s| s.success()).unwrap_or(false)
}
