use std::os::unix::process::CommandExt;

fn exec_menu(subcmd: &str, args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-menu-{}", omarchy_path, subcmd);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn main_menu(args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-menu", omarchy_path);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn clipboard(args: &[String]) -> i32 {
    exec_menu("clipboard", args)
}

pub fn emoji(args: &[String]) -> i32 {
    exec_menu("emoji", args)
}

pub fn emoji_insert(args: &[String]) -> i32 {
    exec_menu("emoji-insert", args)
}

pub fn file(args: &[String]) -> i32 {
    exec_menu("file", args)
}

pub fn herdr_keybindings(args: &[String]) -> i32 {
    exec_menu("herdr-keybindings", args)
}

pub fn images(args: &[String]) -> i32 {
    exec_menu("images", args)
}

pub fn input(args: &[String]) -> i32 {
    exec_menu("input", args)
}

pub fn keybindings(args: &[String]) -> i32 {
    exec_menu("keybindings", args)
}

pub fn plugin(args: &[String]) -> i32 {
    exec_menu("plugin", args)
}

pub fn select(args: &[String]) -> i32 {
    exec_menu("select", args)
}

pub fn share(args: &[String]) -> i32 {
    if args.is_empty() || (args.len() == 1 && (args[0] == "--help" || args[0] == "-h")) {
        println!("Usage: omarchy share <clipboard|file|folder> [path...]");
        println!();
        println!("Share clipboard contents, files, or folders with LocalSend.");
        println!();
        println!("Binary: omarchy-menu-share");
        return 0;
    }
    exec_menu("share", args)
}

pub fn timezone(args: &[String]) -> i32 {
    exec_menu("timezone", args)
}

pub fn tmux_keybindings(args: &[String]) -> i32 {
    exec_menu("tmux-keybindings", args)
}
