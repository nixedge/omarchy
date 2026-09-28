use std::os::unix::process::CommandExt;

pub fn run(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-file-select", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}
