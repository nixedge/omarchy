pub mod ai;
pub mod and_launch;
pub mod app;
pub mod browser;
pub mod chromium;
pub mod devenv;
pub mod docker_dbs;
pub mod editor;
pub mod font;
pub mod gaming;
pub mod hermes_cli;
pub mod openclaw_cli;
pub mod preinstalls;
pub mod service_once;
pub mod terminal;
pub mod tui;
pub mod voxtype;
pub mod webapp;

use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;

/// Install multiple packages in a single rebuild.
pub fn add_many(names: &[&str], sync: bool) -> i32 {
    let p = theme::Palette::load();

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    let names_owned: Vec<String> = names.iter().map(|s| s.to_string()).collect();

    if sync {
        output::status(&p, &format!("{} Installing {} packages\u{2026}", output::GLYPH_PKG, names.len()));
        let mut fs = filter::FilterState::default();
        let resp = match client.stream(Request::PkgAddMany { names: names_owned }, |line| {
            if let Some(out) = filter::filter(line, &mut fs, &p) {
                println!("{out}");
            }
        }) {
            Ok(r) => r,
            Err(e) => {
                output::err(&p, &format!("daemon error: {e}"));
                return 1;
            }
        };
        if resp.ok {
            output::ok(&p, &format!("{} Packages installed", output::GLYPH_OK));
        } else {
            output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
            return 1;
        }
    } else {
        output::muted(&p, &format!("{} Installing {} packages in background\u{2026}", output::GLYPH_PKG, names.len()));
    }

    0
}

/// Remove multiple packages in a single rebuild.
pub fn remove_many(names: &[&str]) -> i32 {
    let p = theme::Palette::load();

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    let names_owned: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    output::status(&p, &format!("{} Removing {} packages\u{2026}", output::GLYPH_PKG, names.len()));
    let mut fs = filter::FilterState::default();
    let resp = match client.stream(Request::PkgRemoveMany { names: names_owned }, |line| {
        if let Some(out) = filter::filter(line, &mut fs, &p) {
            println!("{out}");
        }
    }) {
        Ok(r) => r,
        Err(e) => {
            output::err(&p, &format!("daemon error: {e}"));
            return 1;
        }
    };
    if resp.ok {
        output::ok(&p, &format!("{} Packages removed", output::GLYPH_OK));
    } else {
        output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
        return 1;
    }

    0
}

/// Spawn a program detached from the terminal.
pub fn launch_detached(program: &str, args: &[&str]) {
    let _ = std::process::Command::new("setsid")
        .arg("uwsm-app")
        .arg("--")
        .arg(program)
        .args(args)
        .spawn();
}

pub fn launch_desktop(desktop_id: &str) {
    let _ = std::process::Command::new("setsid")
        .args(["uwsm-app", "--", "gtk-launch", desktop_id])
        .spawn();
}

pub fn home_path(rel: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::PathBuf::from(home).join(rel)
}
