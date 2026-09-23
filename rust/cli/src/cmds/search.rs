use crate::{output, theme};
use std::process::Command;

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
        .status();

    match status {
        Ok(s) if s.success() => 0,
        Ok(_) => 1,
        Err(e) => {
            output::err(&p, &format!("search failed: {e}"));
            1
        }
    }
}
