use crate::desktop;
use std::fs;
use std::path::Path;

pub fn run(
    name: Option<&str>,
    url: Option<&str>,
    icon: Option<&str>,
    custom_exec: Option<&str>,
    mime_types: Option<&str>,
) -> i32 {
    let icon_dir = desktop::user_icon_dir();
    let app_dir = desktop::user_app_dir();

    let (app_name, app_url, icon_ref, custom_exec_s, mime_types_s, interactive) =
        if let (Some(n), Some(u), Some(i)) = (name, url, icon) {
            let url_norm = desktop::normalize_url(u);
            if let Err(e) = desktop::require_http_url(&url_norm) {
                eprintln!("Error: {e}");
                return 1;
            }
            (n.to_owned(), url_norm, i.to_owned(),
             custom_exec.unwrap_or("").to_owned(),
             mime_types.unwrap_or("").to_owned(),
             false)
        } else {
            println!("\x1b[32mLet\u{2019}s create a new web app you can start with the app launcher.\n\x1b[0m");
            let n = match desktop::gum_input("Name> ", "My favorite web app") {
                Some(v) => v,
                None => { eprintln!("Cancelled."); return 1; }
            };
            if n.contains('/') {
                eprintln!("App name cannot contain '/'");
                return 1;
            }
            let u_raw = match desktop::gum_input("URL> ", "https://example.com") {
                Some(v) => v,
                None => { eprintln!("Cancelled."); return 1; }
            };
            let url_norm = desktop::normalize_url(&u_raw);
            if let Err(e) = desktop::require_http_url(&url_norm) {
                eprintln!("Error: {e}");
                return 1;
            }

            // Try to auto-fetch the site icon
            let icon_name = desktop::safe_icon_name(&n);
            let _ = fs::create_dir_all(&icon_dir);
            let dest = icon_dir.join(format!("{icon_name}.png"));
            let auto_icon = if desktop::fetch_site_icon(&url_norm, &dest) {
                desktop::update_icon_cache(&icon_dir);
                icon_name.clone()
            } else {
                // Fall back to asking user
                match desktop::gum_input(
                    "Icon URL/name> ",
                    "Could not fetch favicon automatically. Enter PNG icon URL or icon name",
                ) {
                    Some(v) => v,
                    None => { eprintln!("Cancelled."); return 1; }
                }
            };

            (n, url_norm, auto_icon, String::new(), String::new(), true)
        };

    if app_name.is_empty() || app_url.is_empty() {
        eprintln!("You must set app name and app URL!");
        return 1;
    }
    if app_name.contains('/') {
        eprintln!("App name cannot contain '/'");
        return 1;
    }
    if let Err(e) = desktop::require_http_url(&app_url) {
        eprintln!("Error: {e}");
        return 1;
    }

    let icon_value = resolve_icon(&app_name, &icon_ref, &icon_dir, &app_url, interactive);

    let exec_command = if !custom_exec_s.is_empty() {
        custom_exec_s.clone()
    } else {
        format!("omarchy-launch-webapp {}", desktop::desktop_exec_arg(&app_url))
    };

    let _ = fs::create_dir_all(&app_dir);
    let desktop_file = app_dir.join(format!("{app_name}.desktop"));

    let name_field = desktop::desktop_string_escape(&app_name);
    let exec_field = desktop::desktop_string_escape(&exec_command);
    let icon_field = desktop::desktop_string_escape(&icon_value);

    let mut content = format!(
        "[Desktop Entry]\nVersion=1.0\nName={name_field}\nComment={name_field}\n\
         Exec={exec_field}\nTerminal=false\nType=Application\n\
         Icon={icon_field}\nStartupNotify=true\n"
    );
    if !mime_types_s.is_empty() {
        content.push_str(&format!("MimeType={}\n", desktop::desktop_string_escape(&mime_types_s)));
    }

    if let Err(e) = fs::write(&desktop_file, &content) {
        eprintln!("Failed to write desktop file: {e}");
        return 1;
    }

    if interactive {
        println!("You can now find {app_name} using the app launcher (SUPER + SPACE)\n");
    }

    0
}

fn resolve_icon(
    app_name: &str,
    icon_ref: &str,
    icon_dir: &std::path::Path,
    site_url: &str,
    was_interactive: bool,
) -> String {
    let lower = icon_ref.to_lowercase();
    if icon_ref.is_empty() {
        // No icon specified — try auto-fetch
        let icon_name = desktop::safe_icon_name(app_name);
        let dest = icon_dir.join(format!("{icon_name}.png"));
        let _ = fs::create_dir_all(icon_dir);
        if desktop::fetch_site_icon(site_url, &dest) {
            desktop::update_icon_cache(icon_dir);
        }
        icon_name
    } else if lower.starts_with("http://") || lower.starts_with("https://") {
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
    } else if was_interactive && (lower.starts_with("http://") || lower.starts_with("https://")) {
        // Already handled above, but keep the arm for clarity
        icon_ref.to_owned()
    } else {
        desktop::icon_name_from_ref(icon_ref)
    }
}
