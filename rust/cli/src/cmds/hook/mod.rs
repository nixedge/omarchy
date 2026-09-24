use std::process::Command;

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

// ─── hook ─────────────────────────────────────────────────────────────────────

pub fn run(hook_name: &str, args: &[String]) -> i32 {
    // Validate hook name
    if hook_name.is_empty() || hook_name.contains('/') || hook_name == "." || hook_name == ".." {
        eprintln!("Invalid hook name: {}", hook_name);
        return 2;
    }

    let home_dir = home();
    let hook_path = format!("{}/.config/omarchy/hooks/{}", home_dir, hook_name);
    let hook_dir = format!("{}.d", hook_path);

    // Run the main hook file if it exists
    if std::path::Path::new(&hook_path).is_file() {
        let status = Command::new("bash")
            .arg(&hook_path)
            .args(args)
            .status();
        if !status.map(|s| s.success()).unwrap_or(false) {
            eprintln!("Hook failed: {}", hook_path);
        }
    }

    // Run all files in the hook.d directory
    if std::path::Path::new(&hook_dir).is_dir() {
        let entries = match std::fs::read_dir(&hook_dir) {
            Ok(e) => e,
            Err(_) => return 0,
        };

        let mut hooks: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().is_file())
            .filter(|e| {
                !e.file_name()
                    .to_string_lossy()
                    .ends_with(".sample")
            })
            .collect();

        // Sort for deterministic order
        hooks.sort_by_key(|e| e.file_name());

        for entry in hooks {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            let status = Command::new("bash")
                .arg(&path_str)
                .args(args)
                .status();
            if !status.map(|s| s.success()).unwrap_or(false) {
                eprintln!("Hook failed: {}", path_str);
            }
        }
    }

    0
}

// ─── hook install ─────────────────────────────────────────────────────────────

pub fn install(hook_type: &str, hook_file: &str) -> i32 {
    // Validate hook type
    if hook_type.is_empty() || hook_type.contains('/') || hook_type == "." || hook_type == ".." {
        eprintln!("Invalid hook name: {}", hook_type);
        return 2;
    }

    if !std::path::Path::new(hook_file).is_file() {
        eprintln!("Hook file not found: {}", hook_file);
        return 1;
    }

    let home_dir = home();
    let hook_dir = format!("{}/.config/omarchy/hooks/{}.d", home_dir, hook_type);

    let hook_name = std::path::Path::new(hook_file)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "hook".to_string());

    let hook_path = format!("{}/{}", hook_dir, hook_name);

    if let Err(e) = std::fs::create_dir_all(&hook_dir) {
        eprintln!("Failed to create hooks directory: {}", e);
        return 1;
    }

    if let Err(e) = std::fs::copy(hook_file, &hook_path) {
        eprintln!("Failed to copy hook: {}", e);
        return 1;
    }

    // chmod 755
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(&hook_path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        let _ = std::fs::set_permissions(&hook_path, perms);
    }

    println!("Installed {} hook: {}", hook_type, hook_path);
    0
}
