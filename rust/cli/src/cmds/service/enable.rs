use crate::{filter, output, socket, theme};
use omarchy_lib::protocol::Request;

// Post-enable notes for services that need additional setup steps after rebuild.
fn post_enable_note(name: &str) -> Option<&'static str> {
    match name {
        "tailscale" => Some("Run 'sudo tailscale up' to authenticate and connect."),
        "nordvpn" => Some("Reboot, then run 'nordvpn login' to authenticate."),
        "fingerprint" => Some("Enroll your fingerprint: sudo fprintd-enroll $USER"),
        "fido2" => Some("Register your FIDO2 key: pamu2fcfg | sudo tee /etc/fido2/fido2"),
        "sshd" => Some("Add an authorized key: ssh-copy-id or append to ~/.ssh/authorized_keys"),
        "dropbox" => Some("Start Dropbox and authenticate via the system tray icon."),
        "1password" => Some("Open 1Password and sign in. Restart Chromium to load the extension."),
        _ => None,
    }
}

pub fn run(name: &str) -> i32 {
    let p = theme::Palette::load();

    let mut client = match socket::DaemonClient::connect() {
        Ok(c) => c,
        Err(e) => { output::err(&p, &format!("daemon not reachable: {e}")); return 1; }
    };

    output::status(&p, &format!("{} Enabling service '{name}'\u{2026}", output::GLYPH_PKG));

    let mut fs = filter::FilterState::default();
    let resp = match client.stream(Request::ServiceEnable { name: name.to_owned() }, |line| {
        if let Some(out) = filter::filter(line, &mut fs, &p) {
            println!("{out}");
        }
    }) {
        Ok(r) => r,
        Err(e) => { output::err(&p, &format!("daemon error: {e}")); return 1; }
    };

    if !resp.ok {
        output::err(&p, &format!("{} {}", output::GLYPH_FAIL, resp.err_msg()));
        return 1;
    }

    output::ok(&p, &format!("{} Service '{name}' enabled", output::GLYPH_OK));

    if let Some(note) = post_enable_note(name) {
        println!("{}{}{}", p.muted, note, p.reset);
    }

    0
}
