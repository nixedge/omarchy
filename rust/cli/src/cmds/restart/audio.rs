use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn run() -> i32 {
    restart_audio_services();

    if !wpctl_healthy() {
        eprintln!("\nPipeWire is still not responding. Checking for stuck USB audio devices...");

        if recover_stuck_usb_audio() {
            eprintln!("\nRestarting audio services after USB audio reset...\n");
            restart_audio_services();
        } else {
            eprintln!("No stuck USB audio device was detected automatically.");
        }
    }

    std::thread::sleep(Duration::from_secs(2));

    println!("\nAudio status:\n");
    let status = Command::new("wpctl").arg("status").status();
    match status {
        Ok(s) if s.success() => 0,
        _ => {
            eprintln!("Audio services are still not responding. Try replugging or power-cycling the selected USB audio device.");
            1
        }
    }
}

fn restart_audio_services() {
    let services = ["wireplumber.service", "pipewire.service", "pipewire-pulse.service"];
    println!("Restarting audio services...\n");

    let status = Command::new("systemctl")
        .args(["--user", "restart"])
        .args(services)
        .status();

    if status.map(|s| s.success()).unwrap_or(false) {
        return;
    }

    eprintln!("\nAudio services did not restart cleanly. Forcing stuck services down...\n");
    let _ = Command::new("systemctl").args(["--user", "cancel"]).status();
    let _ = Command::new("systemctl")
        .args(["--user", "kill", "--kill-whom=all", "--signal=KILL"])
        .args(services)
        .status();
    let _ = Command::new("systemctl")
        .args(["--user", "reset-failed"])
        .args(services)
        .status();

    let _ = Command::new("systemctl")
        .args(["--user", "start", "pipewire.service", "pipewire-pulse.service", "wireplumber.service"])
        .status();
}

fn wpctl_healthy() -> bool {
    Command::new("wpctl")
        .arg("status")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn usb_device_for_card(card: &str) -> Option<String> {
    let path = fs::read_link(format!("/sys/class/sound/card{card}")).ok()?;
    let mut current = path;
    loop {
        if current.join("idVendor").exists()
            && current.join("idProduct").exists()
            && current.join("busnum").exists()
            && current.join("devnum").exists()
        {
            return current.file_name()?.to_str().map(|s| s.to_string());
        }
        let parent = current.parent()?.to_path_buf();
        if parent == current {
            break;
        }
        current = parent;
    }
    None
}

fn audio_process_blocked_on_usb() -> bool {
    let Ok(proc_dir) = fs::read_dir("/proc") else { return false; };
    for entry in proc_dir.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let base = entry.path();
        let comm = fs::read_to_string(base.join("comm")).unwrap_or_default();
        let comm = comm.trim();
        if comm != "wireplumber" && comm != "pipewire" {
            continue;
        }
        let wchan = fs::read_to_string(base.join("wchan")).unwrap_or_default();
        let wchan = wchan.trim();
        if wchan.starts_with("usb_") || wchan.contains("usb") {
            return true;
        }
    }
    false
}

fn usb_audio_devices() -> Vec<String> {
    let mut devices: Vec<String> = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/sound") else { return devices; };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("card") {
            continue;
        }
        let card_num = &name_str["card".len()..];
        if !card_num.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        if let Some(dev) = usb_device_for_card(card_num) {
            if !devices.contains(&dev) {
                devices.push(dev);
            }
        }
    }
    devices
}

