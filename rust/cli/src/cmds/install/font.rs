use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;
use std::process::Command;

pub fn install(name: &str, package: &str, family: &str) -> i32 {
    let p = theme::Palette::load();
    output::status(&p, &format!("{} Installing {name}\u{2026}", output::GLYPH_PKG));

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    let mut fs = filter::FilterState::default();
    let resp = match client.stream(Request::FontAdd { name: package.to_owned(), enable_family: Some(family.to_owned()) }, |line| {
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

    if !resp.ok {
        output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
        return 1;
    }

    let _ = Command::new("omarchy-font-set").arg(family).status();

    output::ok(&p, &format!("{} {name} installed", output::GLYPH_OK));
    0
}
