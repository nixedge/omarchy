use std::os::unix::process::CommandExt;

pub fn info(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-drive-info", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn password(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-drive-password", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn select(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-drive-select", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}
