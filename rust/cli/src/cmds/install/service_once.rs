use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;
use std::process::Command;

pub fn run() -> i32 {
    let p = theme::Palette::load();
    output::status(&p, &format!("{} Installing ONCE\u{2026}", output::GLYPH_PKG));

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    let mut fs = filter::FilterState::default();
    let resp = match client.stream(
        Request::PkgAdd { name: "once-bin".to_owned() },
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

    println!("Enabling ONCE background service...");
    let _ = Command::new("sudo")
        .args(["systemctl", "enable", "--now", "once-background.service"])
        .status();

    println!("\nLaunching ONCE...");
    let _ = Command::new("sudo").arg("once").status();

    0
}
