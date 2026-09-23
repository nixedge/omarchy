use crate::theme::Palette;

pub const GLYPH_PKG: &str = "\u{f0196}"; // 󰏖

#[derive(Default)]
pub struct FilterState {
    pub building_shown: bool,
}

pub fn filter(line: &str, state: &mut FilterState, p: &Palette) -> Option<String> {
    if line.is_empty() {
        return None;
    }
    if line.starts_with("unpacking '") {
        return None;
    }
    if line.starts_with("copying path ") {
        return None;
    }
    if line.starts_with("querying info") {
        return None;
    }
    if line.trim_start().starts_with("/nix/store/") {
        return None;
    }
    if line.starts_with("these ")
        && (line.contains("derivations will be built")
            || line.contains("paths will be fetched")
            || line.contains("paths will be copied"))
    {
        return None;
    }

    if line.contains("building the system configuration") {
        return Some(format!(
            "{}{} Resolving dependencies\u{2026}{}",
            p.accent, GLYPH_PKG, p.reset
        ));
    }
    if line.contains("activating the configuration") {
        return Some(format!(
            "{}{} Activating configuration\u{2026}{}",
            p.accent, GLYPH_PKG, p.reset
        ));
    }
    if line.starts_with("building '") {
        if !state.building_shown {
            state.building_shown = true;
            return Some(format!(
                "{}{} Building packages\u{2026}{}",
                p.accent, GLYPH_PKG, p.reset
            ));
        }
        return None;
    }

    Some(line.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bare() -> Palette {
        Palette::default()
    }

    #[test]
    fn suppresses_empty() {
        assert!(filter("", &mut FilterState::default(), &bare()).is_none());
    }

    #[test]
    fn suppresses_unpacking() {
        assert!(filter("unpacking 'foo'", &mut FilterState::default(), &bare()).is_none());
    }

    #[test]
    fn suppresses_copying_path() {
        assert!(filter("copying path '/nix/store/abc'", &mut FilterState::default(), &bare()).is_none());
    }

    #[test]
    fn suppresses_store_path() {
        assert!(filter("  /nix/store/foo-1.0/bin/foo", &mut FilterState::default(), &bare()).is_none());
    }

    #[test]
    fn translates_building_system_config() {
        let out = filter("building the system configuration...", &mut FilterState::default(), &bare()).unwrap();
        assert!(out.contains("Resolving dependencies"));
    }

    #[test]
    fn translates_activating() {
        let out = filter("activating the configuration...", &mut FilterState::default(), &bare()).unwrap();
        assert!(out.contains("Activating configuration"));
    }

    #[test]
    fn building_shown_only_once() {
        let mut state = FilterState::default();
        let p = bare();
        let first = filter("building '/nix/store/foo.drv'", &mut state, &p);
        let second = filter("building '/nix/store/bar.drv'", &mut state, &p);
        assert!(first.unwrap().contains("Building packages"));
        assert!(second.is_none());
    }

    #[test]
    fn passthrough_plain_line() {
        let out = filter("some other output", &mut FilterState::default(), &bare()).unwrap();
        assert_eq!(out, "some other output");
    }
}
