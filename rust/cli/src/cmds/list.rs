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

    let resp = match client.send_recv(Request::PkgList) {
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

    let pkgs = match resp.data.as_ref().and_then(|d| d.as_array()) {
        Some(a) => a,
        None => {
            output::err(&p, "unexpected response format");
            return 1;
        }
    };

    if pkgs.is_empty() {
        println!("{}No packages installed{}", p.muted, p.reset);
        return 0;
    }

    let mut sorted = pkgs.to_vec();
    sorted.sort_by(|a, b| {
        let an = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let bn = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
        an.cmp(bn)
    });

    for pkg in &sorted {
        let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("");
        println!("{}", name);
    }

    0
}
