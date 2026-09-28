use super::{add_many, home_path, launch_desktop, launch_detached};
use crate::cmds::add;
use std::fs;
use std::process::Command;

pub fn steam() -> i32 {
    println!("Installing Steam…");
    let rc = add::run("steam");
    if rc != 0 {
        return rc;
    }
    gpu_lib32();
    println!("\nSteam will start automatically now. This might take a while…");
    launch_desktop("steam");
    0
}

pub fn heroic() -> i32 {
    println!("Installing Heroic Games Launcher…");
    let rc = add::run("heroic");
    if rc != 0 {
        return rc;
    }
    gpu_lib32();
    launch_desktop("heroic");
    0
}

pub fn lutris() -> i32 {
    println!("Installing Lutris…");
    let rc = add_many(&["lutris", "umu-launcher", "wine-staging", "wine-mono", "wine-gecko", "winetricks", "python-protobuf"], true);
    if rc != 0 {
        return rc;
    }
    gpu_lib32();
    println!("\nLutris will open and auto-fetch its DXVK and VKD3D runtimes in the background.");
    launch_detached("lutris", &[]);
    0
}

pub fn retroarch() -> i32 {
    println!("Installing RetroArch…");
    let rc = add::run("retroarch-full");
    if rc != 0 {
        return rc;
    }

    // Set up ~/Games for BIOS files and ROMs
    let games_dir = home_path("Games");
    let _ = fs::create_dir_all(games_dir.join("bios"));
    let _ = fs::create_dir_all(games_dir.join("roms"));

    // Write retroarch.cfg settings
    let cfg_path = home_path(".config/retroarch/retroarch.cfg");
    if let Some(parent) = cfg_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    set_retroarch_cfg(&cfg_path, &[
        ("rgui_browser_directory", &format!("{}/Games/roms", home_path("").display())),
        ("system_directory", &format!("{}/Games/bios", home_path("").display())),
        ("libretro_directory", "/run/current-system/sw/lib/libretro"),
        ("libretro_info_path", "/run/current-system/sw/share/libretro/info"),
        ("video_driver", "vulkan"),
        ("menu_driver", "xmb"),
        ("video_shader_enable", "true"),
        ("auto_shaders_enable", "true"),
        ("content_show_images", "false"),
        ("content_show_video", "false"),
    ]);

    // Default CRT shader
    let presets_dir = home_path(".config/retroarch/config");
    let _ = fs::create_dir_all(&presets_dir);
    let _ = fs::write(
        presets_dir.join("global.slangp"),
        "#reference \"/run/current-system/sw/share/libretro/shaders/shaders_slang/crt/crt-royale.slangp\"\n",
    );

    println!("\nPut your roms and bios files in ~/Games. Then start RetroArch from the app launcher.");
    let _ = Command::new("setsid")
        .args(["nautilus", &games_dir.to_string_lossy()])
        .spawn();
    0
}

fn set_retroarch_cfg(path: &std::path::Path, entries: &[(&str, &str)]) {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = existing.lines().map(|l| l.to_owned()).collect();

    for (key, value) in entries {
        let prefix = format!("{key} = ");
        if let Some(pos) = lines.iter().position(|l| l.starts_with(&prefix)) {
            lines[pos] = format!("{key} = \"{value}\"");
        } else {
            lines.push(format!("{key} = \"{value}\""));
        }
    }

    let _ = fs::write(path, lines.join("\n") + "\n");
}

pub fn xbox_controllers() -> i32 {
    println!("Installing Xbox controller Bluetooth support…");
    let rc = add::run("xpadneo-dkms");
    if rc != 0 {
        return rc;
    }

    // On NixOS, module configuration is handled declaratively; here we just
    // add the package. Users may need to add boot.blacklistedKernelModules = ["xpad"]
    // to their NixOS configuration for full effect.
    println!("\nNow you can pair your Xbox controller with Bluetooth using Super + Ctrl + B.");
    println!("Note: add 'boot.blacklistedKernelModules = [\"xpad\"]' to your NixOS config for best results.");
    0
}

pub fn xbox_cloud() -> i32 {
    println!("Installing Xbox Cloud Gaming…");
    let status = Command::new("omarchy-webapp-install")
        .args(["Xbox Cloud Gaming", "https://www.xbox.com/en-US/play", "https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/png/xbox.png"])
        .status();
    match status {
        Ok(s) if s.success() => {
            let _ = Command::new("setsid")
                .args(["omarchy-launch-webapp", "https://www.xbox.com/en-US/play"])
                .spawn();
            0
        }
        _ => 1,
    }
}

