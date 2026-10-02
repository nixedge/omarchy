use std::process::Command;

use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;

// ─── font current ──────────────────────────────────────────────────────────

pub fn current() -> i32 {
    let out = Command::new("fc-match")
        .args(["monospace", "-f", "%{family}\\n"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    if let Some(first_line) = out.lines().next() {
        let family = first_line.split(',').next().unwrap_or(first_line);
        println!("{family}");
        0
    } else {
        1
    }
}

// ─── font list ─────────────────────────────────────────────────────────────

pub fn list() -> i32 {
    let out = Command::new("fc-list")
        .args([":spacing=100", "-f", "%{family[0]}\\n"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut fonts: Vec<String> = out
        .lines()
        .filter(|l| {
            let lower = l.to_lowercase();
            !lower.contains("emoji") && !lower.contains("signwriting") && !lower.contains("omarchy")
        })
        .map(|l| l.to_string())
        .collect();

    fonts.sort();
    fonts.dedup();

    for font in fonts {
        println!("{font}");
    }
    0
}

// ─── font set ──────────────────────────────────────────────────────────────

pub fn set_font(font_name: &str) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{omarchy_path}/bin/omarchy-font-set");
    let err = Command::new(&script).arg(font_name).exec();
    eprintln!("exec {script}: {err}");
    1
}

// ─── font enable ───────────────────────────────────────────────────────────

pub fn enable(family: &str) -> i32 {
    let p = theme::Palette::load();
    output::status(&p, &format!("Enabling '{family}' as system font\u{2026}"));

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    let mut fs_state = filter::FilterState::default();
    let resp = match client.stream(Request::FontEnable { family: family.to_owned() }, |line| {
        if let Some(out) = filter::filter(line, &mut fs_state, &p) {
            println!("{out}");
        }
    }) {
        Ok(r) => r,
        Err(e) => {
            output::err(&p, &format!("daemon error: {e}"));
            return 1;
        }
    };

    if !resp.ok {
        output::err(&p, resp.err_msg());
        return 1;
    }

    let _ = Command::new("omarchy-font-set").arg(family).status();

    output::ok(&p, &format!("'{family}' enabled as system default"));
    0
}
