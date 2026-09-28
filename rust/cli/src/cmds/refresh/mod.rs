use std::os::unix::process::CommandExt;

fn exec_refresh(subcmd: &str, args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-refresh-{}", omarchy_path, subcmd);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn applications(args: &[String]) -> i32 {
    exec_refresh("applications", args)
}

pub fn chromium(args: &[String]) -> i32 {
    exec_refresh("chromium", args)
}

pub fn config(args: &[String]) -> i32 {
    exec_refresh("config", args)
}

pub fn herdr(args: &[String]) -> i32 {
    exec_refresh("herdr", args)
}

pub fn hyprland(args: &[String]) -> i32 {
    exec_refresh("hyprland", args)
}

pub fn hyprsunset(args: &[String]) -> i32 {
    exec_refresh("hyprsunset", args)
}

pub fn shell(args: &[String]) -> i32 {
    exec_refresh("shell", args)
}

pub fn tmux(args: &[String]) -> i32 {
    exec_refresh("tmux", args)
}
