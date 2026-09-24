use std::process::{Command, Stdio};

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_default()
}

fn exec_delegate(script_name: &str, args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/{}", omarchy_path, script_name);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

// ─── band ────────────────────────────────────────────────────────────────────

pub fn band(args: &[String]) -> i32 {
    exec_delegate("omarchy-network-band", args)
}

// ─── password ────────────────────────────────────────────────────────────────

pub fn password(args: &[String]) -> i32 {
    exec_delegate("omarchy-network-password", args)
}

// ─── qr ──────────────────────────────────────────────────────────────────────

pub fn qr(args: &[String]) -> i32 {
    exec_delegate("omarchy-network-qr", args)
}

// ─── speedtest ───────────────────────────────────────────────────────────────

pub fn speedtest(args: &[String]) -> i32 {
    exec_delegate("omarchy-network-speedtest", args)
}

// ─── status ──────────────────────────────────────────────────────────────────

pub fn status(args: &[String]) -> i32 {
    exec_delegate("omarchy-network-status", args)
}
