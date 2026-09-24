use crate::desktop;
use std::fs;
use std::path::Path;

pub fn run(
    name: Option<&str>,
    command: Option<&str>,
    window_style: Option<&str>,
    icon: Option<&str>,
) -> i32 {
    let icon_dir = desktop::user_icon_dir();
    let app_dir = desktop::user_app_dir();

    let (app_name, app_exec, win_style, icon_ref) = if let (
        Some(n), Some(c), Some(w), Some(i)
    ) = (name, command, window_style, icon) {
        (n.to_owned(), c.to_owned(), w.to_owned(), i.to_owned())
    } else {
        println!("\x1b[32mLet\u{2019}s create a TUI shortcut you can start with the app launcher.\n\x1b[0m");
        let n = match desktop::gum_input("Name> ", "My TUI") {
            Some(v) => v,
            None => { eprintln!("Cancelled."); return 1; }
        };
        let c = match desktop::gum_input("Launch Command> ", "lazydocker or bash -c 'dust; read -n 1 -s'") {
            Some(v) => v,
            None => { eprintln!("Cancelled."); return 1; }
        };
        let w = match desktop::gum_choose(&["float", "tile"], "Window style") {
            Some(v) => v.into_iter().next().unwrap_or_default(),
            None => { eprintln!("Cancelled."); return 1; }
        };
        let i = match desktop::gum_input("Icon URL/name> ", "See https://dashboardicons.com or enter an installed icon name") {
            Some(v) => v,
            None => { eprintln!("Cancelled."); return 1; }
        };
        (n, c, w, i)
    };

    if app_name.is_empty() || app_exec.is_empty() || icon_ref.is_empty() {
        eprintln!("You must set app name, app command, and icon URL/name!");
        return 1;
    }

    let icon_value = resolve_icon(&app_name, &icon_ref, &icon_dir);

    let app_class = if win_style == "float" { "TUI.float" } else { "TUI.tile" };

    let desktop_file = app_dir.join(format!("{app_name}.desktop"));
    if let Err(e) = fs::create_dir_all(&app_dir) {
        eprintln!("Failed to create applications dir: {e}");
        return 1;
    }

    let content = format!(
        "[Desktop Entry]\nVersion=1.0\nName={app_name}\nComment={app_name}\n\
         Exec=xdg-terminal-exec --app-id={app_class} -e {app_exec}\n\
         Terminal=false\nType=Application\nIcon={icon_value}\nStartupNotify=true\n"
    );

    if let Err(e) = fs::write(&desktop_file, &content) {
        eprintln!("Failed to write desktop file: {e}");
        return 1;
    }

    if name.is_none() {
        println!("You can now find {app_name} using the app launcher (SUPER + SPACE)\n");
    }

    0
}

fn resolve_icon(app_name: &str, icon_ref: &str, icon_dir: &std::path::Path) -> String {
    let lower = icon_ref.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        let icon_name = desktop::safe_icon_name(app_name);
        let dest = icon_dir.join(format!("{icon_name}.png"));
        let _ = fs::create_dir_all(icon_dir);
        if desktop::download_icon(icon_ref, &dest) {
            desktop::update_icon_cache(icon_dir);
        }
        icon_name
    } else if Path::new(icon_ref).is_file() {
        desktop::install_local_icon(Path::new(icon_ref), &desktop::safe_icon_name(app_name), icon_dir)
            .unwrap_or_else(|| desktop::safe_icon_name(app_name))
    } else {
        desktop::icon_name_from_ref(icon_ref)
    }
}
