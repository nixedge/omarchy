use crate::{output, theme};
use omarchy_lib::alias::{self, ResolveError};

pub fn run(name: &str) -> i32 {
    let p = theme::Palette::load();
    match alias::resolve(name) {
        Ok(attr) => {
            println!("{attr}");
            0
        }
        Err(ResolveError::ServiceManaged(opt)) => {
            output::err(&p, &format!("'{name}' is managed by NixOS option `{opt}`; use omarchy-setup to toggle it"));
            1
        }
        Err(ResolveError::Eliminated(reason)) => {
            output::err(&p, &format!("'{name}' is not available on NixOS: {reason}"));
            1
        }
    }
}
