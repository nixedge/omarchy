use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;
use std::io::Write as _;

pub fn run() -> i32 {
    let p = theme::Palette::load();

    // Fetch current config from daemon.
    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => { output::err(&p, &format!("daemon not reachable: {e}")); return 1; }
    };
    let resp = match client.send_recv(Request::ConfigGet) {
        Ok(r) => r,
        Err(e) => { output::err(&p, &format!("daemon error: {e}")); return 1; }
    };
    if !resp.ok {
        output::err(&p, resp.err_msg());
        return 1;
    }
    let current = resp.data
        .as_ref()
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_owned();

    // Write to a temp file and open in $EDITOR.
    let tmp_path = format!("/tmp/omarchy-user-{}.nix", std::process::id());
    {
        let mut f = match std::fs::File::create(&tmp_path) {
            Ok(f) => f,
            Err(e) => { output::err(&p, &format!("failed to create temp file: {e}")); return 1; }
        };
        if let Err(e) = f.write_all(current.as_bytes()) {
            output::err(&p, &format!("failed to write temp file: {e}")); return 1;
        }
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_owned());
    let status = std::process::Command::new(&editor)
        .arg(&tmp_path)
        .status();

    let edited = match std::fs::read_to_string(&tmp_path) {
        Ok(s) => s,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp_path);
            output::err(&p, &format!("failed to read temp file: {e}"));
            return 1;
        }
    };
    let _ = std::fs::remove_file(&tmp_path);

    match status {
        Ok(s) if !s.success() => {
            output::err(&p, &format!("editor exited with error"));
            return 1;
        }
        Err(e) => {
            output::err(&p, &format!("failed to launch {editor}: {e}"));
            return 1;
        }
        _ => {}
    }

    if edited == current {
        output::muted(&p, "No changes.");
        return 0;
    }

    // Apply via daemon (streams rebuild progress).
    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => { output::err(&p, &format!("daemon not reachable: {e}")); return 1; }
    };
    output::status(&p, &format!("{} Applying configuration\u{2026}", output::GLYPH_PKG));
    let mut fs = filter::FilterState::default();
    let resp = match client.stream(Request::ConfigApply { content: edited }, |line| {
        if let Some(out) = filter::filter(line, &mut fs, &p) {
            println!("{out}");
        }
    }) {
        Ok(r) => r,
        Err(e) => { output::err(&p, &format!("daemon error: {e}")); return 1; }
    };

    if resp.ok {
        output::ok(&p, &format!("{} Configuration applied", output::GLYPH_OK));
        0
    } else {
        output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
        1
    }
}
