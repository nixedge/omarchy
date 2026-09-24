use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_else(|_| "/usr/share/omarchy".to_string())
}

fn state_dir() -> PathBuf {
    std::env::var("OMARCHY_MIGRATION_STATE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(home()).join(".local/state/omarchy/migrations"))
}

fn migrations_dir() -> PathBuf {
    PathBuf::from(omarchy_path()).join("migrations")
}

struct MigrationEntry {
    name: String,
    file: PathBuf,
    marker: PathBuf,
}

fn migration_entries() -> Vec<MigrationEntry> {
    let mdir = migrations_dir();
    if !mdir.is_dir() {
        return vec![];
    }

    let mut entries: Vec<_> = fs::read_dir(&mdir)
        .unwrap_or_else(|_| fs::read_dir("/dev/null").unwrap())
        .flatten()
        .filter(|e| {
            e.file_type().map(|t| t.is_file()).unwrap_or(false)
                && e.file_name().to_string_lossy().ends_with(".sh")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let sdir = state_dir();
    entries
        .into_iter()
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let marker = sdir.join(&name);
            MigrationEntry {
                name,
                file: e.path(),
                marker,
            }
        })
        .collect()
}

fn pending_migrations() -> Vec<String> {
    migration_entries()
        .into_iter()
        .filter(|e| !e.marker.exists())
        .map(|e| e.name)
        .collect()
}

/// Run pending migrations.
pub fn run(args: &[String]) -> i32 {
    let mut mode = "run";

    for arg in args {
        match arg.as_str() {
            "--pending" | "--check" => mode = "pending",
            "-h" | "--help" => {
                println!("Usage: omarchy-migrate [--pending]");
                return 0;
            }
            _ => {
                eprintln!("Unknown option: {}", arg);
                return 1;
            }
        }
    }

    if mode == "pending" {
        let pending = pending_migrations();
        if pending.is_empty() {
            return 1;
        }
        for name in &pending {
            println!("{}", name);
        }
        return 0;
    }

    // mode == "run"
    let sdir = state_dir();
    let mdir = migrations_dir();

    if !mdir.is_dir() {
        return 0;
    }

    let _ = fs::create_dir_all(&sdir);

    let omarchy_path_val = omarchy_path();

    for entry in migration_entries() {
        if entry.marker.exists() {
            continue;
        }

        println!("\x1b[32m\nRunning migration ({})\x1b[0m", entry.name.trim_end_matches(".sh"));

        let status = Command::new("bash")
            .args(["-euo", "pipefail", "--", entry.file.to_string_lossy().as_ref()])
            .env("OMARCHY_PATH", &omarchy_path_val)
            .status();

        match status {
            Ok(s) if s.success() => {
                if let Some(parent) = entry.marker.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::File::create(&entry.marker);
            }
            Ok(s) => {
                let code = s.code().unwrap_or(1);
                eprintln!("Migration {} failed with exit code {}", entry.name, code);
                return code;
            }
            Err(e) => {
                eprintln!("Failed to run migration {}: {}", entry.name, e);
                return 1;
            }
        }
    }

    // Dismiss any pending migrations notification
    let _ = Command::new("omarchy-notification-dismiss")
        .arg("Omarchy Migrations")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    0
}

/// Notify the user when there are pending migrations.
pub fn notify() -> i32 {
    // Check if an update is in progress
    let xdg_runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
    if !xdg_runtime.is_empty() {
        let lock = format!("{}/omarchy-update.lock", xdg_runtime);
        // If lock file exists and is held, exit
        if std::path::Path::new(&lock).exists() {
            let held = Command::new("flock")
                .args(["-n", &lock, "true"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| !s.success()) // flock -n fails when lock is held
                .unwrap_or(false);
            if held {
                return 0;
            }
        }
    }

    let pending = pending_migrations();
    if pending.is_empty() {
        return 0;
    }

    let count = pending.len();
    let message = if count == 1 {
        "Click to run 1 pending migration.".to_string()
    } else {
        format!("Click to run {} pending migrations.", count)
    };

    // Wait for notification server to be ready
    let _ = Command::new("omarchy-notification-wait")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // Re-check update in progress after wait
    if !xdg_runtime.is_empty() {
        let lock = format!("{}/omarchy-update.lock", xdg_runtime);
        if std::path::Path::new(&lock).exists() {
            let held = Command::new("flock")
                .args(["-n", &lock, "true"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| !s.success())
                .unwrap_or(false);
            if held {
                return 0;
            }
        }
    }

    let status = Command::new("omarchy-notification-send")
        .args([
            "-u",
            "critical",
            "-g",
            "\u{f0139}",
            "Pending Omarchy Migrations",
            &message,
            "--exec",
            "omarchy-launch-floating-terminal-with-presentation",
            "omarchy-migrate",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if status {
        return 0;
    }

    // Fallback: print to terminal or stderr
    let pending_list: Vec<String> = pending_migrations();
    let print_pending = || {
        println!("Omarchy has pending migrations. Run omarchy-migrate in a terminal to apply them:");
        for m in &pending_list {
            if !m.is_empty() {
                println!("  {}", m);
            }
        }
    };

    // Check if stdout is a terminal
    let is_tty = unsafe { libc_isatty(1) };
    if is_tty {
        print_pending();
    } else {
        eprintln!("Omarchy has pending migrations. Run omarchy-migrate in a terminal to apply them:");
        for m in &pending_list {
            if !m.is_empty() {
                eprintln!("  {}", m);
            }
        }
    }

    0
}

fn libc_isatty(fd: i32) -> bool {
    extern "C" {
        fn isatty(fd: i32) -> i32;
    }
    unsafe { isatty(fd) != 0 }
}
