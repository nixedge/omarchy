use std::collections::HashSet;
use std::fs;
use std::path::Path;

use clap::CommandFactory;

use crate::cli::Cli;

// ── Data model ───────────────────────────────────────────────────────────────

#[derive(Debug)]
struct CommandEntry {
    binary: String,
    route: String,
    filename_route: String,
    routes: Vec<String>,
    summary: Option<String>,
    args: Option<String>,
    requires_sudo: bool,
    hidden: bool,
    aliases: Vec<String>,
}

// ── Shell script metadata parsing ────────────────────────────────────────────

fn parse_script_metadata(path: &Path) -> Option<CommandEntry> {
    let content = fs::read_to_string(path).ok()?;
    let filename = path.file_name()?.to_str()?;

    if !filename.starts_with("omarchy-") {
        return None;
    }

    // Parse the header: stop at first non-blank, non-comment line (code ends header).
    let mut summary: Option<String> = None;
    let mut group: Option<String> = None;
    let mut name: Option<Option<String>> = None; // Some(None) means key present but empty
    let mut args: Option<String> = None;
    let mut requires_sudo = false;
    let mut hidden = false;
    let mut aliases: Vec<String> = Vec::new();

    let mut past_shebang = false;
    let mut in_header = true;

    for line in content.lines() {
        // Skip the shebang on the first line
        if !past_shebang {
            past_shebang = true;
            if line.starts_with("#!") {
                continue;
            }
        }

        if !in_header {
            break;
        }

        // Blank lines are allowed in the header
        if line.trim().is_empty() {
            continue;
        }

        // Non-comment, non-blank line ends the header
        if !line.starts_with('#') {
            in_header = false;
            break;
        }

        // Parse omarchy: metadata lines
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("# omarchy:") {
            if let Some(eq_pos) = rest.find('=') {
                let key = &rest[..eq_pos];
                let val = &rest[eq_pos + 1..];
                match key {
                    "summary" => summary = Some(val.to_string()),
                    "group" => group = Some(val.to_string()),
                    "name" => {
                        if val.is_empty() {
                            name = Some(None);
                        } else {
                            name = Some(Some(val.to_string()));
                        }
                    }
                    "args" => {
                        if !val.is_empty() {
                            args = Some(val.to_string());
                        }
                    }
                    "requires-sudo" => {
                        if val == "true" {
                            requires_sudo = true;
                        }
                    }
                    "hidden" => {
                        if val == "true" {
                            hidden = true;
                        }
                    }
                    "aliases" => {
                        // The value is a full alias route (e.g. "omarchy screenshot")
                        let alias_route = val.trim().to_string();
                        if !alias_route.is_empty() {
                            aliases.push(alias_route);
                        }
                    }
                    "alias" => {
                        // Singular alias, can appear multiple times
                        let alias_route = val.trim().to_string();
                        if !alias_route.is_empty() {
                            aliases.push(alias_route);
                        }
                    }
                    _ => {
                        // Ignore unknown keys
                    }
                }
            }
            // Lines like "# omarchy:this malformed line should be ignored" (no =) are ignored
        }
        // Plain comments without "# omarchy:" prefix are fine and ignored
    }

    // Derive filename_route: binary name with all hyphens -> spaces
    let filename_route = filename.replace('-', " ");

    // Derive route from group/name metadata or fall back to filename_route
    let route = if let Some(ref grp) = group {
        match &name {
            Some(None) => {
                // name= is empty: route = "omarchy {group}"
                format!("omarchy {}", grp)
            }
            Some(Some(n)) => {
                // name is set: route = "omarchy {group} {name}"
                format!("omarchy {} {}", grp, n)
            }
            None => {
                // group set but no name override — fall back to filename_route
                filename_route.clone()
            }
        }
    } else {
        filename_route.clone()
    };

    // routes = [filename_route] + aliases
    let mut routes = vec![filename_route.clone()];
    routes.extend(aliases.iter().cloned());

    Some(CommandEntry {
        binary: filename.to_string(),
        route,
        filename_route,
        routes,
        summary,
        args,
        requires_sudo,
        hidden,
        aliases,
    })
}

