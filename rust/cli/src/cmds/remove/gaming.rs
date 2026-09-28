use super::rm_rf;
use crate::cmds::{drop, install};
use std::process::Command;

pub fn steam() -> i32 {
    let rc = drop::run("steam");
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.steam",
        "$HOME/.local/share/Steam",
        "$HOME/.config/steam",
        "$HOME/.cache/steam",
    ]);
    println!("\nSteam and its data have been removed.");
    0
}

pub fn heroic() -> i32 {
    let rc = drop::run("heroic");
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.config/heroic",
        "$HOME/.local/share/heroic",
        "$HOME/.cache/heroic",
        "$HOME/Games/Heroic",
    ]);
    println!("\nHeroic and its data have been removed.");
    0
}

pub fn lutris() -> i32 {
    let rc = install::remove_many(&["lutris", "wine-staging", "wine-mono", "wine-gecko", "winetricks", "python-protobuf", "umu-launcher"]);
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.config/lutris",
        "$HOME/.local/share/lutris",
        "$HOME/.cache/lutris",
        "$HOME/.local/share/umu",
        "$HOME/.cache/umu",
        "$HOME/.wine",
        "$HOME/.cache/wine",
        "$HOME/.cache/winetricks",
    ]);
    println!("\nLutris, Wine, umu-launcher, and their configs have been removed.");
    0
}

pub fn retroarch() -> i32 {
    let rc = drop::run("retroarch-full");
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.config/retroarch",
        "$HOME/.local/share/retroarch",
        "$HOME/.cache/retroarch",
    ]);
    println!("\nRetroArch and its cores have been removed.");
    println!("ROMs and BIOS files at ~/Games/roms and ~/Games/bios were left in place.");
    0
}

pub fn minecraft() -> i32 {
    let rc = drop::run("minecraft-launchers");
    if rc != 0 { return rc; }
    rm_rf(&[
        "$HOME/.minecraft",
        "$HOME/.config/Minecraft Launcher",
        "$HOME/.local/share/minecraft-launcher",
        "$HOME/.cache/minecraft",
    ]);
    println!("\nMinecraft and its data have been removed.");
    0
}

pub fn xbox_controllers() -> i32 {
    let rc = drop::run("xpadneo-dkms");
    if rc != 0 { return rc; }
    // On NixOS these files would be in a declarative config, but clean up
    // any manually created ones
    let _ = std::fs::remove_file("/etc/modprobe.d/blacklist-xpad.conf");
    let _ = std::fs::remove_file("/etc/modules-load.d/xpadneo.conf");
    println!("\nXbox controller support removed. Reboot to fully unload xpadneo.");
    0
}

pub fn xbox_cloud() -> i32 {
    let status = Command::new("omarchy-webapp-remove").arg("Xbox Cloud Gaming").status();
    match status {
        Ok(s) if s.success() => 0,
        _ => 1,
    }
}

pub fn battlenet() -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let prefix = format!("{home}/Games/battlenet");

    // Kill any running processes
    let _ = Command::new("pkill").args(["-f", &prefix]).status();
    std::thread::sleep(std::time::Duration::from_secs(1));

    rm_rf(&[
        &prefix,
        "$HOME/.local/share/applications/battlenet.desktop",
        "$HOME/.cache/omarchy/Battle.net-Setup.exe",
    ]);

    let _ = Command::new("update-desktop-database")
        .arg(format!("{home}/.local/share/applications"))
        .status();

    println!("\nBattle.net and its Proton prefix have been removed.");
    0
}

pub fn geforce_now() -> i32 {
    let flatpak_ok = Command::new("flatpak")
        .args(["info", "com.nvidia.geforcenow"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if flatpak_ok {
        let _ = Command::new("flatpak")
            .args(["uninstall", "-y", "--delete-data", "com.nvidia.geforcenow"])
            .status();
    }

    println!("\nGeForce NOW removed.");
    0
}
