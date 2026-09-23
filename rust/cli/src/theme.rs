use std::collections::HashMap;
use std::io::IsTerminal;

#[derive(Default, Clone)]
pub struct Palette {
    pub accent: String,
    pub green: String,
    pub red: String,
    pub muted: String,
    pub reset: String,
}

impl Palette {
    pub fn load() -> Self {
        if !std::io::stdout().is_terminal() {
            return Self::default();
        }
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return Self::default(),
        };
        let path = format!("{home}/.local/state/omarchy/current/theme/colors.toml");
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        let table: HashMap<String, String> = match toml::from_str(&content) {
            Ok(t) => t,
            Err(_) => return Self::default(),
        };
        Self {
            accent: ansi_fg(table.get("accent")),
            green: ansi_fg(table.get("green")),
            red: ansi_fg(table.get("red")),
            muted: ansi_fg(table.get("foreground")),
            reset: "\x1b[0m".to_owned(),
        }
    }
}

fn ansi_fg(hex: Option<&String>) -> String {
    let hex = match hex {
        Some(h) => h.trim_start_matches('#'),
        None => return String::new(),
    };
    if hex.len() != 6 {
        return String::new();
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    format!("\x1b[38;2;{r};{g};{b}m")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_fg_valid_hex() {
        assert_eq!(ansi_fg(Some(&"ff0000".to_owned())), "\x1b[38;2;255;0;0m");
    }

    #[test]
    fn ansi_fg_with_hash() {
        assert_eq!(ansi_fg(Some(&"#00ff00".to_owned())), "\x1b[38;2;0;255;0m");
    }

    #[test]
    fn ansi_fg_none_gives_empty() {
        assert!(ansi_fg(None).is_empty());
    }
}
