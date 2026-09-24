use std::fs;
use std::path::Path;
use std::process::Command;

const RETROARCH_CORES: &[&str] = &[
    "atari800", "beetle_lynx", "beetle_pce", "beetle_pce_fast", "beetle_pcfx",
    "beetle_supergrafx", "bsnes", "bsnes_hd_beta", "bsnes_mercury_accuracy",
    "bsnes_mercury_balanced", "bsnes_mercury_performance", "cap32", "crocods",
    "desmume", "desmume2015", "dosbox_core", "dosbox_pure", "dosbox_svn",
    "fbalpha2012", "fbalpha2012_cps1", "fbalpha2012_cps2", "fbalpha2012_cps3",
    "fbalpha2012_neogeo", "fbneo", "fceumm", "flycast", "fmsx", "freechaf",
    "freeintv", "fuse", "gambatte", "gearboy", "gearcoleco", "gearsystem",
    "genesis_plus_gx", "genesis_plus_gx_wide", "gpsp", "gw", "handy",
    "hatari", "highscore", "higan_sfc", "higan_sfc_balanced", "kronos",
    "lowresnx", "lutro", "mame", "mame2000", "mame2003", "mame2003_plus",
    "mame2010", "mame2015", "mame2016", "mednafen_gba", "mednafen_lynx",
    "mednafen_ngp", "mednafen_pce", "mednafen_pce_fast", "mednafen_pcfx",
    "mednafen_psx", "mednafen_psx_hw", "mednafen_saturn", "mednafen_supergrafx",
    "mednafen_vb", "mednafen_wswan", "melonds", "mesen", "mesen_s",
    "meteor", "mgba", "mrboom", "mupen64plus_next", "neocd", "nestopia",
    "nxengine", "o2em", "opera", "parallel_n64", "picodrive", "play",
    "pocket_cdg", "pocketcdg", "potator", "ppsspp", "prboom", "prosystem",
    "puae", "puae2021", "px68k", "quasi88", "quicknes", "race", "reminiscence",
    "same_cdi", "sameduck", "scummvm", "smsplus", "snes9x", "snes9x2002",
    "snes9x2005", "snes9x2005_plus", "snes9x2010", "stella", "stella2014",
    "tgbdual", "theodore", "thepowdertoy", "tic80", "tyrquake", "ume",
    "uzem", "vbam", "vba_next", "vecx", "vice_x128", "vice_x64", "vice_x64sc",
    "vice_xpet", "vice_xplus4", "vice_xvic", "virtualjaguar", "vitaquake2",
    "vitaquake2_rogue", "vitaquake2_xatrix", "vitaquake2_zaero", "vitaquake3",
    "wasm4", "x1", "xrick", "yabause",
];

pub fn retro_cores() -> i32 {
    let libretro_dir = "/usr/lib/libretro";
    let Ok(entries) = fs::read_dir(libretro_dir) else {
        eprintln!("RetroArch libretro directory not found: {libretro_dir}");
        return 1;
    };

    let installed: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with("_libretro.so") {
                let core = name.trim_end_matches("_libretro.so").to_string();
                if RETROARCH_CORES.contains(&core.as_str()) {
                    return Some(format!("{} ({libretro_dir}/{name})", core.replace('_', " ")));
                }
            }
            None
        })
        .collect();

    for core in &installed {
        println!("{core}");
    }
    0
}

pub fn retro_install(core: Option<&str>, game: Option<&str>) -> i32 {
    let (core_path, game_path) = if let (Some(c), Some(g)) = (core, game) {
        // Both provided
        let core_path = if c.contains('/') {
            c.to_string()
        } else {
            format!("/usr/lib/libretro/{c}_libretro.so")
        };
        (core_path, g.to_string())
    } else if core.is_none() && game.is_none() {
        // Interactive
        let cores_out = Command::new("omarchy-games-retro-cores").output();
        let cores_str = match cores_out {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => String::new(),
        };
        let cores: Vec<&str> = cores_str.lines().filter(|l| !l.is_empty()).collect();

        if cores.is_empty() {
            Command::new("omarchy-notification-send")
                .args(["-g", "󰯉", "No RetroArch cores found", "/usr/lib/libretro"])
                .status().ok();
            return 1;
        }

        // Select core
        let core_sel = Command::new("omarchy-menu-select")
            .arg("RetroArch core")
            .args(&cores)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        if core_sel.is_empty() { return 0; }

        // Extract path from format "name (path)"
        let core_path = core_sel.trim_end_matches(')')
            .rsplit_once('(')
            .map(|(_, p)| p.to_string())
            .unwrap_or(core_sel.clone());

        // Select game file
        let game_sel = Command::new("omarchy-menu-file")
            .args(["Retro game", &format!("{}/Games/roms", std::env::var("HOME").unwrap_or_default()),
                "7z bin ccd chd cue dmg elf fds gb gba gbc iso lha m3u md n64 nds nes pbp sfc smc swc zip z64"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        if game_sel.is_empty() { return 0; }

        (core_path, game_sel)
    } else {
        eprintln!("Usage: omarchy-games-retro-install [core path-to-game]");
        return 1;
    };

    if !Path::new(&game_path).is_file() {
        eprintln!("Game not found: {game_path}");
        return 1;
    }

    if !Path::new(&core_path).is_file() {
        eprintln!("Core not found: {core_path}");
        return 1;
    }

    // Generate game name from filename
    let filename = Path::new(&game_path).file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // Remove parenthetical suffixes and capitalize
    let game_name = clean_game_name(&filename);

    let desktop_name = game_name.clone();
    let desktop_id: String = desktop_name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    let home = std::env::var("HOME").unwrap_or_default();
    let desktop_dir = format!("{home}/.local/share/applications");
    let _ = fs::create_dir_all(&desktop_dir);
    let desktop_file = format!("{desktop_dir}/{desktop_id}.desktop");

    let desktop_content = format!(
        "[Desktop Entry]\nVersion=1.0\nName={desktop_name}\nComment=Play {game_name} with RetroArch\nExec=retroarch -L \"{core_path}\" \"{game_path}\"\nTerminal=false\nType=Application\nIcon=retro-gaming\nStartupNotify=true\nCategories=Game;Emulator;\n"
    );

    if let Err(e) = fs::write(&desktop_file, &desktop_content) {
        eprintln!("Cannot write desktop file: {e}");
        return 1;
    }

    let _ = Command::new("update-desktop-database")
        .arg(&desktop_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    Command::new("omarchy-notification-send")
        .args(["-g", "󰯉", &format!("{game_name} installed"), "Start it with Super + Space"])
        .status().ok();

    0
}

fn clean_game_name(filename: &str) -> String {
    // Remove parenthetical parts like (USA) (v1.0) etc.
    let mut name = String::new();
    let mut depth = 0;
    for c in filename.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => { if depth > 0 { depth -= 1; } }
            _ if depth == 0 => name.push(c),
            _ => {}
        }
    }

    // Capitalize words
    let name = name.trim().to_string();
    name.split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
