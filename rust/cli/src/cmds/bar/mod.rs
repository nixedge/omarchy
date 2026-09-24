use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run_output(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn run_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH").unwrap_or_default()
}

fn config_file() -> PathBuf {
    PathBuf::from(home()).join(".config/omarchy/shell.json")
}

fn defaults_file() -> PathBuf {
    PathBuf::from(omarchy_path()).join("config/omarchy/shell.json")
}

fn source_file() -> PathBuf {
    let cf = config_file();
    if cf.exists() && fs::metadata(&cf).map(|m| m.len() > 0).unwrap_or(false) {
        cf
    } else {
        defaults_file()
    }
}

// NORMALIZE jq expression (inlined from omarchy-shell-config)
const NORMALIZE: &str = r#"
  def object_or_empty: if type == "object" then . else {} end;
  def array_or_empty: if type == "array" then . else [] end;
  object_or_empty
  | .version = 1
  | .bar = (.bar | object_or_empty)
  | .bar.layout = (.bar.layout | object_or_empty)
  | .bar.layout.left = (.bar.layout.left | array_or_empty)
  | .bar.layout.center = (.bar.layout.center | array_or_empty)
  | .bar.layout.right = (.bar.layout.right | array_or_empty)
  | .plugins = (.plugins | array_or_empty)
"#;

fn refresh_shell_config() {
    if !run_ok("omarchy-shell", &["shell", "reloadConfig"]) {
        let _ = Command::new("omarchy-shell")
            .args(["-q", "shell", "rescanPlugins"])
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
    }
}

fn fail(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1);
}

// Apply a jq program to source file, atomically write result to user config, then refresh shell
fn commit(program: &str, extra_args: &[&str]) {
    let cf = config_file();
    if let Some(parent) = cf.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let src = source_file();
    let tmp_path = {
        let dir = cf.parent().unwrap_or(std::path::Path::new("/tmp"));
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        dir.join(format!(".shell.json.tmp.{ts}"))
    };

    let mut jq_args = vec!["-S", "-e"];
    jq_args.push(program);
    jq_args.extend_from_slice(extra_args);
    jq_args.push(src.to_str().unwrap_or(""));

    let out = Command::new("jq")
        .args(&jq_args)
        .output();

    match out {
        Ok(o) if o.status.success() => {
            if let Err(e) = fs::write(&tmp_path, &o.stdout) {
                fail(&format!("could not write temp config: {e}"));
            }
            if let Err(e) = fs::rename(&tmp_path, &cf) {
                let _ = fs::remove_file(&tmp_path);
                fail(&format!("could not update shell config: {e}"));
            }
        }
        Ok(o) => {
            let _ = fs::remove_file(&tmp_path);
            let stderr = String::from_utf8_lossy(&o.stderr);
            fail(&format!("could not update shell config: {stderr}"));
        }
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            fail(&format!("could not run jq: {e}"));
        }
    }

    refresh_shell_config();
}

