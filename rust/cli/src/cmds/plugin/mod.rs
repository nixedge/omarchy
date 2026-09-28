use std::os::unix::process::CommandExt;

fn exec_plugin(subcmd: &str, args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-plugin-{}", omarchy_path, subcmd);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn add(args: &[String]) -> i32 {
    exec_plugin("add", args)
}

pub fn catalog(args: &[String]) -> i32 {
    exec_plugin("catalog", args)
}

pub fn clone(args: &[String]) -> i32 {
    exec_plugin("clone", args)
}

pub fn disable(args: &[String]) -> i32 {
    exec_plugin("disable", args)
}

pub fn enable(args: &[String]) -> i32 {
    exec_plugin("enable", args)
}

pub fn list(args: &[String]) -> i32 {
    exec_plugin("list", args)
}

pub fn remove(args: &[String]) -> i32 {
    exec_plugin("remove", args)
}

pub fn update(args: &[String]) -> i32 {
    exec_plugin("update", args)
}

pub fn validate(args: &[String]) -> i32 {
    exec_plugin("validate", args)
}
