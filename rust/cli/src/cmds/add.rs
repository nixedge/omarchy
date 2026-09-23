use crate::{filter, output, socket, theme};
use omarchy_lib::alias::{self, ResolveError};
use omarchy_lib::protocol::Request;

pub fn run(name: &str, sync: bool) -> i32 {
    let p = theme::Palette::load();

    match alias::resolve(name) {
        Ok(_) => {}
        Err(ResolveError::ServiceManaged(opt)) => {
            output::err(&p, &format!("'{name}' is managed by NixOS option `{opt}`; use omarchy-setup to toggle it"));
            return 1;
        }
        Err(ResolveError::Eliminated(reason)) => {
            output::err(&p, &format!("'{name}' is not available on NixOS: {reason}"));
            return 1;
        }
    };

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    if sync {
        output::status(&p, &format!("{} Installing {name}\u{2026}", output::GLYPH_PKG));
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
    } else {
        let resp = match client.send_recv(Request::PkgAddAsync { name: name.to_owned() }) {
            Ok(r) => r,
            Err(e) => {
                output::err(&p, &format!("daemon error: {e}"));
                return 1;
            }
        };
        if resp.ok {
            output::muted(&p, &format!("{} Installing {name} in background\u{2026}", output::GLYPH_PKG));
        } else {
            output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
            return 1;
        }
    }

    0
}
