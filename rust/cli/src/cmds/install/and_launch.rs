use super::{add_many, launch_desktop};

pub fn run(name: &str, packages: &str, desktop_id: &str) -> i32 {
    let pkg_list: Vec<&str> = packages.split_whitespace().collect();
    println!("Installing {name}\u{2026}");
    let rc = add_many(&pkg_list, true);
    if rc != 0 {
        return rc;
    }
    launch_desktop(desktop_id);
    0
}
