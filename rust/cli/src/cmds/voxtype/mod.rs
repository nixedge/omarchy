use std::os::unix::process::CommandExt;

pub fn config(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-voxtype-config", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn model(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-voxtype-model", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn status(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-voxtype-status", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}
