use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;
use std::io::Write as _;

pub fn run() -> i32 {
    let p = theme::Palette::load();

    // Fetch current config from daemon.
    let current = {
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
        resp.data.as_ref().and_then(|d| d.as_str()).unwrap_or("").to_owned()
    };

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_owned());
    let tmp_path = format!("/tmp/omarchy-user-{}.nix", std::process::id());

    // Start with the current config; on failure loop back with the edited content.
    let mut working = current.clone();

    loop {
        // Write working content to temp file.
        {
            let mut f = match std::fs::File::create(&tmp_path) {
                Ok(f) => f,
                Err(e) => { output::err(&p, &format!("failed to create temp file: {e}")); return 1; }
            };
            if let Err(e) = f.write_all(working.as_bytes()) {
                output::err(&p, &format!("failed to write temp file: {e}"));
                return 1;
            }
        }

        let status = std::process::Command::new(&editor).arg(&tmp_path).status();

        let edited = match std::fs::read_to_string(&tmp_path) {
            Ok(s) => s,
            Err(e) => {
                let _ = std::fs::remove_file(&tmp_path);
                output::err(&p, &format!("failed to read temp file: {e}"));
                return 1;
            }
        };

        match status {
            Ok(s) if !s.success() => {
                let _ = std::fs::remove_file(&tmp_path);
                output::err(&p, "editor exited with error");
                return 1;
            }
            Err(e) => {
                let _ = std::fs::remove_file(&tmp_path);
                output::err(&p, &format!("failed to launch {editor}: {e}"));
                return 1;
            }
            _ => {}
        }

        if edited == current {
            let _ = std::fs::remove_file(&tmp_path);
            output::muted(&p, "No changes.");
            return 0;
        }

        // Apply via daemon — streams rebuild progress.
        let mut client = match socket::DaemonClient::connect() {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_file(&tmp_path);
                output::err(&p, &format!("daemon not reachable: {e}"));
                return 1;
            }
        };
        output::status(&p, &format!("{} Applying configuration\u{2026}", output::GLYPH_PKG));
        let mut fs = filter::FilterState::default();
        let resp = match client.stream(Request::ConfigApply { content: edited.clone() }, |line| {
            if let Some(out) = filter::filter(line, &mut fs, &p) {
                println!("{out}");
            }
        }) {
            Ok(r) => r,
            Err(e) => {
                let _ = std::fs::remove_file(&tmp_path);
                output::err(&p, &format!("daemon error: {e}"));
                return 1;
            }
        };

        if resp.ok {
            let _ = std::fs::remove_file(&tmp_path);
            output::ok(&p, &format!("{} Configuration applied", output::GLYPH_OK));
            return 0;
        }

        // Rebuild failed — config was rolled back by daemon. Ask user to re-edit.
        output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
        eprint!("{}Re-open editor to fix the error? [Y/n] {}", p.muted, p.reset);
        let _ = std::io::stderr().flush();
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer).unwrap_or(0);
        if matches!(answer.trim().to_ascii_lowercase().as_str(), "n" | "no") {
            let _ = std::fs::remove_file(&tmp_path);
            output::muted(&p, "Discarded — original configuration is still active.");
            return 1;
        }

        // Loop with the failed edit so the user doesn't lose their work.
        working = edited;
    }
}
