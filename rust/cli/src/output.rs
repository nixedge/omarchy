use crate::theme::Palette;

pub const GLYPH_OK: &str = "\u{f00c}";   //
pub const GLYPH_FAIL: &str = "\u{f00d}"; //
pub const GLYPH_PKG: &str = "\u{f0196}"; // 󰏖

pub fn status(p: &Palette, msg: &str) {
    println!("{}{}{}", p.accent, msg, p.reset);
}

pub fn ok(p: &Palette, msg: &str) {
    println!("{}{}{}", p.green, msg, p.reset);
}

pub fn err(p: &Palette, msg: &str) {
    eprintln!("{}{}{}", p.red, msg, p.reset);
}

pub fn muted(p: &Palette, msg: &str) {
    println!("{}{}{}", p.muted, msg, p.reset);
}
