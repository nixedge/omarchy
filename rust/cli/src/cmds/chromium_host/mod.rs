use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_default()
}

fn read_message(reader: &mut impl Read) -> Option<String> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).ok()?;
    let length = u32::from_le_bytes(len_buf) as usize;
    if length == 0 {
        return None;
    }
    let mut payload = vec![0u8; length];
    reader.read_exact(&mut payload).ok()?;
    String::from_utf8(payload).ok()
}

fn write_message(writer: &mut impl Write, msg: &[u8]) {
    let len = msg.len() as u32;
    let _ = writer.write_all(&len.to_le_bytes());
    let _ = writer.write_all(msg);
    let _ = writer.flush();
}

fn extract_url(json: &str) -> Option<String> {
    // Simple extraction of .url field from JSON
    // Use jq if available, otherwise simple parse
    let out = Command::new("jq")
        .args(["-r", ".url // empty"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()
        .and_then(|mut c| {
            use std::io::Write;
            if let Some(ref mut stdin) = c.stdin {
                let _ = stdin.write_all(json.as_bytes());
            }
            c.wait_with_output().ok()
        })
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    out
}

// ─── omarchy-chromium-copy-url-host ──────────────────────────────────────────

pub fn copy_url_host() -> i32 {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    let payload = match read_message(&mut stdin_lock) {
        Some(p) => p,
        None => return 0,
    };

    let url = extract_url(&payload);
    let copied = if let Some(ref url_str) = url {
        if !url_str.is_empty() {
            // wl-copy
            let mut wl = Command::new("wl-copy")
                .args(["--type", "text/plain"])
                .stdin(Stdio::piped())
                .spawn();
            let ok = if let Ok(ref mut c) = wl {
                if let Some(ref mut stdin) = c.stdin {
                    let _ = stdin.write_all(url_str.as_bytes());
                }
                c.wait().map(|s| s.success()).unwrap_or(false)
            } else {
                false
            };
            if ok {
                let _ = Command::new("omarchy-notification-send")
                    .args(["-g", "\u{f0154}", "URL copied to clipboard"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            }
            ok
        } else {
            false
        }
    } else {
        false
    };

    if copied {
        // {"copied":true} = 15 bytes
        write_message(&mut stdout_lock, b"{\"copied\":true}");
    } else {
        // {"copied":false} = 16 bytes
        write_message(&mut stdout_lock, b"{\"copied\":false}");
    }

    0
}

// ─── omarchy-chromium-ytdlp-host ─────────────────────────────────────────────

pub fn ytdlp_host(args: &[String]) -> i32 {
    // Check for --download internal dispatch
    if args.first().map(|s| s.as_str()) == Some("--download") {
        if let Some(url) = args.get(1) {
            return ytdlp_download(url);
        }
        return 1;
    }

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    let payload = match read_message(&mut stdin_lock) {
        Some(p) => p,
        None => return 0,
    };

    // Ack with empty message immediately
    write_message(&mut stdout_lock, b"{}");
    drop(stdout_lock);

    let url = match extract_url(&payload) {
        Some(u) if u.starts_with("http://") || u.starts_with("https://") => u,
        _ => return 0,
    };

    // Get current executable path
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => {
            // Fall back to the bash script
            let omarchy_path = omarchy_path();
            let script = format!("{}/bin/omarchy-chromium-ytdlp-host", omarchy_path);
            let err = Command::new(&script)
                .args(["--download", &url])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            if err.is_ok() {
                return 0;
            }
            return 1;
        }
    };

    // Detach download via setsid
    let _ = Command::new("setsid")
        .arg("-f")
        .arg(&exe)
        .args(["chromium-ytdlp-host", "--download", &url])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    0
}

fn ytdlp_download(url: &str) -> i32 {
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let download_dir = std::env::var("OMARCHY_YTDLP_DIR")
        .unwrap_or_else(|_| format!("{}/Videos", home_dir));

    let _ = std::fs::create_dir_all(&download_dir);

    // First simulate to check if URL has video
    let simulate = Command::new("yt-dlp")
        .args([
            "--no-playlist", "--simulate", "--quiet",
            "--no-warnings", "--no-exec", "--no-exec-before-download",
            "--", url,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if !simulate.map(|s| s.success()).unwrap_or(false) {
        let _ = Command::new("omarchy-notification-send")
            .args(["-u", "critical", "-g", "\u{f0156}", "No video found for download", url])
            .status();
        return 0;
    }

    // Show initial OSD progress
    let _ = Command::new("omarchy-osd")
        .args(["-i", "\u{f001a}", "-p", "0", "-d", "8000"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    // Download with progress tracking
    let mut filepath = String::new();
    let mut title = String::new();

    let output = Command::new("yt-dlp")
        .args([
            "--no-playlist", "--no-simulate",
            "--quiet", "--no-warnings", "--no-exec", "--no-exec-before-download",
            "--progress", "--newline",
            "--progress-template", "download:OMARCHY_PROG\t%(progress._percent_str)s",
            "--paths", &download_dir,
            "-o", "%(title)s.%(ext)s",
            "--print", "after_move:OMARCHY_FILE\t%(filepath)s",
            "--print", "after_move:OMARCHY_TITLE\t%(title)j",
            "--", url,
        ])
        .env("PYTHONUNBUFFERED", "1")
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let combined = String::from_utf8_lossy(&o.stdout).to_string()
                + &String::from_utf8_lossy(&o.stderr);
            let mut last_pct = String::new();

            for line in combined.lines() {
                if let Some(pct) = line.strip_prefix("OMARCHY_PROG\t") {
                    let pct_clean = pct.trim_end_matches('%').trim().to_string();
                    if !pct_clean.is_empty() && pct_clean != last_pct {
                        last_pct = pct_clean.clone();
                        if let Ok(n) = pct_clean.parse::<f64>() {
                            let _ = Command::new("omarchy-osd")
                                .args(["-i", "\u{f001a}", "-p", &n.round().to_string(), "-d", "8000"])
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .spawn();
                        }
                    }
                } else if let Some(f) = line.strip_prefix("OMARCHY_FILE\t") {
                    filepath = f.trim().to_string();
                } else if let Some(t) = line.strip_prefix("OMARCHY_TITLE\t") {
                    // Decode JSON-encoded title
                    let decoded = Command::new("jq")
                        .args(["-r", "if type == \"string\" then . else empty end"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                        .ok()
                        .and_then(|mut c| {
                            use std::io::Write;
                            if let Some(ref mut stdin) = c.stdin {
                                let _ = stdin.write_all(t.as_bytes());
                            }
                            c.wait_with_output().ok()
                        })
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    if !decoded.is_empty() && !decoded.starts_with('-') {
                        title = decoded;
                    }
                }
            }
        }
        Err(_) => {}
    }

    // Close OSD
    let _ = Command::new("omarchy-shell")
        .args(["-q", "osd", "close"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    if !filepath.is_empty() && std::path::Path::new(&filepath).exists() {
        if title.is_empty() {
            let name = std::path::Path::new(&filepath)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Video".to_string());
            title = name;
        }
        if title.len() > 50 {
            title.truncate(50);
            title.push('…');
        }

        // Generate preview thumbnail
        let preview = std::env::temp_dir().join(format!("ytdlp-preview-{}.jpg", std::process::id()));
        let _ = Command::new("ffmpeg")
            .args([
                "-y", "-i", &filepath,
                "-ss", "00:00:00.1", "-vframes", "1",
                "-vf", "crop='min(iw,ih)':'min(iw,ih)',scale=256:256",
                "-q:v", "2",
                preview.to_str().unwrap_or("/dev/null"),
                "-loglevel", "quiet",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        let preview_str = if preview.exists() {
            preview.to_string_lossy().to_string()
        } else {
            filepath.clone()
        };

        let _ = Command::new("omarchy-notification-send")
            .args([
                "-g", "\u{f0114}",
                "Download complete", &title,
                "-t", "10000",
                "--image", &preview_str,
                "--exec", "mpv", "--", &filepath,
            ])
            .status();

        // Clean up preview after a moment
        if preview.exists() {
            let preview_clone = preview.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(2));
                let _ = std::fs::remove_file(preview_clone);
            });
        }
    } else {
        let _ = Command::new("omarchy-notification-send")
            .args(["-u", "critical", "-g", "\u{f0156}", "Download failed", url])
            .status();
    }

    0
}
