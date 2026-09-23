use crate::cmds::present;

pub fn run(name: &str) -> i32 {
    match present::run(name) {
        0 => 1,
        _ => 0,
    }
}
