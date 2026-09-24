use std::process::Command;

pub fn direct_boot() -> i32 {
    // Check UEFI
    if !std::path::Path::new("/sys/firmware/efi").is_dir() {
        eprintln!("Error: System is not booted in UEFI mode");
        return 1;
    }

    let efi_ok = Command::new("efibootmgr").stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null()).status().map(|s| s.success()).unwrap_or(false);
    if !efi_ok {
        eprintln!("Error: efibootmgr is not available or not functional");
        return 1;
    }

    // Check BIOS vendor
    let bios_vendor = std::fs::read_to_string("/sys/class/dmi/id/bios_vendor")
        .unwrap_or_default().to_lowercase();
    if bios_vendor.contains("american megatrends") {
        eprintln!("Error: American Megatrends firmware may not safely support custom EFI entries");
        return 1;
    }
    if bios_vendor.contains("apple") {
        eprintln!("Error: Apple firmware uses its own boot manager");
        return 1;
    }

    // Check for existing Omarchy EFI entry
    let efi_list = Command::new("efibootmgr").stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null()).output();
    let efi_output = efi_list.map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let existing = efi_output.lines()
        .find(|l| {
            let re_like = l.contains("Omarchy");
            re_like && l.starts_with("Boot")
        });

    if let Some(entry) = existing {
        let boot_num = entry.get(4..8).unwrap_or("").to_string();
        // Ask to remove
        let confirmed = gum_confirm("Disable direct boot (remove Omarchy EFI entry)?");
        if confirmed {
            println!("Removing EFI boot entry {boot_num}");
            let _ = Command::new("sudo")
                .args(["efibootmgr", "--bootnum", &boot_num, "--delete-bootnum"])
                .status();
        }
    } else {
        // Find UKI file
        let find_out = Command::new("sudo")
            .args(["find", "/boot/EFI/Linux/", "-name", "omarchy*.efi", "-printf", "%f\n"])
            .output();
        let uki_file = find_out.map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .map(|l| l.to_string())
                .unwrap_or_default()
        }).unwrap_or_default();

        if uki_file.is_empty() {
            eprintln!("Error: No Omarchy UKI found in /boot/EFI/Linux/");
            return 1;
        }

        // Get boot partition info
        let findmnt = Command::new("findmnt")
            .args(["-n", "-o", "SOURCE", "/boot"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        // disk = source without trailing partition number
        let disk = strip_partition(&findmnt);
        let part = extract_partition_num(&findmnt);

        let confirmed = gum_confirm("Setup direct boot (so snapshot booting must be done via bios)?");
        if confirmed {
            println!("Creating EFI boot entry for {uki_file}");
            let _ = Command::new("sudo")
                .args(["efibootmgr", "--create",
                    "--disk", &disk,
                    "--part", &part,
                    "--label", "Omarchy",
                    "--loader", &format!("\\EFI\\Linux\\{uki_file}")])
                .status();
        }
    }
    0
}

fn gum_confirm(msg: &str) -> bool {
    Command::new("gum").args(["confirm", msg])
        .status().map(|s| s.success()).unwrap_or(false)
}

fn strip_partition(source: &str) -> String {
    // Remove trailing p\d+ or \d+ (partition number)
    let bytes = source.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1].is_ascii_digit() { end -= 1; }
    if end > 0 && bytes[end - 1] == b'p' { end -= 1; }
    source[..end].to_string()
}

fn extract_partition_num(source: &str) -> String {
    // Get trailing digits (after optional 'p')
    let s = source.trim_end_matches(|c: char| c.is_ascii_digit());
    let num_str = &source[s.len()..];
    num_str.to_string()
}
