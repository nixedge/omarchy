use crate::{output, socket, theme};
use omarchy_lib::protocol::Request;

pub fn run(name: &str) -> i32 {
    let p = theme::Palette::load();

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    let resp = match client.send_recv(Request::PkgPresent { name: name.to_owned() }) {
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

    let present = resp
        .data
        .as_ref()
        .and_then(|d| d.get("present"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if present { 0 } else { 1 }
}
