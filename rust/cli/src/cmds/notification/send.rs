use std::process::{Command, Stdio};

pub fn run(args: &[String]) -> i32 {
    // Parse arguments matching the bash script
    let mut headline = String::new();
    let mut description = String::new();
    let mut glyph: Option<String> = None;
    let mut urgency = "low".to_string();
    let mut app_name = "omarchy-action".to_string();
    let mut app_icon = String::new();
    let mut image: Option<String> = None;
    let mut expire_timeout: i64 = -1;
    let mut replaces_id: u64 = 0;
    let mut print_id = false;
    let mut exec_args: Vec<String> = Vec::new();
    let mut exec_present = false;

    let mut i = 0;
    let n = args.len();

    // Parse leading options
    while i < n {
        let arg = &args[i];
        if arg == "-p" || arg == "--print-id" {
            print_id = true;
            i += 1;
            continue;
        }
        if let Some(val) = arg.strip_prefix("--") {
            // Try --key=value form
            if let Some((key, val)) = val.split_once('=') {
                match format!("--{key}").as_str() {
                    "--glyph" | "-g" => { glyph = Some(val.to_string()); i += 1; continue; }
                    "--urgency" => { urgency = val.to_string(); i += 1; continue; }
                    "--app-name" => { app_name = val.to_string(); i += 1; continue; }
                    "--icon" => { app_icon = val.to_string(); i += 1; continue; }
                    "--image" => { image = Some(val.to_string()); i += 1; continue; }
                    "--replace-id" => { replaces_id = val.parse().unwrap_or(0); i += 1; continue; }
                    "--expire-time" => { expire_timeout = val.parse().unwrap_or(-1); i += 1; continue; }
                    _ => {}
                }
            }
        }
        // Try two-arg forms
        match arg.as_str() {
            "-g" | "--glyph" => {
                if i + 1 < n { glyph = Some(args[i + 1].clone()); i += 2; continue; }
            }
            "-u" | "--urgency" => {
                if i + 1 < n { urgency = args[i + 1].clone(); i += 2; continue; }
            }
            "--app-name" => {
                if i + 1 < n { app_name = args[i + 1].clone(); i += 2; continue; }
            }
            "-i" | "--icon" => {
                if i + 1 < n { app_icon = args[i + 1].clone(); i += 2; continue; }
            }
            "--image" => {
                if i + 1 < n { image = Some(args[i + 1].clone()); i += 2; continue; }
            }
            "-r" | "--replace-id" => {
                if i + 1 < n { replaces_id = args[i + 1].parse().unwrap_or(0); i += 2; continue; }
            }
            "-t" | "--expire-time" => {
                if i + 1 < n { expire_timeout = args[i + 1].parse().unwrap_or(-1); i += 2; continue; }
            }
            _ => break,
        }
    }

    if i >= n {
        eprintln!("Usage: omarchy-notification-send [options] <headline> [description] [--exec <cmd> [args...]]");
        return 1;
    }

    headline = args[i].clone();
    i += 1;

    // Check if next arg is description (not a known flag and not --exec)
    if i < n && !is_known_flag(&args[i]) && args[i] != "--exec" {
        description = args[i].clone();
        i += 1;
    }

    // Parse remaining options and --exec
    while i < n {
        if args[i] == "--exec" {
            i += 1;
            exec_args = args[i..].to_vec();
            exec_present = true;
            break;
        }
        // parse remaining flags
        match args[i].as_str() {
            "-p" | "--print-id" => { print_id = true; i += 1; }
            "-g" | "--glyph" => { if i + 1 < n { glyph = Some(args[i + 1].clone()); i += 2; } else { i += 1; } }
            "-u" | "--urgency" => { if i + 1 < n { urgency = args[i + 1].clone(); i += 2; } else { i += 1; } }
            "--app-name" => { if i + 1 < n { app_name = args[i + 1].clone(); i += 2; } else { i += 1; } }
            "-i" | "--icon" => { if i + 1 < n { app_icon = args[i + 1].clone(); i += 2; } else { i += 1; } }
            "--image" => { if i + 1 < n { image = Some(args[i + 1].clone()); i += 2; } else { i += 1; } }
            "-r" | "--replace-id" => { if i + 1 < n { replaces_id = args[i + 1].parse().unwrap_or(0); i += 2; } else { i += 1; } }
            "-t" | "--expire-time" => { if i + 1 < n { expire_timeout = args[i + 1].parse().unwrap_or(-1); i += 2; } else { i += 1; } }
            other => {
                eprintln!("Unknown option: {other}");
                return 1;
            }
        }
    }

    let urgency_byte: u8 = match urgency.as_str() {
        "low" => 0,
        "normal" => 1,
        "critical" => 2,
        other => {
            eprintln!("Unknown urgency: {other} (use low, normal, or critical)");
            return 1;
        }
    };

    // Build hints: (key, type, value) triples
    let mut hints: Vec<String> = vec![
        "urgency".to_string(), "y".to_string(), urgency_byte.to_string(),
    ];

    if let Some(ref g) = glyph {
        hints.push("omarchy-glyph".to_string());
        hints.push("s".to_string());
        hints.push(g.clone());
    }

    if let Some(ref img) = image {
        hints.push("image-path".to_string());
        hints.push("s".to_string());
        hints.push(img.clone());
    }

    if exec_present {
        if exec_args.is_empty() || exec_args[0].is_empty() {
            eprintln!("--exec needs a command: --exec <program> [args...]");
            return 1;
        }
        if exec_args.len() == 1 && exec_args[0].contains(char::is_whitespace) {
            eprintln!("--exec takes the command as separate words, not one quoted string.");
            return 1;
        }
        // Build JSON array of exec args
        let exec_json = serde_json::to_string(&exec_args).unwrap_or_else(|_| "[]".to_string());
        hints.push("omarchy-exec-argv".to_string());
        hints.push("s".to_string());
        hints.push(exec_json);
    }

    let hint_count = hints.len() / 3;

    // Build busctl command
    // Signature: susssasa{sv}i
    let mut cmd_args: Vec<String> = vec![
        "--user".to_string(),
        "--".to_string(),
        "call".to_string(),
        "org.freedesktop.Notifications".to_string(),
        "/org/freedesktop/Notifications".to_string(),
        "org.freedesktop.Notifications".to_string(),
        "Notify".to_string(),
        "susssasa{sv}i".to_string(),
        app_name.clone(),
        replaces_id.to_string(),
        app_icon.clone(),
        headline.clone(),
        description.clone(),
        "0".to_string(),
        hint_count.to_string(),
    ];
    cmd_args.extend(hints);
    cmd_args.push(expire_timeout.to_string());

    if print_id {
        let out = Command::new("busctl")
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match out {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                // busctl prints "u <id>", emit just the id
                let id = stdout.trim().split_whitespace().last().unwrap_or("").to_string();
                println!("{id}");
                if o.status.success() { 0 } else { 1 }
            }
            Err(e) => { eprintln!("busctl error: {e}"); 1 }
        }
    } else {
        let status = Command::new("busctl")
            .args(&cmd_args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
    }
}

fn is_known_flag(s: &str) -> bool {
    matches!(s,
        "-g" | "--glyph" | "-u" | "--urgency" | "--app-name" | "-i" | "--icon" |
        "-t" | "--expire-time" | "--image" | "-r" | "--replace-id" | "-p" | "--print-id" | "--exec"
    ) || s.starts_with("--glyph=") || s.starts_with("--urgency=") || s.starts_with("--app-name=")
        || s.starts_with("--icon=") || s.starts_with("--expire-time=") || s.starts_with("--image=")
        || s.starts_with("--replace-id=")
}
