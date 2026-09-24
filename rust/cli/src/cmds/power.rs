use std::fs;

pub fn present() -> i32 {
    let ps_path = std::env::var("OMARCHY_POWER_SUPPLY_PATH")
        .unwrap_or_else(|_| "/sys/class/power_supply".to_string());

    let Ok(entries) = fs::read_dir(&ps_path) else { return 1; };
    for entry in entries.flatten() {
        let path = entry.path();
        let type_str = fs::read_to_string(path.join("type"))
            .unwrap_or_default()
            .trim()
            .to_string();
        if type_str != "Mains" && type_str != "USB" {
            continue;
        }
        let online = fs::read_to_string(path.join("online"))
            .unwrap_or_default()
            .trim()
            .to_string();
        if online == "1" {
            return 0;
        }
    }
    1
}
