use crate::cli::TerminalName;
use crate::cmds::add;
use std::fs;
use std::process::Command;

pub fn install(name: TerminalName) -> i32 {
    let (pkg, desktop_id) = match name {
        TerminalName::Alacritty => ("alacritty", "Alacritty.desktop"),
        TerminalName::Foot => ("foot", "foot.desktop"),
        TerminalName::Ghostty => ("ghostty", "com.mitchellh.ghostty.desktop"),
        TerminalName::Kitty => ("kitty", "kitty.desktop"),
    };

    println!("Installing {pkg}…");
    let rc = add::run(pkg);
    if rc != 0 {
        return rc;
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let apps_dir = format!("{home}/.local/share/applications");
    let _ = fs::create_dir_all(&apps_dir);

    // Copy custom desktop entries for alacritty and foot
    match name {
        TerminalName::Alacritty => {
            let src = format!("{omarchy_path}/default/alacritty/{desktop_id}");
            if std::path::Path::new(&src).exists() {
                let _ = fs::copy(&src, format!("{apps_dir}/{desktop_id}"));
            }
        }
        TerminalName::Foot => {
            let src = format!("{omarchy_path}/applications/{desktop_id}");
            if std::path::Path::new(&src).exists() {
                let _ = fs::copy(&src, format!("{apps_dir}/{desktop_id}"));
            }
        }
        _ => {}
    }

    // Copy default config if missing
    let config_dir = format!("{home}/.config/{pkg}");
    if !std::path::Path::new(&config_dir).exists() {
        let src = format!("{omarchy_path}/config/{pkg}");
        if std::path::Path::new(&src).exists() {
            let _ = Command::new("cp")
                .args(["-Rpf", &src, &config_dir])
                .status();
        }
    }

    // Update xdg-terminals.list
    let terminals_list = format!("{home}/.config/xdg-terminals.list");
    let content = format!("# Terminal emulator preference order for xdg-terminal-exec\n{desktop_id}\n");
    let _ = fs::write(&terminals_list, content);

    0
}
