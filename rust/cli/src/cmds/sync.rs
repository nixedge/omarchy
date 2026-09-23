use crate::{output, socket, theme};
use omarchy_lib::protocol::Request;

pub fn run() -> i32 {
    let p = theme::Palette::load();

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    let resp = match client.send_recv(Request::PkgSync) {
        Ok(r) => r,
        Err(e) => {
            output::err(&p, &format!("daemon error: {e}"));
            return 1;
        }
    };

    if resp.ok {
        output::muted(&p, &format!("{} System sync queued in background\u{2026}", output::GLYPH_PKG));
        0
    } else {
        output::err(&p, resp.err_msg());
        1
    }
}
