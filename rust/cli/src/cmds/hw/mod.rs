pub mod check;
pub mod detect;
pub mod state;

use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Shared low-level detection helpers used by both detect and check.
// ---------------------------------------------------------------------------

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default().trim().to_string()
}

fn file_contains(path: &str, needle: &str) -> bool {
    read_file(path).to_lowercase().contains(&needle.to_lowercase())
}

fn file_eq(path: &str, value: &str) -> bool {
    read_file(path).eq_ignore_ascii_case(value.trim())
}

// ---------------------------------------------------------------------------
// PCI-based detection (reads cached sysfs, never wakes suspended devices)
// ---------------------------------------------------------------------------

fn pci_devices_path() -> String {
    std::env::var("OMARCHY_PCI_DEVICES_PATH")
        .unwrap_or_else(|_| "/sys/bus/pci/devices".to_string())
}

struct PciDevice {
    vendor: u32,
    class: u32,
    device_id: u32,
}

fn pci_devices() -> Vec<PciDevice> {
    let base = pci_devices_path();
    let Ok(entries) = fs::read_dir(&base) else {
        return vec![];
    };
    entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            let vendor = u32::from_str_radix(
                read_file(p.join("vendor").to_str()?).trim_start_matches("0x"),
                16,
            )
            .ok()?;
            let class = u32::from_str_radix(
                read_file(p.join("class").to_str()?).trim_start_matches("0x"),
                16,
            )
            .ok()?;
            let device_id = u32::from_str_radix(
                read_file(p.join("device").to_str()?).trim_start_matches("0x"),
                16,
            )
            .ok()?;
            Some(PciDevice { vendor, class, device_id })
        })
        .collect()
}

fn is_gpu_class(class: u32) -> bool {
    (class >> 16) == 0x03
}

// ---------------------------------------------------------------------------
// Static identity checks
// ---------------------------------------------------------------------------

fn dmi(field: &str) -> String {
    let path = format!("/sys/class/dmi/id/{field}");
    read_file(&path)
}

pub fn dmi_match(pattern: &str) -> bool {
    let pat = pattern.to_lowercase();
    dmi("product_name").to_lowercase().contains(&pat)
        || dmi("product_family").to_lowercase().contains(&pat)
}

pub fn check_nvidia() -> bool {
    pci_devices()
        .iter()
        .any(|d| d.vendor == 0x10de && is_gpu_class(d.class))
}

pub fn check_nvidia_gsp() -> bool {
    pci_devices()
        .iter()
        .any(|d| d.vendor == 0x10de && is_gpu_class(d.class) && d.device_id >= 0x1e00)
}

pub fn check_nvidia_without_gsp() -> bool {
    pci_devices()
        .iter()
        .any(|d| d.vendor == 0x10de && is_gpu_class(d.class) && d.device_id >= 0x1340 && d.device_id < 0x1e00)
}

pub fn check_intel_cpu() -> bool {
    fs::read_to_string("/proc/cpuinfo")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("vendor_id"))
        .and_then(|l| l.split(':').nth(1))
        .map(|v| v.trim() == "GenuineIntel")
        .unwrap_or(false)
}

pub fn check_intel_ptl() -> bool {
    lspci_contains("vga|3d|display", "panther lake")
}

pub fn check_intel_sof() -> bool {
    lspci_contains("multimedia audio controller|audio device", "intel")
}

fn lspci_contains(class_pattern: &str, name_pattern: &str) -> bool {
    use std::process::{Command, Stdio};
    let Ok(out) = Command::new("lspci")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    else {
        return false;
    };
    let output = String::from_utf8_lossy(&out.stdout).to_lowercase();
    let class_re = class_pattern.to_lowercase();
    let name_re = name_pattern.to_lowercase();
    output.lines().any(|line| {
        let matches_class = class_re.split('|').any(|c| line.contains(c));
        matches_class && line.contains(&name_re)
    })
}

pub fn check_laptop() -> bool {
    // Lid switch is the definitive signal.
    if glob_exists("/proc/acpi/button/lid/*/state") {
        return true;
    }
    // DMI chassis type fallback: 8=Portable 9=Laptop 10=Notebook 14=Sub Notebook
    // 30=Tablet 31=Convertible 32=Detachable
    match read_file("/sys/class/dmi/id/chassis_type")
        .parse::<u32>()
        .unwrap_or(0)
    {
        8 | 9 | 10 | 14 | 30 | 31 | 32 => true,
        _ => false,
    }
}

pub fn check_fingerprint() -> bool {
    let base = std::env::var("OMARCHY_USB_DEVICES_PATH")
        .unwrap_or_else(|_| "/sys/bus/usb/devices".to_string());
    let Ok(entries) = fs::read_dir(&base) else {
        return false;
    };
    // Vendors known to ship fingerprint readers exclusively.
    let fp_vendors = ["27c6", "138a", "06cb", "08ff", "1c7a", "147e"];

    for entry in entries.flatten() {
        let dev = entry.path();

        // Trust the product descriptor first.
        let product = read_file(dev.join("product").to_str().unwrap_or("")).to_lowercase();
        if product.contains("fingerprint")
            || product.contains("biometric")
            || product.contains("elan:arm-m4")
            || product.starts_with("fpc ")
        {
            return true;
        }

        // Vendor-ID guess: only match if no kernel driver is bound (libfprint uses libusb).
        let vendor = read_file(dev.join("idVendor").to_str().unwrap_or("")).to_lowercase();
        if fp_vendors.contains(&vendor.as_str()) && !has_kernel_driver(&dev) {
            return true;
        }
    }
    false
}