// ── Clap tree walking ─────────────────────────────────────────────────────────

/// Expand a Clap subcommand segment to route words by splitting at the FIRST hyphen.
///
/// Examples:
///   "pkg"               → "pkg"
///   "add"               → "add"
///   "gaming-xbox-cloud" → "gaming xbox-cloud"   (first hyphen splits; rest kept)
///   "gaming-gpu-lib32"  → "gaming gpu-lib32"    (first hyphen splits; rest kept)
///   "benchmark-cli"     → "benchmark cli"
///
/// This makes the canonical route differ from filename_route only when a
/// subcommand segment contains a hyphen that separates its group prefix from
/// the command name (e.g. "gaming-xbox-cloud" lives in the "gaming" group with
/// command name "xbox-cloud").
fn expand_seg(seg: &str) -> String {
    match seg.find('-') {
        None => seg.to_string(),
        Some(pos) => {
            let first = &seg[..pos];
            let rest = &seg[pos + 1..];
            format!("{} {}", first, rest)
        }
    }
}

fn walk_clap_tree(
    cmd: &clap::Command,
    path: &mut Vec<String>,
    entries: &mut Vec<CommandEntry>,
    shell_binaries: &HashSet<String>,
) {
    let name = cmd.get_name();

    // Skip meta-subcommands
    if matches!(name, "help" | "commands" | "completions") {
        return;
    }

    let subs: Vec<_> = cmd.get_subcommands().collect();

    if subs.is_empty() {
        // Leaf command — emit an entry
        // Build binary name: "omarchy-" + path segments joined by "-"
        let binary = format!("omarchy-{}", path.join("-"));

        // Skip if there's already a shell script for this binary
        if shell_binaries.contains(&binary) {
            return;
        }

        // filename_route: binary with all hyphens -> spaces
        let filename_route = binary.replace('-', " ");

        // route: "omarchy " + path segments each expanded
        let route_parts: Vec<String> = path.iter().map(|seg| expand_seg(seg)).collect();
        let route = format!("omarchy {}", route_parts.join(" "));

        let routes = vec![filename_route.clone()];

        let summary = cmd.get_about().map(|s| s.to_string());

        entries.push(CommandEntry {
            binary,
            route,
            filename_route,
            routes,
            summary,
            args: None,
            requires_sudo: false,
            hidden: cmd.is_hide_set(),
            aliases: vec![],
        });
    } else {
        // Non-leaf — recurse into subcommands
        for sub in subs {
            let sub_name = sub.get_name().to_string();
            path.push(sub_name);
            walk_clap_tree(sub, path, entries, shell_binaries);
            path.pop();
        }
    }
}

// ── Scan directory for omarchy-* scripts ─────────────────────────────────────

