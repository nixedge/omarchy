use std::os::unix::process::CommandExt;

pub fn hey(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-webapp-handler-hey", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn zoom(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-webapp-handler-zoom", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}
