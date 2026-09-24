use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn history_path() -> String {
    format!("{}/.local/state/omarchy/clipboard-history.json", home())
}

// ─── clipboard open ───────────────────────────────────────────────────────────

pub fn open(history_index: u64) -> i32 {
    use std::os::unix::process::CommandExt;

    let path = history_path();

    // Validate index and get entry type via jq
    let entry_type = Command::new("jq")
        .args([
            "-er",
            &format!(".[{}].type", history_index),
            &path,
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    match entry_type.as_str() {
        "image" => {
            let img_path = Command::new("jq")
                .args([
                    "-er",
                    &format!(".[{}].path", history_index),
                    &path,
                ])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            if img_path.is_empty() || !std::path::Path::new(&img_path).exists() {
                return 1;
            }
            let err = Command::new("omasnap").arg(&img_path).exec();
            eprintln!("exec omasnap: {}", err);
            1
        }
        "text" => {
            let text = Command::new("jq")
                .args([
                    "-er",
                    &format!(".[{}].text", history_index),
                    &path,
                ])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim_end_matches('\n').to_string())
                .unwrap_or_default();

            if text.is_empty() {
                return 1;
            }

            // Check if it's a URL
            let url_re = Command::new("grep")
                .args(["-Eom1", "https?://[^[:space:]\"'<>]+"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .ok()
                .and_then(|mut c| {
                    if let Some(ref mut stdin) = c.stdin {
                        let _ = write!(stdin, "{}", text);
                    }
                    c.wait_with_output().ok()
                })
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            if !url_re.is_empty() {
                let err = Command::new("omarchy-launch-browser").arg(&url_re).exec();
                eprintln!("exec omarchy-launch-browser: {}", err);
                return 1;
            }

            // Write to temp file and open in editor
            let open_dir = format!(
                "{}/omarchy/clipboard-open",
                std::env::var("XDG_STATE_HOME")
                    .unwrap_or_else(|_| format!("{}/.local/state", home()))
            );
            let _ = std::fs::create_dir_all(&open_dir);

            let tmp = format!("{}/clipboard.{}.txt", open_dir, std::process::id());
            if std::fs::write(&tmp, &text).is_err() {
                return 1;
            }

            let err = Command::new("omarchy-launch-editor").arg(&tmp).exec();
            eprintln!("exec omarchy-launch-editor: {}", err);
            1
        }
        _ => 1,
    }
}

// ─── clipboard paste-file ─────────────────────────────────────────────────────

pub fn paste_file(copy_only: bool, mime_type: &str, path: &str) -> i32 {
    if !std::path::Path::new(path).exists() {
        eprintln!("File not readable: {}", path);
        return 1;
    }

    // wl-copy --type "$mime" < "$path"
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Cannot open {}: {}", path, e);
            return 1;
        }
    };

    let status = Command::new("wl-copy")
        .args(["--type", mime_type])
        .stdin(file)
        .status();

    if !status.map(|s| s.success()).unwrap_or(false) {
        return 1;
    }

    if copy_only {
        return 0;
    }

    thread::sleep(Duration::from_millis(150));

    let _ = Command::new("wtype")
        .args(["-M", "shift", "-k", "Insert", "-m", "shift"])
        .stderr(Stdio::null())
        .status();

    0
}

// ─── clipboard paste-text ─────────────────────────────────────────────────────

pub fn paste_text(
    shift_insert: bool,
    copy_only: bool,
    history_index: Option<u64>,
    text_args: &[String],
) -> i32 {
    let mut use_shift_insert = shift_insert;

    if let Some(idx) = history_index {
        // Copy from history
        let hist_path = history_path();
        let ok = Command::new("jq")
            .args([
                "-e",
                &format!(
                    ".[{}].type == \"text\" and (.[{}].text | type == \"string\")",
                    idx, idx
                ),
                &hist_path,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !ok {
            return 0;
        }

        // jq -j to get raw without trailing newline
        let wl_ok = Command::new("jq")
            .args([
                "-j",
                &format!(".[{}].text", idx),
                &hist_path,
            ])
            .stdout(Stdio::piped())
            .spawn()
            .ok()
            .and_then(|mut c| {
                let stdout = c.stdout.take()?;
                let status = Command::new("wl-copy")
                    .stdin(stdout)
                    .status()
                    .ok()?;
                let _ = c.wait();
                Some(status.success())
            })
            .unwrap_or(false);

        if !wl_ok {
            return 0;
        }

        if !copy_only {
            use_shift_insert = true;
        }
    } else {
        let text = text_args.join(" ");
        if text.is_empty() {
            return 0;
        }

        let mut wl = match Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
            Ok(c) => c,
            Err(_) => return 1,
        };
        if let Some(ref mut stdin) = wl.stdin {
            let _ = write!(stdin, "{}", text);
        }
        let _ = wl.wait();

        if copy_only {
            return 0;
        }

        thread::sleep(Duration::from_millis(150));

        if use_shift_insert {
            let _ = Command::new("wtype")
                .args(["-M", "shift", "-k", "Insert", "-m", "shift"])
                .stderr(Stdio::null())
                .status();
        } else {
            let _ = Command::new("wtype")
                .arg(&text)
                .stderr(Stdio::null())
                .status();
        }
        return 0;
    }

    if copy_only {
        return 0;
    }

    thread::sleep(Duration::from_millis(150));

    if use_shift_insert {
        let _ = Command::new("wtype")
            .args(["-M", "shift", "-k", "Insert", "-m", "shift"])
            .stderr(Stdio::null())
            .status();
    }

    0
}