fn scan_dir(dir: &Path) -> Vec<CommandEntry> {
    let mut entries = Vec::new();

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let fname = match path.file_name().and_then(|n| n.to_str()) {
            Some(f) => f.to_string(),
            None => continue,
        };

        if !fname.starts_with("omarchy-") {
            continue;
        }

        // Only include executable files
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(&path) {
                if meta.permissions().mode() & 0o111 == 0 {
                    continue;
                }
            }
        }

        if let Some(mut entry) = parse_script_metadata(&path) {
            // Synthesize summary for scripts with no metadata summary
            if entry.summary.is_none() {
                let stem = fname.strip_prefix("omarchy-").unwrap_or(&fname);
                let name_words = stem.replace('-', " ");
                entry.summary = Some(format!("Run the {} command", name_words));
            }

            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| a.binary.cmp(&b.binary));
    entries
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(all: bool, json: bool, check: bool) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let bin_dir = Path::new(&omarchy_path).join("bin");

    // Scan shell scripts in $OMARCHY_PATH/bin/
    let shell_entries: Vec<CommandEntry> = scan_dir(&bin_dir);

    // Build set of shell script binary names for dedup
    let shell_binaries: HashSet<String> = shell_entries.iter().map(|e| e.binary.clone()).collect();

    // Walk Clap tree for Rust-only commands
    let root_cmd = Cli::command();
    let mut rust_entries: Vec<CommandEntry> = Vec::new();

    for sub in root_cmd.get_subcommands() {
        let sub_name = sub.get_name().to_string();
        if matches!(sub_name.as_str(), "help" | "commands" | "completions") {
            continue;
        }
        let mut path = vec![sub_name];
        walk_clap_tree(sub, &mut path, &mut rust_entries, &shell_binaries);
    }

    // When --all, also scan the directory of argv[0] for extension scripts
    let mut extra_entries: Vec<CommandEntry> = Vec::new();
    if all {
        if let Some(argv0) = std::env::args().next() {
            let argv0_path = Path::new(&argv0);
            if let Some(parent) = argv0_path.parent() {
                // Skip empty parent (no directory component) and the main bin dir
                let parent_str = parent.to_str().unwrap_or("");
                if !parent_str.is_empty() && parent.as_os_str() != bin_dir.as_os_str() {
                    let mut ext = scan_dir(parent);
                    // Only include entries not already covered by shell_binaries or rust_entries
                    let rust_binaries: HashSet<String> =
                        rust_entries.iter().map(|e| e.binary.clone()).collect();
                    ext.retain(|e| {
                        !shell_binaries.contains(&e.binary) && !rust_binaries.contains(&e.binary)
                    });
                    extra_entries.extend(ext);
                }
            }
        }
    }

    if check {
        return 0;
    }

    // Combine all entries:
    //   - shell entries: include hidden only when --all
    //   - rust entries: include hidden only when --all
    //   - extra entries from argv[0] dir: only in --all mode
    let all_entries: Vec<&CommandEntry> = {
        let mut v: Vec<&CommandEntry> = if all {
            shell_entries.iter().collect()
        } else {
            shell_entries.iter().filter(|e| !e.hidden).collect()
        };
        if all {
            v.extend(rust_entries.iter());
        } else {
            v.extend(rust_entries.iter().filter(|e| !e.hidden));
        }
        v.extend(extra_entries.iter());
        v
    };

    if json {
        output_json(&all_entries)
    } else {
        output_text(&all_entries)
    }
}

fn output_json(entries: &[&CommandEntry]) -> i32 {
    use serde_json::{json, Value};

    let commands: Vec<Value> = entries
        .iter()
        .map(|e| {
            let summary = e
                .summary
                .as_deref()
                .unwrap_or("undocumented")
                .to_string();

            let mut obj = json!({
                "binary": e.binary,
                "route": e.route,
                "filename_route": e.filename_route,
                "routes": e.routes,
                "summary": summary,
            });

            let map = obj.as_object_mut().unwrap();

            if e.requires_sudo {
                map.insert("requires_sudo".to_string(), json!(true));
            }

            if !e.aliases.is_empty() {
                map.insert("aliases".to_string(), json!(e.aliases));
            }

            if e.hidden {
                map.insert("hidden".to_string(), json!(true));
            }

            obj
        })
        .collect();

    let output = json!({
        "ok": true,
        "commands": commands,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    0
}

fn output_text(entries: &[&CommandEntry]) -> i32 {
    for entry in entries {
        if entry.hidden {
            continue;
        }

        let route_with_args = match &entry.args {
            Some(a) if !a.is_empty() => format!("{} {}", entry.route, a),
            _ => entry.route.clone(),
        };

        let summary = entry.summary.as_deref().unwrap_or("");

        if summary.is_empty() {
            println!("{}", route_with_args);
        } else {
            println!("{}  {}", route_with_args, summary);
        }
    }
    0
}