fn jq_output(input: &str, filter: &str) -> Option<String> {
    let mut child = Command::new("jq")
        .args(["-r", filter])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }
    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn jq_check(input: &str, filter: &str) -> bool {
    let mut child = match Command::new("jq")
        .args(["-e", filter])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn() {
        Ok(c) => c,
        Err(_) => return false,
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

fn bar_option_exists(id: &str) -> bool {
    let catalog = run_output("omarchy-plugin-catalog", &[]).unwrap_or_default();
    if catalog.is_empty() { return false; }
    let filter = format!(r#"any(.[]; (.kinds | index("bar")) and .barPath != null and .id == "{id}")"#);
    jq_check(&catalog, &filter)
}

fn validate_section(s: &str) {
    if !matches!(s, "left" | "center" | "right") {
        fail("section must be left, center, or right");
    }
}

fn validate_index(s: &str) {
    if !s.chars().all(|c| c.is_ascii_digit()) {
        fail("index must be a non-negative integer");
    }
}

// Placement state
#[derive(Default)]
struct Placement {
    section: String,
    index: String,
    before: String,
    after: String,
    from_section: String,
    from_index: String,
}

impl Placement {
    fn parse(&mut self, args: &[String]) {
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--section" => {
                    i += 1;
                    self.section = args.get(i).cloned().unwrap_or_default();
                    validate_section(&self.section);
                }
                "--index" => {
                    i += 1;
                    self.index = args.get(i).cloned().unwrap_or_default();
                    validate_index(&self.index);
                }
                "--before" => {
                    i += 1;
                    self.before = args.get(i).cloned().unwrap_or_default();
                    if self.before.is_empty() { fail("--before requires a widget id"); }
                }
                "--after" => {
                    i += 1;
                    self.after = args.get(i).cloned().unwrap_or_default();
                    if self.after.is_empty() { fail("--after requires a widget id"); }
                }
                "--from-section" => {
                    i += 1;
                    self.from_section = args.get(i).cloned().unwrap_or_default();
                    validate_section(&self.from_section);
                }
                "--from-index" => {
                    i += 1;
                    self.from_index = args.get(i).cloned().unwrap_or_default();
                    validate_index(&self.from_index);
                }
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => fail(&format!("unknown option: {other}")),
            }
            i += 1;
        }
        if !self.before.is_empty() && !self.after.is_empty() {
            fail("use only one of --before or --after");
        }
    }

    fn to_json(&self) -> String {
        let mut parts = vec![];
        if !self.section.is_empty() { parts.push(format!(r#""section":"{}""#, self.section)); }
        if !self.index.is_empty() { parts.push(format!(r#""index":{}"#, self.index)); }
        if !self.before.is_empty() { parts.push(format!(r#""before":"{}""#, self.before)); }
        if !self.after.is_empty() { parts.push(format!(r#""after":"{}""#, self.after)); }
        if !self.from_section.is_empty() { parts.push(format!(r#""fromSection":"{}""#, self.from_section)); }
        if !self.from_index.is_empty() { parts.push(format!(r#""fromIndex":{}"#, self.from_index)); }
        format!("{{{}}}", parts.join(","))
    }
}

fn print_usage() {
    println!("Usage: omarchy bar <command> [args...]");
    println!();
    println!("  use <id>                             Use a bar option as the active bar");
    println!("  reset                                Return to the built-in Omarchy bar");
    println!("  defaults                             Restore the default bar and service widgets");
    println!("  position <top|bottom|left|right>     Bar position");
    println!("  transparent <true|false|toggle>      Bar transparency");
    println!("  put <id> [placement]                 Put a widget on the bar");
    println!("  move <id> [placement]                Move a widget within or between sections");
    println!("  set <id> <key> <value> [--json] [placement]  Set a per-widget option");
}

fn bar_widget_default_section(id: &str) -> String {
    let catalog = run_output("omarchy-plugin-catalog", &[]).unwrap_or_default();
    if catalog.is_empty() { return "center".into(); }
    let filter = format!(
        r#"map(select(.id == "{id}"))[0].barWidget.defaultSection // "center" | if IN("left","center","right") then . else "center" end"#
    );
    jq_output(&catalog, &filter).unwrap_or_else(|| "center".into())
}

fn shell_put_bar_widget(id: &str, placement: &str) -> Option<String> {
    let attempts = std::env::var("OMARCHY_SHELL_READY_ATTEMPTS")
        .ok().and_then(|s| s.parse().ok()).unwrap_or(50usize);
    let absent_attempts = std::env::var("OMARCHY_SHELL_ABSENT_ATTEMPTS")
        .ok().and_then(|s| s.parse().ok()).unwrap_or(30usize);

    let mut shell_answered = false;
    let mut absent = 0usize;

    for _ in 0..attempts {
        let out = Command::new("omarchy-shell")
            .args(["shell", "putBarWidget", id, placement])
            .output();
        match out {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                if o.status.success() {
                    shell_answered = true;
                    if stdout != "not ready" {
                        return Some(stdout);
                    }
                } else if stdout.contains("not ready") || stderr.contains("not ready") {
                    shell_answered = true;
                } else if stdout.contains("is not running") || stderr.contains("is not running") {
                    if shell_answered {
                        fail(&format!("omarchy-shell did not become ready; {id} was not put on the bar"));
                    }
                    absent += 1;
                    if absent >= absent_attempts {
                        eprintln!("omarchy-shell is not running; {id} was not put on the bar");
                        return None;
                    }
                } else {
                    let msg = if !stdout.is_empty() { stdout } else { stderr.trim().to_string() };
                    fail(&format!("could not put {id} on the bar: {msg}"));
                }
            }
            Err(e) => {
                eprintln!("Failed to run omarchy-shell: {e}");
                return None;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    fail(&format!("omarchy-shell did not become ready; {id} was not put on the bar"));
}

// ─── bar subcommands ─────────────────────────────────────────────────────

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        print_usage();
        return 0;
    }

    match args[0].as_str() {
        "use" => cmd_use(&args[1..]),
        "reset" => {
            if args.len() > 1 { fail("reset does not take arguments"); }
            cmd_use(&["omarchy.bar".to_string()])
        }
        "defaults" => cmd_defaults(&args[1..]),
        "position" => cmd_position(&args[1..]),
        "transparent" => cmd_transparent(&args[1..]),
        "put" => cmd_put(&args[1..]),
        "move" => cmd_move(&args[1..]),
        "set" => cmd_set(&args[1..]),
        "-h" | "--help" | "help" => { print_usage(); 0 }
        other => { fail(&format!("unknown command: {other}")); }
    }
}

fn cmd_use(args: &[String]) -> i32 {
    let plugin = match args.first() {
        Some(p) => p,
        None => fail("bar option id is required"),
    };
    if args.len() != 1 { fail("use takes a single bar option id"); }

    let plugin = if plugin == "default" || plugin == "built-in" {
        "omarchy.bar"
    } else {
        plugin.as_str()
    };

    if !bar_option_exists(plugin) {
        fail(&format!("{plugin} is not a known bar option; run 'omarchy plugin list'"));
    }

    let program = if plugin == "omarchy.bar" {
        format!("{NORMALIZE} | del(.bar.id)")
    } else {
        format!(r#"{NORMALIZE} | .bar.id = $plugin"#)
    };

    let extra: Vec<&str> = if plugin == "omarchy.bar" {
        vec![]
    } else {
        vec!["--arg", "plugin", plugin]
    };

    commit(&program, &extra);
    println!("Using {plugin} as the active bar");
    0
}

fn cmd_defaults(args: &[String]) -> i32 {
    if !args.is_empty() { fail("defaults does not take arguments"); }

    let mut optional_widgets: Vec<String> = Vec::new();
    for service in &["dropbox", "tailscale"] {
        let cmd = format!("omarchy-installed-service-{service}");
        if Command::new(&cmd)
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status().map(|s| s.success()).unwrap_or(false)
        {
            let id = format!("omarchy.{service}");
            let section = bar_widget_default_section(&id);
            optional_widgets.push(format!(r#"{{"id":"{id}","section":"{section}"}}"#));
        }
    }

    let widgets_json = format!("[{}]", optional_widgets.join(","));
    let defaults_file = defaults_file();
    let program = format!(
        r#"{NORMALIZE}
    | .bar = $defaults[0].bar
    | def entry_id: if type == "object" then (.id // "" | tostring) else tostring end;
      def anchor_for($section): {{ left: "omarchy.workspaces", center: "omarchy.weather", right: "omarchy.tray" }}[$section];
      reduce $widgets[] as $widget (.;
        .bar.layout.left = (.bar.layout.left | map(select(entry_id != $widget.id)))
        | .bar.layout.center = (.bar.layout.center | map(select(entry_id != $widget.id)))
        | .bar.layout.right = (.bar.layout.right | map(select(entry_id != $widget.id)))
        | (.bar.layout[$widget.section] | map(entry_id) | index(anchor_for($widget.section))) as $anchor
        | ($anchor | if . == null then (.bar.layout[$widget.section] | length) else . + 1 end) as $index
        | .bar.layout[$widget.section] = (
            .bar.layout[$widget.section][0:$index]
            + [{{id: $widget.id}}]
            + .bar.layout[$widget.section][$index:]
          )
      )"#
    );

    commit(&program, &[
        "--slurpfile", "defaults", defaults_file.to_str().unwrap_or(""),
        "--argjson", "widgets", &widgets_json,
    ]);
    println!("Restored the default Omarchy bar");
    0
}

fn cmd_position(args: &[String]) -> i32 {
    let position = match args.first() {
        Some(p) => p.as_str(),
        None => fail("position is required"),
    };
    if args.len() != 1 { fail("position takes a single value"); }
    if !matches!(position, "top" | "bottom" | "left" | "right") {
        fail("position must be top, bottom, left, or right");
    }
    commit(&format!("{NORMALIZE} | .bar.position = $position"), &["--arg", "position", position]);
    println!("Bar position set to {position}");
    0
}

fn cmd_transparent(args: &[String]) -> i32 {
    let transparent = match args.first() {
        Some(t) => t.as_str(),
        None => fail("transparent is required"),
    };
    if args.len() != 1 { fail("transparent takes a single value"); }
    if !matches!(transparent, "true" | "false" | "toggle") {
        fail("transparent must be true, false, or toggle");
    }
    if transparent == "toggle" {
        commit(&format!("{NORMALIZE} | .bar.transparent = (.bar.transparent != true)"), &[]);
        println!("Bar transparency toggled");
    } else {
        commit(&format!("{NORMALIZE} | .bar.transparent = $transparent"), &["--argjson", "transparent", transparent]);
        println!("Bar transparency set to {transparent}");
    }
    0
}

fn cmd_put(args: &[String]) -> i32 {
    if args.is_empty() { fail("put requires a widget id"); }
    let id = &args[0];
    let rest = &args[1..];

    let mut positional_section = String::new();
    let mut placement_args: Vec<String> = Vec::new();
    let rest_vec = rest.to_vec();
    let mut i = 0;

    if !rest_vec.is_empty() && !rest_vec[0].starts_with("--") {
        positional_section = rest_vec[0].clone();
        validate_section(&positional_section);
        i = 1;
    }

    while i < rest_vec.len() {
        placement_args.push(rest_vec[i].clone());
        i += 1;
    }

    let mut p = Placement::default();
    p.parse(&placement_args);

    if !positional_section.is_empty() && !p.section.is_empty() {
        fail("specify a section positionally or with --section, not both");
    }
    if !positional_section.is_empty() { p.section = positional_section.clone(); }
    if !p.from_section.is_empty() || !p.from_index.is_empty() {
        fail("put does not accept --from-section or --from-index");
    }

    let placement = p.to_json();
    let result = shell_put_bar_widget(id, &placement);
    let result = match result {
        Some(r) => r,
        None => return 0,
    };

    // If could not find target widget, retry without before/after
    if result.starts_with("could not find target widget") {
        p.before.clear();
        p.after.clear();
        let placement2 = p.to_json();
        let result2 = shell_put_bar_widget(id, &placement2);
        let result2 = match result2 {
            Some(r) => r,
            None => return 0,
        };
        if result2 == "unknown" { fail(&format!("{id} is not a known widget; run 'omarchy plugin list'")); }
        if result2 != "ok" { fail(&result2); }
    } else {
        if result == "unknown" { fail(&format!("{id} is not a known widget; run 'omarchy plugin list'")); }
        if result != "ok" { fail(&result); }
    }

    println!("{id} is on the bar");
    0
}

fn cmd_move(args: &[String]) -> i32 {
    if args.is_empty() { fail("move requires a widget id"); }
    let id = &args[0];
    let rest = args[1..].to_vec();

    let mut positional_section = String::new();
    let mut placement_args: Vec<String> = Vec::new();
    let mut i = 0;

    if !rest.is_empty() && !rest[0].starts_with("--") {
        positional_section = rest[0].clone();
        validate_section(&positional_section);
        i = 1;
    }

    while i < rest.len() {
        placement_args.push(rest[i].clone());
        i += 1;
    }

    let mut p = Placement::default();
    p.parse(&placement_args);

    if !positional_section.is_empty() && !p.section.is_empty() {
        fail("specify a section positionally or with --section, not both");
    }
    if !positional_section.is_empty() && !p.index.is_empty() {
        fail("specify a section positionally or use --index, not both");
    }
    if !positional_section.is_empty() && !p.before.is_empty() {
        fail("specify a section positionally or use --before, not both");
    }
    if !positional_section.is_empty() && !p.after.is_empty() {
        fail("specify a section positionally or use --after, not both");
    }
    if !positional_section.is_empty() { p.section = positional_section; }

    let placement = p.to_json();
    let result = Command::new("omarchy-shell")
        .args(["shell", "moveBarWidget", id, &placement])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if result != "ok" { fail(&result); }
    println!("Moved {id}");
    0
}

fn cmd_set(args: &[String]) -> i32 {
    if args.len() < 3 {
        if args.is_empty() { fail("set requires a widget id"); }
        if args.len() == 1 { fail("set requires a setting key"); }
        fail("set requires a value");
    }
    let id = &args[0];
    let key = &args[1];
    let value = &args[2];
    let rest = args[3..].to_vec();

    let mut value_is_json = false;
    let mut placement_args: Vec<String> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--json" {
            value_is_json = true;
        } else {
            placement_args.push(rest[i].clone());
        }
        i += 1;
    }

    let mut p = Placement::default();
    p.parse(&placement_args);
    if !p.before.is_empty() || !p.after.is_empty() {
        fail("set does not accept --before or --after");
    }

    let value_json = if value_is_json {
        // Validate JSON
        let out = Command::new("jq")
            .args(["-cn", "--argjson", "value", value, "$value"])
            .output()
            .ok()
            .and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok() } else { None })
            .map(|s| s.trim().to_string());
        match out {
            Some(j) => j,
            None => fail(&format!("invalid JSON value: {value}")),
        }
    } else {
        let out = Command::new("jq")
            .args(["-cn", "--arg", "value", value, "$value"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| format!(r#""{value}""#));
        out
    };

    let placement = p.to_json();
    let result = Command::new("omarchy-shell")
        .args(["shell", "setBarWidget", id, key, &value_json, &placement])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if result != "ok" { fail(&result); }
    println!("Set {key} on {id}");
    0
}

// ─── bar-text-color ───────────────────────────────────────────────────────

pub fn text_color(
    position: &str,
    bar_size: &str,
    text_color: &str,
    background_color: &str,
    background_path: Option<&str>,
    screen_size: Option<&str>,
) -> i32 {
    let fallback = || {
        println!("{text_color}");
    };

    let valid_hex = |s: &str| -> bool {
        s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
    };

    if !valid_hex(text_color) { fallback(); return 0; }
    if !valid_hex(background_color) { fallback(); return 0; }
    if !matches!(position, "top" | "bottom" | "left" | "right") { fallback(); return 0; }
    let bar_size_num: u32 = match bar_size.parse() {
        Ok(n) => n,
        Err(_) => { fallback(); return 0; }
    };

    if !Command::new("omarchy-cmd-present")
        .arg("magick")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false)
    {
        fallback();
        return 0;
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let bg_path = background_path
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .unwrap_or_else(|| {
            std::fs::canonicalize(format!("{home}/.local/state/omarchy/current/background"))
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default()
        });

    if bg_path.is_empty() || !std::path::Path::new(&bg_path).is_file() {
        fallback();
        return 0;
    }

    let screen = screen_size.unwrap_or("").to_string();
    let screen = if screen.is_empty() {
        Command::new("hyprctl")
            .args(["monitors", "-j"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                let mut child = Command::new("jq")
                    .args(["-r", r#".[0] | "\(.width)x\(.height)""#])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .ok()?;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(s.as_bytes());
                }
                child.wait_with_output().ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_default()
    } else {
        screen
    };

    let parts: Vec<&str> = screen.split('x').collect();
    if parts.len() != 2 { fallback(); return 0; }
    let screen_width: u32 = match parts[0].parse() {
        Ok(n) => n, Err(_) => { fallback(); return 0; }
    };
    let screen_height: u32 = match parts[1].parse() {
        Ok(n) => n, Err(_) => { fallback(); return 0; }
    };
    if screen_width == 0 || screen_height == 0 || bar_size_num == 0 { fallback(); return 0; }

    let crop = match position {
        "top" => format!("{screen_width}x{bar_size_num}+0+0"),
        "bottom" => {
            let y = screen_height.saturating_sub(bar_size_num);
            format!("{screen_width}x{bar_size_num}+0+{y}")
        }
        "left" => format!("{bar_size_num}x{screen_height}+0+0"),
        "right" => {
            let x = screen_width.saturating_sub(bar_size_num);
            format!("{bar_size_num}x{screen_height}+{x}+0")
        }
        _ => { fallback(); return 0; }
    };

    let magick_out = Command::new("magick")
        .args([
            &format!("{bg_path}[0]"),
            "-auto-orient",
            "-resize", &format!("{screen_width}x{screen_height}^"),
            "-gravity", "center",
            "-extent", &format!("{screen_width}x{screen_height}"),
            "-gravity", "NorthWest",
            "-crop", &crop,
            "+repage",
            "-resize", "1x1!",
            "-format", "%[fx:int(255*r)],%[fx:int(255*g)],%[fx:int(255*b)]",
            "info:-",
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let rgb: Vec<u32> = magick_out.split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if rgb.len() != 3 { fallback(); return 0; }

    let sample = format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]);

    let text_contrast = contrast(text_color, &sample);
    let bg_contrast = contrast(background_color, &sample);

    if bg_contrast > text_contrast {
        println!("{background_color}");
    } else {
        println!("{text_color}");
    }
    0
}

fn luminance(hex: &str) -> f64 {
    if hex.len() < 7 { return 0.0; }
    let r = u32::from_str_radix(&hex[1..3], 16).unwrap_or(0) as f64 / 255.0;
    let g = u32::from_str_radix(&hex[3..5], 16).unwrap_or(0) as f64 / 255.0;
    let b = u32::from_str_radix(&hex[5..7], 16).unwrap_or(0) as f64 / 255.0;
    let linear = |c: f64| -> f64 {
        if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

fn contrast(fg: &str, bg: &str) -> f64 {
    let l1 = luminance(fg);
    let l2 = luminance(bg);
    let (l1, l2) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (l1 + 0.05) / (l2 + 0.05)
}
