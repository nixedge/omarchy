use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

pub fn user_app_dir() -> PathBuf {
    home_dir().join(".local/share/applications")
}

pub fn user_icon_dir() -> PathBuf {
    home_dir().join(".local/share/icons/hicolor/256x256/apps")
}

/// Slug-ify a display name for use as an icon filename.
pub fn safe_icon_name(name: &str) -> String {
    let lower = name.to_lowercase();
    let slugged: String = lower
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = slugged.trim_matches('-');
    // collapse runs of '-'
    let mut prev_dash = false;
    let mut out = String::new();
    for c in trimmed.chars() {
        if c == '-' {
            if !prev_dash {
                out.push(c);
            }
            prev_dash = true;
        } else {
            out.push(c);
            prev_dash = false;
        }
    }
    out
}

/// Desktop Entry "string" value escaping (freedesktop spec).
pub fn desktop_string_escape(value: &str) -> String {
    let mut v = value.replace('\\', "\\\\");
    v = v.replace('\t', "\\t");
    v = v.replace('\r', "\\r");
    v = v.replace('\n', "\\n");
    if v.starts_with(' ') {
        v = format!("\\s{}", &v[1..]);
    }
    v
}

/// Escape a URL/argument for use inside a double-quoted Exec value.
pub fn desktop_exec_arg(url: &str) -> String {
    let escaped = url
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('`', "\\`")
        .replace('$', "\\$")
        .replace('%', "%%");
    format!("\"{escaped}\"")
}

/// Icon name derived from a file reference (strips path and extension).
pub fn icon_name_from_ref(icon_ref: &str) -> String {
    let base = Path::new(icon_ref)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(icon_ref);
    if base.contains('.') {
        let stem = Path::new(base)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(base);
        safe_icon_name(stem)
    } else {
        base.to_owned()
    }
}

/// Download a URL to dest using curl. Returns true on success.
pub fn download_icon(url: &str, dest: &Path) -> bool {
    let status = Command::new("curl")
        .args(["-fsSL", "--max-time", "10", "-o", dest.to_str().unwrap_or(""), url])
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        return false;
    }
    // Verify it's actually an image (file -b --mime-type)
    let out = Command::new("file")
        .args(["-b", "--mime-type", dest.to_str().unwrap_or("")])
        .output()
        .ok();
    out.map(|o| String::from_utf8_lossy(&o.stdout).trim().starts_with("image/"))
        .unwrap_or(false)
}

/// Copy a local file as an icon, preserving extension.
pub fn install_local_icon(src: &Path, icon_name: &str, icon_dir: &Path) -> Option<String> {
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let dest = icon_dir.join(format!("{icon_name}.{ext}"));
    std::fs::create_dir_all(icon_dir).ok()?;
    std::fs::copy(src, &dest).ok()?;
    update_icon_cache(icon_dir);
    Some(icon_name.to_owned())
}

/// Try to fetch the site's touch icon; falls back to Google favicon service.
pub fn fetch_site_icon(site_url: &str, dest: &Path) -> bool {
    let origin: String = {
        let trimmed = site_url.trim_start_matches("https://").trim_start_matches("http://");
        let host = trimmed.split('/').next().unwrap_or(trimmed);
        // Reconstruct with scheme
        if site_url.starts_with("http://") {
            format!("http://{host}")
        } else {
            format!("https://{host}")
        }
    };

    // Try to get page content and look for apple-touch-icon
    let page_out = Command::new("curl")
        .args(["-fsSL", "--max-time", "5", site_url])
        .output();

    if let Ok(page) = page_out {
        let html = String::from_utf8_lossy(&page.stdout);
        let html_flat: String = html.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
        // Simple grep for apple-touch-icon href
        if let Some(icon_url) = extract_touch_icon_url(&html_flat, &origin) {
            if download_icon(&icon_url, dest) {
                return true;
            }
        }
    }

    // Well-known path
    if download_icon(&format!("{origin}/apple-touch-icon.png"), dest) {
        return true;
    }

    // Google favicon service
    download_icon(
        &format!("https://www.google.com/s2/favicons?domain={site_url}&sz=256"),
        dest,
    )
}

fn extract_touch_icon_url(html: &str, origin: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let pat = "apple-touch-icon";
    let pos = lower.find(pat)?;
    let tag = &html[pos.saturating_sub(200)..];
    // Find href= after the pattern
    let href_pos = tag.find("href=")?;
    let after = &tag[href_pos + 5..];
    let (quote, rest) = if after.starts_with('"') {
        ('"', &after[1..])
    } else if after.starts_with('\'') {
        ('\'', &after[1..])
    } else {
        return None;
    };
    let end = rest.find(quote)?;
    let raw = &rest[..end];
    let url = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_owned()
    } else if raw.starts_with("//") {
        format!("https:{raw}")
    } else if raw.starts_with('/') {
        format!("{origin}{raw}")
    } else {
        format!("{origin}/{raw}")
    };
    Some(url)
}

pub fn update_icon_cache(icon_dir: &Path) {
    // Walk up to the hicolor base dir
    let hicolor = icon_dir.ancestors().find(|p| {
        p.file_name().and_then(|n| n.to_str()) == Some("hicolor")
    });
    if let Some(base) = hicolor {
        let _ = Command::new("gtk-update-icon-cache")
            .arg(base)
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status();
    }
}

/// Run `gum confirm` — returns true if the user confirmed.
pub fn gum_confirm(prompt: &str) -> bool {
    Command::new("gum")
        .args(["confirm", prompt])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run `gum input` — returns the entered text, or None if cancelled.
pub fn gum_input(prompt: &str, placeholder: &str) -> Option<String> {
    let out = Command::new("gum")
        .args(["input", "--prompt", prompt, "--placeholder", placeholder])
        .output()
        .ok()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        if s.is_empty() { None } else { Some(s) }
    } else {
        None
    }
}

/// Run `gum choose` with the given options piped on stdin.
pub fn gum_choose(options: &[&str], header: &str) -> Option<Vec<String>> {
    use std::io::Write;
    let mut child = Command::new("gum")
        .args(["choose", "--header", header])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(options.join("\n").as_bytes());
    }
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    let chosen: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_owned())
        .filter(|l| !l.is_empty())
        .collect();
    if chosen.is_empty() { None } else { Some(chosen) }
}

/// Normalize a URL — add https:// prefix if schemeless.
pub fn normalize_url(url: &str) -> String {
    if url.contains("://") {
        url.to_owned()
    } else {
        format!("https://{url}")
    }
}

/// Validate that a URL is http or https with no whitespace.
pub fn require_http_url(url: &str) -> Result<(), String> {
    if url.chars().any(|c| c.is_whitespace()) {
        return Err("web app URL must not contain whitespace".to_owned());
    }
    let lower = url.to_lowercase();
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        return Err("web app URL must be http or https".to_owned());
    }
    Ok(())
}