pub fn battlenet() -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let prefix = format!("{home}/Games/battlenet");
    let launcher = format!("{prefix}/drive_c/Program Files (x86)/Battle.net/Battle.net Launcher.exe");
    let installer_url = "https://downloader.battle.net/download/getInstallerForGame?os=win&gameProgram=BATTLENET_APP&version=Live";

    println!("Installing Battle.net…");

    let rc = add::run("umu-launcher");
    if rc != 0 {
        return rc;
    }
    gpu_lib32();

    let _ = fs::create_dir_all(&prefix);

    if !std::path::Path::new(&launcher).exists() {
        let cache_dir = format!("{home}/.cache/omarchy");
        let _ = fs::create_dir_all(&cache_dir);
        let installer = format!("{cache_dir}/Battle.net-Setup.exe");

        println!("\nDownloading Battle.net installer…");
        let dl = Command::new("curl")
            .args(["--fail", "--location", "--retry", "3", installer_url, "--output", &installer])
            .status();
        if !dl.map(|s| s.success()).unwrap_or(false) {
            eprintln!("Download failed.");
            return 1;
        }

        println!("\nLaunching the Battle.net setup wizard. Click through it normally.");
        let log = "/tmp/omarchy-battlenet-installer.log";
        let _ = Command::new("setsid")
            .env("WINEPREFIX", &prefix)
            .env("PROTONPATH", "GE-Proton")
            .env("GAMEID", "umu-battlenet")
            .env("PROTON_VERB", "run")
            .arg("sh")
            .arg("-c")
            .arg(format!("umu-run '{installer}' >'{log}' 2>&1"))
            .spawn();
        println!("Installer log: {log}");
    }

    // Install desktop entry
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let desktop_src = format!("{omarchy_path}/default/applications/battlenet.desktop");
    let apps_dir = format!("{home}/.local/share/applications");
    let _ = fs::create_dir_all(&apps_dir);
    let _ = fs::copy(&desktop_src, format!("{apps_dir}/battlenet.desktop"));
    let _ = Command::new("update-desktop-database").arg(&apps_dir).status();

    println!("\nBattle.net installer is running in the background.");
    0
}

pub fn geforce_now() -> i32 {
    println!("Installing GeForce NOW…");
    let rc = add::run("flatpak");
    if rc != 0 {
        return rc;
    }

    let dl = Command::new("curl")
        .args(["-LO", "https://international.download.nvidia.com/GFNLinux/GeForceNOWSetup.bin"])
        .current_dir("/tmp")
        .status();

    if !dl.map(|s| s.success()).unwrap_or(false) {
        eprintln!("Download failed.");
        return 1;
    }

    let _ = Command::new("chmod").args(["+x", "/tmp/GeForceNOWSetup.bin"]).status();
    let s = Command::new("/tmp/GeForceNOWSetup.bin").current_dir("/tmp").status();
    if !s.map(|s| s.success()).unwrap_or(false) {
        eprintln!("GeForce NOW setup failed.");
        return 1;
    }

    let _ = Command::new("setsid").arg("omarchy-launch-browser").spawn();
    0
}

pub fn gpu_lib32() {
    let lspci_out = Command::new("lspci")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();

    let mut pkgs: Vec<&str> = vec![];

    let has_intel = lspci_out.to_lowercase().contains("intel") && lspci_out.contains("VGA");
    let has_amd = lspci_out.to_lowercase().contains("amd") || lspci_out.to_lowercase().contains("radeon");

    // On NixOS, lib32 vulkan drivers come from nixpkgs
    if has_intel {
        pkgs.push("lib32-vulkan-intel"); // resolves to vulkan-loader / intel driver via alias
    }
    if has_amd {
        pkgs.push("lib32-vulkan-radeon");
    }

    // Check for NVIDIA via omarchy hw scripts
    let nvidia_gsp = Command::new("omarchy-hw-nvidia-gsp").status().map(|s| s.success()).unwrap_or(false);
    let nvidia_no_gsp = Command::new("omarchy-hw-nvidia-without-gsp").status().map(|s| s.success()).unwrap_or(false);

    if nvidia_gsp {
        pkgs.push("lib32-nvidia-utils");
    } else if nvidia_no_gsp {
        pkgs.push("lib32-nvidia-580xx-utils");
    }

    if !pkgs.is_empty() {
        println!("Installing lib32 graphics drivers…");
        let _ = add_many(&pkgs, true);
    }
}
