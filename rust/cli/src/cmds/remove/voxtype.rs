use crate::cmds::drop;
use std::process::Command;

pub fn run() -> i32 {
    // Check if voxtype is installed
    let voxtype_present = Command::new("which")
        .arg("voxtype")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !voxtype_present {
        println!("Voxtype was not installed.");
        return 0;
    }

    println!("Uninstall Voxtype to remove dictation.");

    let _ = Command::new("systemctl")
        .args(["--user", "disable", "--now", "voxtype.service"])
        .status();
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    let rc = drop::run("voxtype-bin");

    let home = std::env::var("HOME").unwrap_or_default();
    let _ = std::fs::remove_dir_all(format!("{home}/.config/voxtype"));
    let _ = std::fs::remove_dir_all(format!("{home}/.local/share/voxtype"));

    let _ = Command::new("hyprctl")
        .arg("reload")
        .stdout(std::process::Stdio::null())
        .status();

    rc
}
