use super::{home_path, launch_desktop};
use crate::cmds::add;
use std::fs;
use std::process::Command;

pub fn helix() -> i32 {
    println!("Installing Helix…");
    let rc = add::run("helix", true);
    if rc != 0 {
        return rc;
    }

    let themes_dir = home_path(".config/helix/themes");
    let _ = fs::create_dir_all(&themes_dir);

    // Symlink the rendered Omarchy theme
    let theme_link = themes_dir.join("omarchy.toml");
    let theme_src = home_path(".local/state/omarchy/current/theme/helix.toml");
    let _ = fs::remove_file(&theme_link);
    let _ = std::os::unix::fs::symlink(&theme_src, &theme_link);

    // Seed config.toml if missing
    let config_path = home_path(".config/helix/config.toml");
    if !config_path.exists() {
        let _ = fs::write(&config_path, "theme = \"omarchy\"\n");
    }

    // Ensure the symlink target exists
    if !theme_src.exists() {
        let _ = Command::new("omarchy-theme-refresh").status();
    }

    0
}

pub fn vscode() -> i32 {
    println!("Installing VS Code…");
    let rc = add::run("vscode", true);
    if rc != 0 {
        return rc;
    }

    let vscode_dir = home_path(".vscode");
    let user_dir = home_path(".config/Code/User");
    let _ = fs::create_dir_all(&vscode_dir);
    let _ = fs::create_dir_all(&user_dir);

    let argv_json = r#"// This configuration file allows you to pass permanent command line arguments to VS Code.
{
  "password-store":"gnome-libsecret"
}
"#;
    let _ = fs::write(vscode_dir.join("argv.json"), argv_json);

    let settings_json = "{\n  \"update.mode\": \"none\"\n}\n";
    let _ = fs::write(user_dir.join("settings.json"), settings_json);

    let _ = Command::new("omarchy-theme-set-vscode").status();

    launch_desktop("code");
    0
}

pub fn emacs() -> i32 {
    println!("Installing Emacs…");
    // On NixOS, use the emacs package from nixpkgs
    // The omarchy-emacs AUR package is Arch-specific; use emacs directly
    let rc = add::run("emacs", true);
    if rc != 0 {
        return rc;
    }
    launch_desktop("emacsclient");
    0
}

pub fn zed() -> i32 {
    println!("Installing Zed Editor…");
    let rc = add::run("zed-editor", true);
    if rc != 0 {
        return rc;
    }

    let _ = Command::new("omazed").arg("setup").status();

    launch_desktop("dev.zed.Zed");
    0
}
