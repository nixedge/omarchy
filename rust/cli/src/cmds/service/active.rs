use std::process::{Command, Stdio};

/// Maps an omarchy service name to its systemd unit and whether it is a user
/// unit (true) or a system unit (false). Returns None for services that are
/// purely NixOS configuration options with no running daemon to check.
fn unit_for(name: &str) -> Option<(&'static str, bool)> {
    match name {
        "tailscale" => Some(("tailscaled", false)),
        "dropbox" => Some(("dropbox", true)),
        "nordvpn" => Some(("nordvpnd", false)),
        "sshd" => Some(("sshd", false)),
        "sunshine" => Some(("sunshine", false)),
        _ => None,
    }
}

pub fn run(name: &str) -> i32 {
    let Some((unit, user)) = unit_for(name) else {
        return 1;
    };

    let mut cmd = Command::new("systemctl");
    if user {
        cmd.arg("--user");
    }
    cmd.args(["is-active", "--quiet", unit])
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    cmd.status().map(|s| if s.success() { 0 } else { 1 }).unwrap_or(1)
}
