use super::add_many;

pub fn run(name: &str, packages: &str) -> i32 {
    let pkg_list: Vec<&str> = packages.split_whitespace().collect();
    println!("Installing {name}\u{2026}");
    add_many(&pkg_list, true)
}
