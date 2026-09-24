use crate::{output, theme};
use std::process::{Command, Stdio};

pub fn run(query: &str) -> i32 {
    let p = theme::Palette::load();

    let status = Command::new("nix")
        .args([
            "--extra-experimental-features",
            "nix-command flakes",
            "search",
            "nixpkgs",
            query,
        ])
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => 0,
        // Killed by signal (OOM/timeout): results already printed to stdout.
        // std::process::ExitStatus::code() returns None for signal-killed processes.
        Ok(s) if s.code().is_none() => 0,
        Ok(_) => 1,
        Err(e) => {
            output::err(&p, &format!("search failed: {e}"));
            1
        }
    }
}
