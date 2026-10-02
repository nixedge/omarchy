use std::os::unix::process::CommandExt;
use std::process::Command;

fn exec_provision(script_name: &str, args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-provision-{}", omarchy_path, script_name);
    let err = Command::new(&script).args(args).exec();
    eprintln!("exec {}: {}", script, err);
    1
}

pub fn first_run(args: &[String]) -> i32 {
    exec_provision("first-run", args)
}

pub fn user(args: &[String]) -> i32 {
    exec_provision("user", args)
}
