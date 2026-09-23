use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;

pub fn run(name: &str) -> i32 {
    let p = theme::Palette::load();

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => { output::err(&p, &format!("daemon not reachable: {e}")); return 1; }
    };

    output::status(&p, &format!("{} Disabling service '{name}'\u{2026}", output::GLYPH_PKG));

    let mut fs = filter::FilterState::default();
    let resp = match client.stream(Request::ServiceDisable { name: name.to_owned() }, |line| {
        if let Some(out) = filter::filter(line, &mut fs, &p) {
            println!("{out}");
        }
    }) {
        Ok(r) => r,
        Err(e) => { output::err(&p, &format!("daemon error: {e}")); return 1; }
    };

    if !resp.ok {
        output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
        return 1;
    }

    output::ok(&p, &format!("{} Service '{name}' disabled", output::GLYPH_OK));
    0
}