fn has_kernel_driver(dev: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dev) else {
        return false;
    };
    for e in entries.flatten() {
        let name = e.file_name();
        let name = name.to_string_lossy();
        // Interface directories match "N:N" (busnum:devnum format).
        if !name.contains(':') {
            continue;
        }
        let driver_path = e.path().join("driver");
        if !driver_path.exists() {
            continue;
        }
        let driver = fs::read_link(&driver_path)
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_default();
        if driver != "usbfs" {
            return true;
        }
    }
    false
}

pub fn check_vulkan() -> bool {
    let icd_dir = "/usr/share/vulkan/icd.d";
    if !Path::new(icd_dir).is_dir() {
        return false;
    }
    fs::read_dir(icd_dir)
        .ok()
        .map(|mut d| d.any(|e| e.map(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json")).unwrap_or(false)))
        .unwrap_or(false)
}

pub fn check_hybrid_gpu() -> bool {
    use std::process::{Command, Stdio};
    if let Ok(out) = Command::new("supergfxctl")
        .args(["-s"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        // If supergfxctl timed out or was killed, fall through to lspci.
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout)
                .to_lowercase()
                .contains("hybrid");
        }
    }
    // Count display-class PCI devices.
    pci_devices().iter().filter(|d| is_gpu_class(d.class)).count() >= 2
}

pub fn check_asus_rog() -> bool {
    dmi("sys_vendor").eq_ignore_ascii_case("ASUSTeK COMPUTER INC.")
        && file_contains("/sys/class/dmi/id/product_family", "ROG")
}

pub fn check_asus_expertbook() -> bool {
    dmi_match("B9406") && check_intel_ptl()
}

pub fn check_asus_zenbook() -> bool {
    dmi_match("ux5406aa") && check_intel_ptl()
}

pub fn check_framework16() -> bool {
    dmi("sys_vendor").eq_ignore_ascii_case("Framework")
        && dmi_match("Laptop 16")
}

pub fn check_surface() -> bool {
    dmi("sys_vendor").eq_ignore_ascii_case("Microsoft Corporation")
        && dmi_match("Surface")
}

pub fn check_dell_xps_oled() -> bool {
    if !dmi_match("XPS") || !check_intel_ptl() {
        return false;
    }
    // Check EDID bytes 8-9 for LG OLED panel (0x30 0xe4).
    let pattern = glob_first("/sys/class/drm/card*-eDP-*/edid");
    let Some(edid_path) = pattern else {
        return false;
    };
    let Ok(bytes) = fs::read(&edid_path) else {
        return false;
    };
    bytes.len() >= 10 && bytes[8] == 0x30 && bytes[9] == 0xe4
}

pub fn check_dell_xps_haptic() -> bool {
    dmi_match("XPS") && Path::new("/sys/bus/i2c/devices/i2c-VEN_06CB:00").exists()
}

pub fn check_dell_xps13_sidecar_amps() -> bool {
    let sku_path = std::env::var("OMARCHY_DMI_PRODUCT_SKU")
        .unwrap_or_else(|_| "/sys/class/dmi/id/product_sku".to_string());
    dmi_match("DX13260") && file_eq(&sku_path, "0E53")
}

pub fn check_elgato_camlink() -> bool {
    let Ok(entries) = fs::read_dir("/sys/bus/usb/devices") else {
        return false;
    };
    entries.flatten().any(|e| {
        file_eq(
            e.path().join("product").to_str().unwrap_or(""),
            "Cam Link 4K",
        )
    })
}

// ---------------------------------------------------------------------------
// Utility: glob-style existence check (single wildcard segment)
// ---------------------------------------------------------------------------

fn glob_exists(pattern: &str) -> bool {
    glob_first(pattern).is_some()
}

fn glob_first(pattern: &str) -> Option<std::path::PathBuf> {
    // Split at the first '*' component.
    let path = Path::new(pattern);
    let mut parts = path.components().peekable();
    let mut base = std::path::PathBuf::new();

    while let Some(part) = parts.next() {
        let s = part.as_os_str().to_string_lossy();
        if s.contains('*') {
            // Read the parent dir and match the glob fragment.
            let prefix = s.replace('*', "");
            let suffix_parts: Vec<_> = parts.collect();
            let Ok(entries) = fs::read_dir(&base) else {
                return None;
            };
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with(prefix.as_str()) {
                    let mut candidate = entry.path();
                    for sp in &suffix_parts {
                        candidate.push(sp);
                    }
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
            return None;
        } else {
            base.push(part);
        }
    }
    if base.exists() { Some(base) } else { None }
}
