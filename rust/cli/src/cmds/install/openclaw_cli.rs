use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;

pub fn run(check: bool) -> i32 {
    let p = theme::Palette::load();
    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    let present = {
        let resp = match client.send_recv(Request::PkgPresent { name: "openclaw".to_owned() }) {
            Ok(r) => r,
            Err(e) => {
                output::err(&p, &format!("daemon error: {e}"));
                return 1;
            }
        };
        resp.data
            .as_ref()
            .and_then(|d| d.get("present"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };

    if check {
        return if present { 0 } else { 1 };
    }

    if present {
        return 0;
    }

    println!("Installing OpenClaw...");
    let mut fs = filter::FilterState::default();
    let resp = match client.stream(
        Request::PkgAdd { name: "openclaw".to_owned() },
        |line| {
            if let Some(out) = filter::filter(line, &mut fs, &p) {
                println!("{out}");
            }
        },
    ) {
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
    0
}