fn stuck_usb_audio_devices() -> Vec<String> {
    let mut devices: Vec<String> = Vec::new();

    // Walk /proc/asound/card*/pcm*p/sub*/status
    let Ok(asound) = fs::read_dir("/proc/asound") else { return devices; };
    for card_entry in asound.flatten() {
        let card_name = card_entry.file_name();
        let card_str = card_name.to_string_lossy();
        if !card_str.starts_with("card") {
            continue;
        }
        let card_num = &card_str["card".len()..];
        let Ok(pcm_dir) = fs::read_dir(card_entry.path()) else { continue; };
        for pcm_entry in pcm_dir.flatten() {
            let pcm_name = pcm_entry.file_name();
            let pcm_str = pcm_name.to_string_lossy();
            if !pcm_str.ends_with('p') {
                continue;
            }
            let Ok(sub_dir) = fs::read_dir(pcm_entry.path()) else { continue; };
            for sub_entry in sub_dir.flatten() {
                let status_path = sub_entry.path().join("status");
                let status = fs::read_to_string(&status_path).unwrap_or_default();

                // Find state line
                let state = status.lines()
                    .find(|l| l.starts_with("state:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .unwrap_or("");
                if state != "SETUP" {
                    continue;
                }

                // Find owner_pid
                let owner_pid = status.lines()
                    .find(|l| l.contains("owner_pid"))
                    .and_then(|l| l.split_whitespace().last())
                    .unwrap_or("")
                    .to_string();

                if owner_pid.is_empty() {
                    continue;
                }

                let comm = fs::read_to_string(format!("/proc/{owner_pid}/comm"))
                    .unwrap_or_default();
                let comm = comm.trim();
                if comm != "wireplumber" && comm != "pipewire" {
                    continue;
                }

                if let Some(dev) = usb_device_for_card(card_num) {
                    if !devices.contains(&dev) {
                        devices.push(dev);
                    }
                }
            }
        }
    }
    devices
}

fn clear_usb_audio_defaults() {
    let state_home = std::env::var("XDG_STATE_HOME")
        .unwrap_or_else(|_| format!("{}/.local/state", std::env::var("HOME").unwrap_or_default()));
    let state_dir = format!("{state_home}/wireplumber");

    if !Path::new(&state_dir).is_dir() {
        return;
    }

    let nodes_file = format!("{state_dir}/default-nodes");
    if Path::new(&nodes_file).exists() {
        // backup and remove USB sink lines
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = fs::copy(&nodes_file, format!("{nodes_file}.bak.{ts}"));
        if let Ok(content) = fs::read_to_string(&nodes_file) {
            let filtered: String = content.lines()
                .filter(|l| {
                    !l.starts_with("default.configured.audio.sink=alsa_output.usb-")
                        && !l.contains("=alsa_output.usb-")
                })
                .map(|l| format!("{l}\n"))
                .collect();
            let _ = fs::write(&nodes_file, filtered);
        }
    }

    let routes_file = format!("{state_dir}/default-routes");
    if Path::new(&routes_file).exists() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = fs::copy(&routes_file, format!("{routes_file}.bak.{ts}"));
        if let Ok(content) = fs::read_to_string(&routes_file) {
            let filtered: String = content.lines()
                .filter(|l| !l.starts_with("alsa_card.usb-"))
                .map(|l| format!("{l}\n"))
                .collect();
            let _ = fs::write(&routes_file, filtered);
        }
    }
}

fn reset_usb_audio_device(device: &str) -> bool {
    let sysfs = format!("/sys/bus/usb/devices/{device}");
    if !Path::new(&sysfs).is_dir() {
        return false;
    }

    // Check usbreset is available
    let usbreset_ok = Command::new("omarchy-cmd-present")
        .arg("usbreset")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !usbreset_ok {
        println!("usbreset is not installed; replug or power-cycle {device} to recover it.");
        return false;
    }

    let busnum = fs::read_to_string(format!("{sysfs}/busnum")).unwrap_or_default();
    let devnum = fs::read_to_string(format!("{sysfs}/devnum")).unwrap_or_default();
    let product = fs::read_to_string(format!("{sysfs}/product"))
        .unwrap_or_else(|_| device.to_string());

    let busnum = busnum.trim().parse::<u32>().unwrap_or(0);
    let devnum = devnum.trim().parse::<u32>().unwrap_or(0);
    let bus_device = format!("{busnum:03}/{devnum:03}");

    eprintln!("\nResetting stuck USB audio device: {} ({})...", product.trim(), bus_device);

    let ok = Command::new("sudo")
        .args(["usbreset", &bus_device])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok {
        println!(
            "Could not reset {} automatically. Replug or power-cycle it, then run omarchy-restart-audio again.",
            product.trim()
        );
    }
    ok
}

fn recover_stuck_usb_audio() -> bool {
    let mut detected = stuck_usb_audio_devices();

    if detected.is_empty() && audio_process_blocked_on_usb() {
        eprintln!("Audio service is blocked in USB I/O; resetting USB audio devices...");
        clear_usb_audio_defaults();
        detected = usb_audio_devices();
    }

    if detected.is_empty() {
        return false;
    }

    let mut reset_count = 0;
    for device in &detected {
        if reset_usb_audio_device(device) {
            reset_count += 1;
        }
    }
    reset_count > 0
}
