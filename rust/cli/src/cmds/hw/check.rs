use super::*;

pub fn run(name: &str, pattern: Option<&str>) -> i32 {
    let result = match name {
        "nvidia" => check_nvidia(),
        "nvidia-gsp" => check_nvidia_gsp(),
        "nvidia-without-gsp" => check_nvidia_without_gsp(),
        "intel" => check_intel_cpu(),
        "intel-ptl" => check_intel_ptl(),
        "intel-sof" => check_intel_sof(),
        "laptop" => check_laptop(),
        "fingerprint" => check_fingerprint(),
        "vulkan" => check_vulkan(),
        "hybrid-gpu" => check_hybrid_gpu(),
        "asus-rog" => check_asus_rog(),
        "asus-expertbook" => check_asus_expertbook(),
        "asus-zenbook" => check_asus_zenbook(),
        "framework16" => check_framework16(),
        "surface" => check_surface(),
        "dell-xps-oled" => check_dell_xps_oled(),
        "dell-xps-haptic" => check_dell_xps_haptic(),
        "dell-xps13-sidecar-amps" => check_dell_xps13_sidecar_amps(),
        "elgato-camlink" => check_elgato_camlink(),
        "nvidia-display" => check_nvidia_display(),
        "vm" => check_vm(),
        "match" => {
            let pat = pattern.unwrap_or("");
            if pat.is_empty() {
                eprintln!("hw check match: pattern required");
                return 1;
            }
            dmi_match(pat)
        }
        _ => {
            eprintln!("hw check: unknown property '{name}'");
            return 1;
        }
    };
    if result { 0 } else { 1 }
}
