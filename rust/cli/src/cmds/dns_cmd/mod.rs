use std::process::Command;

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_default()
}

/// Delegate to the bash omarchy-dns script (complex privilege escalation, NetworkManager, etc.)
pub fn run(args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = omarchy_path();
    let script = format!("{}/bin/omarchy-dns", omarchy_path);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}
