use std::os::unix::process::CommandExt;

fn exec_theme(subcmd: &str, args: &[String]) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").expect("OMARCHY_PATH must be set");
    let script = format!("{}/bin/omarchy-theme-{}", omarchy_path, subcmd);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

pub fn bg_cache(args: &[String]) -> i32 {
    exec_theme("bg-cache", args)
}

pub fn bg_current(args: &[String]) -> i32 {
    exec_theme("bg-current", args)
}

pub fn bg_install(args: &[String]) -> i32 {
    exec_theme("bg-install", args)
}

pub fn bg_next(args: &[String]) -> i32 {
    exec_theme("bg-next", args)
}

pub fn bg_set(args: &[String]) -> i32 {
    exec_theme("bg-set", args)
}

pub fn bg_switcher(args: &[String]) -> i32 {
    exec_theme("bg-switcher", args)
}

pub fn color(args: &[String]) -> i32 {
    exec_theme("color", args)
}

pub fn colors_from_alacritty(args: &[String]) -> i32 {
    exec_theme("colors-from-alacritty", args)
}

pub fn current(args: &[String]) -> i32 {
    exec_theme("current", args)
}

pub fn dir(args: &[String]) -> i32 {
    exec_theme("dir", args)
}

pub fn extras(args: &[String]) -> i32 {
    exec_theme("extras", args)
}

pub fn install(args: &[String]) -> i32 {
    exec_theme("install", args)
}

pub fn list(args: &[String]) -> i32 {
    exec_theme("list", args)
}

pub fn osc(args: &[String]) -> i32 {
    exec_theme("osc", args)
}

pub fn refresh(args: &[String]) -> i32 {
    exec_theme("refresh", args)
}

pub fn remove(args: &[String]) -> i32 {
    exec_theme("remove", args)
}

pub fn set(args: &[String]) -> i32 {
    exec_theme("set", args)
}

pub fn set_browser(args: &[String]) -> i32 {
    exec_theme("set-browser", args)
}

pub fn set_browser_policy(args: &[String]) -> i32 {
    exec_theme("set-browser-policy", args)
}

pub fn set_claude(args: &[String]) -> i32 {
    exec_theme("set-claude", args)
}

pub fn set_foot(args: &[String]) -> i32 {
    exec_theme("set-foot", args)
}

pub fn set_gnome(args: &[String]) -> i32 {
    exec_theme("set-gnome", args)
}

pub fn set_hermes(args: &[String]) -> i32 {
    exec_theme("set-hermes", args)
}

pub fn set_keyboard(args: &[String]) -> i32 {
    exec_theme("set-keyboard", args)
}

pub fn set_keyboard_asus_rog(args: &[String]) -> i32 {
    exec_theme("set-keyboard-asus-rog", args)
}

pub fn set_keyboard_f16(args: &[String]) -> i32 {
    exec_theme("set-keyboard-f16", args)
}

pub fn set_obsidian(args: &[String]) -> i32 {
    exec_theme("set-obsidian", args)
}

pub fn set_pi(args: &[String]) -> i32 {
    exec_theme("set-pi", args)
}

pub fn set_t3code(args: &[String]) -> i32 {
    exec_theme("set-t3code", args)
}

pub fn set_templates(args: &[String]) -> i32 {
    exec_theme("set-templates", args)
}

pub fn set_tmux(args: &[String]) -> i32 {
    exec_theme("set-tmux", args)
}

pub fn set_vscode(args: &[String]) -> i32 {
    exec_theme("set-vscode", args)
}

pub fn switcher(args: &[String]) -> i32 {
    exec_theme("switcher", args)
}

pub fn update(args: &[String]) -> i32 {
    exec_theme("update", args)
}
