use crate::{output, socket, theme};
use omarchy_lib::protocol::Request;

pub fn run() -> i32 {
    let p = theme::Palette::load();

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => { output::err(&p, &format!("daemon not reachable: {e}")); return 1; }
    };
    let resp = match client.send_recv(Request::ServiceList) {
        Ok(r) => r,
        Err(e) => { output::err(&p, &format!("daemon error: {e}")); return 1; }
    };
    if !resp.ok {
        output::err(&p, resp.err_msg());
        return 1;
    }

    let data = resp.data.as_ref();
    let enabled: Vec<&str> = data
        .and_then(|d| d.get("enabled"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let available: Vec<&str> = data
        .and_then(|d| d.get("available"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    println!("{}Enabled services:{}", p.accent, p.reset);
    if enabled.is_empty() {
        println!("{}  (none){}", p.muted, p.reset);
    } else {
        for svc in &enabled {
            println!("{}  {} {}{}", p.green, output::GLYPH_OK, svc, p.reset);
        }
    }

    println!();
    println!("{}Available services:{}", p.accent, p.reset);
    for svc in &available {
        let marker = if enabled.contains(svc) {
            format!("{} {}", output::GLYPH_OK, p.muted)
        } else {
            format!("  {}", p.reset)
        };
        println!("  {}{}{}{}", marker, svc, p.reset, "");
    }

    0
}
