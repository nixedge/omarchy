use std::os::unix::process::CommandExt;

fn exec_dev(subcmd: &str, args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-dev-{}", omarchy_path, subcmd);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn add_migration(args: &[String]) -> i32 {
    exec_dev("add-migration", args)
}

pub fn benchmark_cli(args: &[String]) -> i32 {
    exec_dev("benchmark-cli", args)
}

pub fn benchmark_theme_switcher(args: &[String]) -> i32 {
    exec_dev("benchmark-theme-switcher", args)
}

pub fn font(args: &[String]) -> i32 {
    exec_dev("font", args)
}

pub fn theme_preview(args: &[String]) -> i32 {
    exec_dev("theme-preview", args)
}

pub fn ui_preview(args: &[String]) -> i32 {
    exec_dev("ui-preview", args)
}
