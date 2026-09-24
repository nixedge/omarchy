use super::*;

pub fn run() -> i32 {
    let nvidia = check_nvidia();
    let nvidia_gsp = if nvidia { check_nvidia_gsp() } else { false };
    let nvidia_without_gsp = if nvidia { check_nvidia_without_gsp() } else { false };
    let intel_ptl = check_intel_ptl();

    println!(
        "{}",
        serde_json::json!({
            "gpu": {
                "nvidia": nvidia,
                "nvidia_gsp": nvidia_gsp,
                "nvidia_without_gsp": nvidia_without_gsp,
                "intel_cpu": check_intel_cpu(),
                "intel_ptl": intel_ptl,
                "intel_sof": check_intel_sof(),
                "hybrid": check_hybrid_gpu(),
                "vulkan": check_vulkan(),
            },
            "laptop": {
                "is_laptop": check_laptop(),
                "fingerprint": check_fingerprint(),
            },
            "vendor": {
                "asus_rog": check_asus_rog(),
                "asus_expertbook": check_asus_expertbook(),
                "asus_zenbook": check_asus_zenbook(),
                "framework16": check_framework16(),
                "surface": check_surface(),
                "dell_xps_oled": check_dell_xps_oled(),
                "dell_xps_haptic": check_dell_xps_haptic(),
                "dell_xps13_sidecar_amps": check_dell_xps13_sidecar_amps(),
            },
            "peripherals": {
                "elgato_camlink": check_elgato_camlink(),
            },
        })
    );
    0
}
