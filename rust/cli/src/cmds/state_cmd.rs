use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run(action: &str, name: &str) -> i32 {
    // Validate name: no slashes, not . or ..
    if name.contains('/') || name == "." || name == ".." {
        eprintln!("Invalid state name: {name}");
        return 1;
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let state_dir = format!("{home}/.local/state/omarchy");

    match action {
        "set" => {
            let path = format!("{state_dir}/{name}");
            if let Some(parent) = Path::new(&path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            match fs::write(&path, "") {
                Ok(_) => 0,
                Err(e) => { eprintln!("Cannot set state {name}: {e}"); 1 }
            }
        }
        "clear" => {
            // Use find with the name as a pattern (supports glob via find -name)
            let status = Command::new("find")
                .args([&state_dir, "-name", name, "-delete"])
                .status();
            if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
        }
        other => {
            eprintln!("Unknown action: {other}. Use set or clear.");
            1
        }
    }
}
