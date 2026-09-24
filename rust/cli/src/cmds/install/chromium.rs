use std::fs;
use std::process::Command;

pub fn claude() -> i32 {
    let extension_id = "fcoeoabgfenejglbffodgkkbkcdhcgfn";
    let extension_json = r#"{ "external_update_url": "https://clients2.google.com/service/update2/crx" }"#;

    let dirs = [
        "/usr/share/chromium/extensions",
        "/usr/share/google-chrome/extensions",
        "/usr/share/microsoft-edge/extensions",
    ];

    let already_installed = dirs.iter().all(|dir| {
        let path = format!("{dir}/{extension_id}.json");
        fs::read_to_string(&path).map(|c| c.trim() == extension_json).unwrap_or(false)
    });

    if already_installed {
        return 0;
    }

    // Need sudo to write to system extension dirs
    for dir in &dirs {
        let path = format!("{dir}/{extension_id}.json");
        let parent = std::path::Path::new(dir);
        if parent.exists() || sudo_mkdir(dir) {
            if sudo_write(&path, extension_json) {
                continue;
            }
            return 1;
        }
    }
    0
}

fn sudo_mkdir(dir: &str) -> bool {
    Command::new("sudo").args(["install", "-d", "-m", "0755", dir]).status()
        .map(|s| s.success()).unwrap_or(false)
}

fn sudo_write(path: &str, content: &str) -> bool {
    Command::new("sudo").args(["install", "-D", "-m", "0644", "/dev/stdin", path])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(content.as_bytes());
                let _ = stdin.write_all(b"\n");
            }
            child.wait()
        })
        .map(|s| s.success()).unwrap_or(false)
}

pub fn copy_url() -> i32 {
    install_native_host("com.omarchy.copy_url", "omarchy-chromium-copy-url-host")
}

pub fn ytdlp() -> i32 {
    install_native_host("com.omarchy.ytdlp", "omarchy-chromium-ytdlp-host")
}

fn install_native_host(host_name: &str, host_binary: &str) -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let template_path = format!("{omarchy_path}/default/chromium/native-messaging-hosts/{host_name}.json");
    let host_path = format!("{omarchy_path}/bin/{host_binary}");

    let template = match fs::read_to_string(&template_path) {
        Ok(t) => t.replace("__HOST_PATH__", &host_path),
        Err(e) => {
            eprintln!("Failed to read template {template_path}: {e}");
            return 1;
        }
    };

    let browser_dirs = [
        format!("{home}/.config/chromium"),
        format!("{home}/.config/google-chrome"),
        format!("{home}/.config/google-chrome-beta"),
        format!("{home}/.config/google-chrome-unstable"),
        format!("{home}/.config/BraveSoftware/Brave-Browser"),
        format!("{home}/.config/BraveSoftware/Brave-Browser-Beta"),
        format!("{home}/.config/BraveSoftware/Brave-Browser-Nightly"),
        format!("{home}/.config/BraveSoftware/Brave-Origin"),
        format!("{home}/.config/microsoft-edge"),
        format!("{home}/.config/microsoft-edge-dev"),
    ];

    for dir in &browser_dirs {
        let hosts_dir = format!("{dir}/NativeMessagingHosts");
        let _ = fs::create_dir_all(&hosts_dir);
        let _ = fs::write(format!("{hosts_dir}/{host_name}.json"), &template);
    }

    0
}

pub fn google_account() -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let conf = format!("{home}/.config/chromium-flags.conf");

    if !std::path::Path::new(&conf).exists() {
        return 0;
    }

    println!("Installing Chromium Google account support…");

    let content = fs::read_to_string(&conf).unwrap_or_default();
    let client_id = "--oauth2-client-id=77185425430.apps.googleusercontent.com";
    let client_secret = "--oauth2-client-secret=OTJgUOQcT7lO7GsGZq2G4IlT";

    let mut lines: Vec<String> = content.lines().map(|l| l.to_owned()).collect();
    if !lines.iter().any(|l| l == client_id) {
        lines.push(client_id.to_owned());
    }
    if !lines.iter().any(|l| l == client_secret) {
        lines.push(client_secret.to_owned());
    }

    let _ = fs::write(&conf, lines.join("\n") + "\n");
    println!("Now you can login to your Google Account in Chromium.");
    0
}
