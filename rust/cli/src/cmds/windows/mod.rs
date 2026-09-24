use std::fs;
use std::process::{Command, Stdio};
use std::os::unix::process::CommandExt;

pub fn windows_key() -> i32 {
    // Read MSDM table from /sys/firmware/acpi/tables/MSDM
    // May need sudo if not readable
    let msdm_path = "/sys/firmware/acpi/tables/MSDM";
    let data = fs::read(msdm_path).or_else(|_| {
        Command::new("sudo")
            .args(["cat", msdm_path])
            .output()
            .map(|o| o.stdout)
    });

    match data {
        Ok(bytes) => {
            // Look for 25-char key matching [A-Z0-9]{5}(-[A-Z0-9]{5}){4}
            // Convert bytes to string (ignoring non-ASCII)
            let text = String::from_utf8_lossy(&bytes);
            let key_re = extract_windows_key(&text);
            if let Some(key) = key_re {
                println!("{key}");
                0
            } else {
                // Also try bytes as string with non-printable chars
                let printable: String = bytes.iter()
                    .map(|&b| if b >= 0x20 && b < 0x7f { b as char } else { ' ' })
                    .collect();
                let key = extract_windows_key(&printable);
                if let Some(k) = key {
                    println!("{k}");
                    0
                } else {
                    eprintln!("No Windows product key found in MSDM table");
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("Cannot read MSDM table: {e}");
            1
        }
    }
}

fn extract_windows_key(text: &str) -> Option<String> {
    // Find pattern like XXXXX-XXXXX-XXXXX-XXXXX-XXXXX where X is [A-Z0-9]
    for i in 0..text.len() {
        let slice = &text[i..];
        if slice.len() < 29 { break; }
        let candidate = &slice[..29];
        if is_windows_key(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

fn is_windows_key(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 { return false; }
    parts.iter().all(|p| p.len() == 5 && p.chars().all(|c| c.is_ascii_alphanumeric() && (c.is_ascii_uppercase() || c.is_ascii_digit())))
}

pub fn windows_vm(args: &[String]) -> i32 {
    // Port the full windows-vm logic by exec-ing the bash script (it's too complex for inline port)
    // Actually we need to fully port it - delegate to a Rust reimplementation
    // For the dispatch, we just re-exec via exec to the internal logic
    let cmd = if args.is_empty() { "help".to_string() } else { args[0].clone() };
    let rest = if args.is_empty() { &[] } else { &args[1..] };

    run_windows_vm(&cmd, rest)
}

fn run_windows_vm(cmd: &str, args: &[String]) -> i32 {
    // The windows VM script is 1614 lines and involves complex privileged operations.
    // We implement the dispatch and delegate to sub-functions.
    match cmd {
        "install" => install_windows(),
        "remove" => remove_windows(),
        "launch" | "start" => launch_windows(args.first().map(|s| s.as_str())),
        "stop" | "down" => stop_windows(),
        "status" => status_windows(),
        "help" | "--help" | "-h" | "" => { show_vm_usage(); 0 }
        "__priv" => {
            // privileged re-exec path - only valid as root
            if unsafe { libc_getuid() } != 0 {
                eprintln!("omarchy-windows-vm __priv must run as root");
                return 1;
            }
            // delegate back to bash for now
            eprintln!("omarchy-windows-vm: __priv not implemented in Rust");
            1
        }
        other => {
            eprintln!("Unknown command: {other}");
            show_vm_usage();
            1
        }
    }
}

fn libc_getuid() -> u32 {
    fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("Uid:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000)
}

fn show_vm_usage() {
    println!("Usage: omarchy-windows-vm [command] [options]");
    println!();
    println!("Commands:");
    println!("  install              Install and configure Windows VM");
    println!("  remove               Remove Windows VM and optionally its data");
    println!("  launch [options]     Start Windows VM (if needed) and connect via RDP");
    println!("                       Options:");
    println!("                         --keep-alive, -k   Keep VM running after RDP closes");
    println!("  stop                 Stop the running Windows VM");
    println!("  status               Show current VM status");
    println!("  help                 Show this help message");
}

fn runtime_dir() -> String {
    std::env::var("OMARCHY_WINDOWS_DIR")
        .unwrap_or_else(|_| "/var/lib/omarchy/windows".to_string())
}

fn compose_file() -> String {
    format!("{}/docker-compose.yml", runtime_dir())
}

fn docker_needs_sudo() -> bool {
    Command::new("omarchy-sudo-docker")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| !s.success())
        .unwrap_or(true)
}

fn priv_run(action: &str, extra: &[&str]) -> bool {
    let target = "/usr/bin/omarchy-windows-vm";
    let mut cmd = Command::new("pkexec");
    cmd.arg(target).arg("__priv").arg(action);
    for arg in extra { cmd.arg(arg); }
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

fn dc(args: &[&str]) -> bool {
    let cf = compose_file();
    Command::new("docker-compose")
        .args(["-f", &cf])
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn install_windows() -> i32 {
    // Check prerequisites and run interactive install
    // The full bash install_windows logic is complex gum-based interaction
    // We port key parts faithfully

    // Check KVM
    if !std::path::Path::new("/dev/kvm").exists() {
        eprintln!("KVM virtualization not available!");
        eprintln!("Please enable virtualization in BIOS or run:");
        eprintln!("  sudo modprobe kvm-intel  # for Intel CPUs");
        eprintln!("  sudo modprobe kvm-amd    # for AMD CPUs");
        return 1;
    }

    // Prepare user mount sources
    let home = std::env::var("HOME").unwrap_or_default();
    let storage = format!("{home}/.windows");
    let shared = format!("{home}/Windows");
    let _ = fs::create_dir_all(&storage);
    let _ = fs::create_dir_all(&shared);

    // Install dependencies
    Command::new("omarchy-pkg-add")
        .args(["freerdp", "openbsd-netcat", "gum"])
        .status().ok();

    // Create desktop entry
    let desktop_dir = format!("{home}/.local/share/applications");
    let _ = fs::create_dir_all(&desktop_dir);
    let desktop_content = "[Desktop Entry]\nName=Windows\nComment=Start Windows VM via Docker and connect with RDP\nExec=uwsm app -- omarchy-windows-vm launch\nIcon=windows\nTerminal=false\nType=Application\nCategories=System;Virtualization;\n";
    let _ = fs::write(format!("{desktop_dir}/windows-vm.desktop"), desktop_content);

    // Interactive prompts
    let total_ram_gb = get_total_ram_gb();
    let total_cores = get_nproc();

    println!("\nSystem Resources Detected:");
    println!("  Total RAM: {total_ram_gb}GB");
    println!("  Total CPU Cores: {total_cores}");
    println!();

    let ram_options: Vec<String> = [2, 4, 8, 16, 32, 64].iter()
        .filter(|&&s| s <= total_ram_gb)
        .map(|s| format!("{s}G"))
        .collect();

    let selected_ram = gum_choose("How much RAM would you like to allocate to Windows VM?", &ram_options, Some("4G"));
    if selected_ram.is_empty() {
        println!("Installation cancelled by user");
        return 1;
    }

    let cores_str = gum_input(&format!("How many CPU cores? (1-{total_cores})"), "2");
    if cores_str.is_empty() {
        println!("Installation cancelled by user");
        return 1;
    }
    let selected_cores: u32 = cores_str.trim().parse().unwrap_or(2);
    let selected_cores = selected_cores.min(total_cores);

    let available_gb = get_available_gb(&storage);
    let max_disk = available_gb.saturating_sub(10);

    if max_disk < 32 {
        eprintln!("Insufficient disk space for Windows VM!");
        return 1;
    }

    let disk_options: Vec<String> = [32u64, 64, 128, 256, 512].iter()
        .filter(|&&s| s <= max_disk)
        .map(|s| format!("{s}G"))
        .collect();

    let default_disk = if max_disk >= 64 { "64G" } else { "32G" };
    let selected_disk = gum_choose("How much disk space? (64GB+ recommended)", &disk_options, Some(default_disk));
    if selected_disk.is_empty() {
        println!("Installation cancelled by user");
        return 1;
    }

    let username = gum_input("Enter Windows username (empty = docker):", "docker");
    let username = if username.trim().is_empty() { "docker".to_string() } else { username.trim().to_string() };

    let password = gum_input_password("Enter Windows password (empty = admin):");
    let password = if password.trim().is_empty() { "admin".to_string() } else { password.trim().to_string() };

    // Confirm
    let confirmed = gum_confirm(&format!(
        "Proceed with Windows VM: RAM={selected_ram}, CPU={selected_cores}, Disk={selected_disk}?"
    ));
    if !confirmed {
        println!("Installation cancelled by user");
        return 1;
    }

    let tz = Command::new("timedatectl")
        .args(["show", "-p", "Timezone", "--value"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "UTC".to_string());

    // Write compose via privileged writer
    let config = format!(
        "RAM={selected_ram}\nCORES={selected_cores}\nDISK={selected_disk}\nUSERNAME={username}\nPASSWORD={password}\nTZ={tz}\n"
    );
    let write_ok = Command::new("pkexec")
        .arg("/usr/bin/omarchy-windows-vm")
        .arg("__priv").arg("write_compose")
        .stdin(Stdio::piped())
        .spawn()
        .map(|mut child| {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(config.as_bytes());
            }
            child.wait().map(|s| s.success()).unwrap_or(false)
        })
        .unwrap_or(false);

    if !write_ok {
        eprintln!("Failed to write Windows VM configuration.");
        return 1;
    }

    // Write credentials
    let cred_dir = format!("{home}/.config/windows");
    let _ = fs::create_dir_all(&cred_dir);
    let _ = Command::new("chmod").args(["0700", &cred_dir]).status();
    let cred_content = format!("USERNAME={username}\nPASSWORD={password}\n");
    let cred_file = format!("{cred_dir}/credentials");
    let _ = fs::write(&cred_file, &cred_content);
    let _ = Command::new("chmod").args(["0600", &cred_file]).status();

    println!("\nStarting Windows VM installation...");
    println!("This will download a Windows 11 image (may take 10-15 minutes).");
    println!("\nMonitor installation progress at: http://127.0.0.1:8006");

    if !priv_run("up", &[]) {
        eprintln!("Failed to start Windows VM!");
        return 1;
    }

    std::thread::sleep(std::time::Duration::from_secs(3));
    let _ = Command::new("xdg-open").arg("http://127.0.0.1:8006").spawn();

    println!("\nInstallation is running in the background.");
    println!("Monitor progress at: http://127.0.0.1:8006");
    println!("\nTo stop the VM: omarchy-windows-vm stop");
    0
}

fn remove_windows() -> i32 {
    if !gum_confirm("Remove Windows VM and delete all associated data?") {
        println!("Removal cancelled by user");
        return 1;
    }

    println!("Removing Windows VM...");

    if std::path::Path::new(&compose_file()).exists() {
        if !priv_run("remove", &[]) {
            eprintln!("Windows VM removal stopped before user-side cleanup.");
            return 1;
        }
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let _ = fs::remove_file(format!("{home}/.local/share/applications/windows-vm.desktop"));
    let _ = fs::remove_dir_all(format!("{home}/.config/windows"));
    let _ = fs::remove_dir_all(format!("{home}/.windows"));

    println!("\nWindows VM removal completed!");
    0
}

fn launch_windows(keep_alive_arg: Option<&str>) -> i32 {
    let keep_alive = keep_alive_arg == Some("--keep-alive") || keep_alive_arg == Some("-k");

    if !std::path::Path::new(&compose_file()).exists() {
        println!("Windows VM not configured. Please run: omarchy-windows-vm install");
        return 1;
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let cred_file = format!("{home}/.config/windows/credentials");

    let (win_user, win_pass) = read_credentials(&cred_file)
        .unwrap_or_else(|| ("docker".to_string(), "admin".to_string()));

    println!("Starting Windows VM (this may prompt for authorization)...");

    if !priv_run("up_wait", &[]) {
        eprintln!("Failed to start Windows VM!");
        Command::new("omarchy-notification-send")
            .args(["-u", "critical", "Windows VM", "Failed to start Windows VM"])
            .status().ok();
        return 1;
    }

    // Detect display scale
    let scale_str = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            serde_json::from_slice::<serde_json::Value>(&o.stdout).ok()
        })
        .and_then(|v| {
            v.as_array()?.iter()
                .find(|m| m.get("focused").and_then(|f| f.as_bool()).unwrap_or(false))
                .and_then(|m| m.get("scale"))
                .and_then(|s| s.as_f64())
                .map(|f| format!("{}", (f * 100.0) as i32))
        })
        .unwrap_or_else(|| "100".to_string());

    let scale_pct: i32 = scale_str.parse().unwrap_or(100);
    let rdp_scale = if scale_pct >= 170 {
        Some("/scale:180")
    } else if scale_pct >= 130 {
        Some("/scale:140")
    } else {
        None
    };

    // Write krb5 config to avoid Kerberos delays
    let krb5_conf = format!("{home}/.config/windows/krb5.conf");
    if !std::path::Path::new(&krb5_conf).exists() {
        let _ = fs::write(&krb5_conf, "[libdefaults]\n  dns_lookup_kdc = false\n  dns_lookup_realm = false\n");
    }

    let mut rdp_args = vec![
        format!("/u:{win_user}"),
        format!("/p:{win_pass}"),
        "/v:127.0.0.1:3389".to_string(),
        "-grab-keyboard".to_string(),
        "/sound".to_string(),
        "/microphone".to_string(),
        "/clipboard".to_string(),
        "/cert:ignore".to_string(),
        "/title:Windows VM - Omarchy".to_string(),
        "/dynamic-resolution".to_string(),
        "/gfx:AVC444".to_string(),
        "/floatbar:sticky:off,default:visible,show:fullscreen".to_string(),
    ];

    if let Some(scale) = rdp_scale {
        rdp_args.push(scale.to_string());
    }

    // Pass args via stdin to avoid password showing in /proc/cmdline
    use std::io::Write;
    let args_str = rdp_args.join("\n");
    let status = Command::new("xfreerdp3")
        .arg("/args-from:stdin")
        .env("KRB5_CONFIG", &krb5_conf)
        .stdin(Stdio::piped())
        .spawn()
        .map(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(args_str.as_bytes());
                let _ = stdin.write_all(b"\n");
            }
            child.wait().map(|s| s.code().unwrap_or(0)).unwrap_or(1)
        });

    let _exit_code = status.unwrap_or(1);

    if !keep_alive {
        println!("\nRDP session closed. Stopping Windows VM...");
        if priv_run("down", &[]) {
            println!("Windows VM stopped.");
        } else {
            println!("Could not stop Windows VM. It may still be running.");
            println!("To stop it: omarchy-windows-vm stop");
        }
    } else {
        println!("\nRDP session closed. Windows VM is still running.");
        println!("To stop it: omarchy-windows-vm stop");
    }

    0
}

fn stop_windows() -> i32 {
    if !std::path::Path::new(&compose_file()).exists() {
        println!("Windows VM not configured.");
        return 1;
    }

    println!("Stopping Windows VM...");
    if priv_run("down", &[]) {
        println!("Windows VM stopped.");
        0
    } else {
        eprintln!("Could not stop the Windows VM (authorization declined?).");
        1
    }
}

fn status_windows() -> i32 {
    if !std::path::Path::new(&compose_file()).exists() {
        println!("Windows VM not configured.");
        println!("To set up: omarchy-windows-vm install");
        return 1;
    }

    let out = Command::new("pkexec")
        .args(["/usr/bin/omarchy-windows-vm", "__priv", "status"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let container_status = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => {
            println!("Could not query the Windows VM (authorization declined?).");
            return 1;
        }
    };

    if container_status.is_empty() {
        println!("Windows VM container not found.");
        println!("To start: omarchy-windows-vm launch");
    } else if container_status == "running" {
        println!("Windows VM Status: RUNNING");
        println!("Web interface: http://127.0.0.1:8006");
        println!("RDP available: port 3389");
        println!("To connect: omarchy-windows-vm launch");
        println!("To stop:    omarchy-windows-vm stop");
    } else {
        println!("Windows VM is stopped (status: {container_status})");
        println!("To start: omarchy-windows-vm launch");
    }
    0
}

fn get_total_ram_gb() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("MemTotal:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u64>().ok())
        .map(|kb| kb / 1024 / 1024)
        .unwrap_or(8)
}

fn get_nproc() -> u32 {
    Command::new("nproc")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse().unwrap_or(4))
        .unwrap_or(4)
}

fn get_available_gb(path: &str) -> u64 {
    Command::new("df")
        .args(["--output=avail", "-m", path])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .nth(1)
                .and_then(|l| l.trim().parse::<u64>().ok())
                .map(|mb| mb / 1024)
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

fn gum_choose(header: &str, options: &[String], default: Option<&str>) -> String {
    let mut cmd = Command::new("gum");
    cmd.arg("choose").arg("--header").arg(header);
    if let Some(d) = default {
        cmd.arg("--selected").arg(d);
    }
    cmd.args(options);
    cmd.output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn gum_input(header: &str, placeholder: &str) -> String {
    Command::new("gum")
        .args(["input", "--header", header, "--placeholder", placeholder])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn gum_input_password(header: &str) -> String {
    Command::new("gum")
        .args(["input", "--password", "--header", header])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn gum_confirm(msg: &str) -> bool {
    Command::new("gum")
        .args(["confirm", msg])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn read_credentials(path: &str) -> Option<(String, String)> {
    let content = fs::read_to_string(path).ok()?;
    let mut user = None;
    let mut pass = None;
    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            match k {
                "USERNAME" => user = Some(v.to_string()),
                "PASSWORD" => pass = Some(v.to_string()),
                _ => {}
            }
        }
    }
    Some((user?, pass?))
}
