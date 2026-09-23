use crate::{filter, output, socket, theme};
use omarchy_lib::alias::{self, ResolveError};
use omarchy_lib::protocol::Request;
use std::process::Command;

pub fn run(name: &str, sync: bool) -> i32 {
    let p = theme::Palette::load();

    let attr = match alias::resolve(name) {
        Ok(a) => a,
        Err(ResolveError::ServiceManaged(opt)) => {
            output::err(&p, &format!("'{name}' is managed by NixOS option `{opt}`; use omarchy-setup to toggle it"));
            return 1;
        }
        Err(ResolveError::Eliminated(reason)) => {
            output::err(&p, &format!("'{name}' is not available on NixOS: {reason}"));
            return 1;
        }
    };

    if sync {
        // Block until rebuild completes — no profile round-trip needed.
        let mut client = match socket::DaemonClient::connect() {
            Ok(c) => c,
            Err(e) => {
                output::err(&p, &format!("daemon not reachable: {e}"));
                return 1;
            }
        };
        output::status(&p, &format!("{} Syncing system config\u{2026}", output::GLYPH_PKG));
        let mut fs = filter::FilterState::default();
        let resp = match client.stream(Request::PkgAdd { name: name.to_owned() }, |line| {
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
            output::ok(&p, &format!("{} {name} installed", output::GLYPH_OK));
        } else {
            output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
            return 1;
        }
        return 0;
    }

    // Default: profile add for immediate availability, rebuild in background.
    let installable = format!("nixpkgs#{attr}");
    output::status(&p, &format!("{} Fetching {name}\u{2026}", output::GLYPH_PKG));
    let ok = Command::new("nix")
        .args(["--extra-experimental-features", "nix-command flakes", "profile", "add", &installable])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok {
        output::err(&p, &format!("{} package not found or fetch failed: {name}", output::GLYPH_FAIL));
        eprintln!("{}Search with: omarchy pkg search {name}{}", p.muted, p.reset);
        return 1;
    }

    output::ok(&p, &format!("{} {name} installed", output::GLYPH_OK));

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    let resp = match client.send_recv(Request::PkgAddAsync { name: name.to_owned() }) {
        Ok(r) => r,
        Err(e) => {
            output::err(&p, &format!("daemon error: {e}"));
            return 1;
        }
    };

    if resp.ok {
        output::muted(&p, &format!("{} System sync queued in background\u{2026}", output::GLYPH_PKG));
    } else {
        let _ = Command::new("nix")
            .args(["--extra-experimental-features", "nix-command flakes", "profile", "remove", &installable])
            .output();
        output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
        return 1;
    }

    0
}
