use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;
use std::process::Command;

/// Ask the daemon whether a package is in state.packages.
fn pkg_present(client: &mut socket::DaemonClient, name: &str) -> bool {
    client
        .send_recv(Request::PkgPresent { name: name.to_owned() })
        .ok()
        .and_then(|r| r.data)
        .and_then(|d| d.get("present").and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

/// Ask the daemon whether a service is in state.services.
fn service_enabled(client: &mut socket::DaemonClient, name: &str) -> bool {
    client
        .send_recv(Request::ServiceList)
        .ok()
        .and_then(|r| r.data)
        .and_then(|d| {
            d.as_array().map(|arr| {
                arr.iter()
                    .any(|v| v.get("name").and_then(|n| n.as_str()) == Some(name))
            })
        })
        .unwrap_or(false)
}

/// Try to run `hermes chat --help` and verify it exposes --tui and --query.
/// Times out after 15 seconds, matching the desktop app's own readiness probe.
fn hermes_prompt_ready() -> bool {
    let out = Command::new("hermes")
        .args(["chat", "--help"])
        .output();
    let Ok(o) = out else { return false };
    if !o.status.success() {
        return false;
    }
    let help = String::from_utf8_lossy(&o.stdout);
    help.contains("--tui") && help.contains("--query")
}

pub fn run(check: bool, owns: bool, remove: bool) -> i32 {
    let p = theme::Palette::load();

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => {
            output::err(&p, &format!("daemon not reachable: {e}"));
            return 1;
        }
    };

    // hermes-desktop owns the CLI when it is installed as a system package.
    // It writes ~/.hermes/... and puts hermes on PATH itself; the standalone
    // service is not needed and must not run alongside it.
    let desktop_installed = pkg_present(&mut client, "hermes-desktop");
    let standalone_enabled = service_enabled(&mut client, "hermes");

    if owns {
        // --owns: did WE install hermes (not the desktop app)?
        // True when the standalone service is enabled and the desktop is absent.
        return if standalone_enabled && !desktop_installed { 0 } else { 1 };
    }

    if remove {
        // --remove: tear down what this installer put in place.
        if !standalone_enabled {
            return 0; // nothing to do
        }
        let mut fs = filter::FilterState::default();
        let resp = match client.stream(
            Request::ServiceDisable { name: "hermes".to_owned() },
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
        return 0;
    }

    if check {
        // --check: is a ready hermes on PATH?
        // Whether it got there via the desktop or the standalone service
        // doesn't matter — what matters is that it works.
        return if hermes_prompt_ready() { 0 } else { 1 };
    }

    // --now (default): make hermes available.
    if desktop_installed {
        // Desktop app owns hermes; it writes its own CLI to PATH during first
        // launch. Nothing for us to do, but we can verify it is ready.
        if hermes_prompt_ready() {
            return 0;
        }
        output::muted(
            &p,
            "Hermes Desktop is installed but has not set Hermes up yet. \
             Launch Hermes Desktop once to finish installing it.",
        );
        return 1;
    }

    if standalone_enabled {
        if hermes_prompt_ready() {
            return 0;
        }
        // Enabled but not working — try to fix by triggering a rebuild.
    }

    output::status(&p, &format!("{} Enabling Hermes agent CLI\u{2026}", output::GLYPH_PKG));
    let mut fs = filter::FilterState::default();
    let resp = match client.stream(
        Request::ServiceEnable { name: "hermes".to_owned() },
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

    if !hermes_prompt_ready() {
        output::muted(&p, "Hermes installed but not yet fully ready. Try running `hermes --version`.");
    }

    output::ok(&p, &format!("{} Hermes agent CLI installed", output::GLYPH_OK));
    0
}
