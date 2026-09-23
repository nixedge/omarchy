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

    let max_name = pkgs
        .iter()
        .filter_map(|p| p.get("name").and_then(|n| n.as_str()))
        .map(|n| n.len())
        .max()
        .unwrap_or(4);

    println!("{}{:<width$}  VERSION{}", p.accent, "NAME", p.reset, width = max_name);

    let mut sorted = pkgs.to_vec();
    sorted.sort_by(|a, b| {
        let an = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let bn = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
        an.cmp(bn)
    });

    for pkg in &sorted {
        let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("");
        println!("{:<width$}  {}{}{}", name, p.muted, version, p.reset, width = max_name);
    }

    0
}
