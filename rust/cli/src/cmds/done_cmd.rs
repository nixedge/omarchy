use std::fs;
use std::path::Path;
use std::io::{self, Read, Write};

pub fn done(action: &str, name: &str) -> i32 {
    if name.contains('/') || name == "." || name == ".." {
        eprintln!("Invalid done marker name: {name}");
        return 1;
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let done_dir = format!("{home}/.local/state/omarchy/done");
    let path = format!("{done_dir}/{name}");

    match action {
        "check" => {
            if Path::new(&path).exists() { 0 } else { 1 }
        }
        "mark" => {
            let _ = fs::create_dir_all(&done_dir);
            match fs::write(&path, "") {
                Ok(_) => 0,
                Err(e) => { eprintln!("Cannot mark done {name}: {e}"); 1 }
            }
        }
        "ensure" => {
            // Like bash noclobber: return 0 only on the first call (when the
            // marker did not exist yet). Return 1 if already marked, so callers
            // can gate one-time setup with: if omarchy-done ensure task; then ...
            if Path::new(&path).exists() {
                return 1;
            }
            let _ = fs::create_dir_all(&done_dir);
            match fs::OpenOptions::new().create_new(true).write(true).open(&path) {
                Ok(_) => 0,
                Err(_) => 1, // lost race or already existed
            }
        }
        other => {
            eprintln!("Unknown action: {other}. Use check, mark, or ensure.");
            1
        }
    }
}

pub fn show_done(exit_code: Option<i32>) -> i32 {
    let code = exit_code.unwrap_or(0);
    let tty = fs::OpenOptions::new().write(true).open("/dev/tty");
    if let Ok(mut tty_file) = tty {
        if code == 0 {
            let _ = tty_file.write_all(b"\x1b[32mDone!\x1b[0m\n");
        } else {
            let _ = tty_file.write_all(b"\x1b[31mFailed!\x1b[0m\n");
        }
        // Wait for a keypress
        let _ = tty_file.write_all(b"Press any key to continue...");
        let _ = tty_file.flush();
        let stdin = fs::File::open("/dev/tty");
        if let Ok(mut stdin_file) = stdin {
            let mut buf = [0u8; 1];
            let _ = stdin_file.read(&mut buf);
        }
    }
    0
}

pub fn show_logo() -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let logo_path = format!("{omarchy_path}/logo.txt");
    match fs::read_to_string(&logo_path) {
        Ok(content) => {
            print!("\x1b[32m{content}\x1b[0m");
            0
        }
        Err(e) => {
            eprintln!("Cannot read logo: {e}");
            1
        }
    }
}
