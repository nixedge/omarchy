use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ─── helpers ───────────────────────────────────────────────────────────────

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

fn omarchy_run(args: &[&str]) -> Option<String> {
    let first = args[0];
    Command::new(first)
        .args(&args[1..])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

// ─── audio output-volume ──────────────────────────────────────────────────

pub fn output_volume(action: &str) -> i32 {
    // Resolve sink
    let sink = match omarchy_run(&["omarchy-audio-output-sink"]) {
        Some(s) if !s.is_empty() => s,
        _ => {
            eprintln!("Could not resolve an audio sink to control.");
            return 1;
        }
    };

    let action = match action {
        "raise" => "+5",
        "lower" => "-5",
        other => other,
    };

    if action == "mute-toggle" {
        // Debounce 250ms
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
        let debounce_file = PathBuf::from(&runtime_dir)
            .join("omarchy-audio-output-volume-mute-toggle.last");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let last: u64 = fs::read_to_string(&debounce_file)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        if now.saturating_sub(last) < 250 {
            return 0;
        }
        let _ = fs::write(&debounce_file, format!("{now}\n"));
        let _ = Command::new("pactl")
            .args(["set-sink-mute", &sink, "toggle"])
            .status();
    } else if let Some(rest) = action.strip_prefix('+') {
        let step: u32 = rest.parse().unwrap_or(5);
        let current = volume_percent(&sink).unwrap_or(0);
        let next = (current + step).min(100);
        let _ = Command::new("pactl").args(["set-sink-mute", &sink, "0"]).status();
        let _ = Command::new("pactl")
            .args(["set-sink-volume", &sink, &format!("{next}%")])
            .status();
    } else if let Some(rest) = action.strip_prefix('-') {
        let step: u32 = rest.parse().unwrap_or(5);
        let current = volume_percent(&sink).unwrap_or(0);
        let next = current.saturating_sub(step);
        let _ = Command::new("pactl").args(["set-sink-mute", &sink, "0"]).status();
        let _ = Command::new("pactl")
            .args(["set-sink-volume", &sink, &format!("{next}%")])
            .status();
    } else {
        eprintln!("Unknown volume action: {action}");
        return 1;
    }

    let percent = volume_percent(&sink).unwrap_or(0);
    let muted = sink_muted(&sink);
    let icon = if muted || percent == 0 { "volume-muted" } else { "volume-high" };

    let _ = Command::new("omarchy-osd")
        .args(["-i", icon, "-p", &percent.to_string()])
        .status();
    0
}

fn volume_percent(sink: &str) -> Option<u32> {
    let out = run_output("pactl", &["get-sink-volume", sink])?;
    out.split_whitespace()
        .find(|t| t.ends_with('%'))
        .and_then(|t| t.trim_end_matches('%').parse().ok())
}

fn sink_muted(sink: &str) -> bool {
    run_output("pactl", &["get-sink-mute", sink])
        .map(|s| s.contains("yes"))
        .unwrap_or(false)
}

// ─── audio output-switch ──────────────────────────────────────────────────

pub fn output_switch() -> i32 {
    let fronted = Command::new("omarchy-audio-tuning")
        .args(["fronted-sink"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Build jq filter using pactl json output
    let pactl_out = Command::new("pactl")
        .args(["-f", "json", "list", "sinks"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let sinks_json = match pactl_out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => {
            let _ = Command::new("omarchy-osd").args(["-m", "No audio devices found"]).status();
            return 1;
        }
    };

    // Filter sinks via jq
    let jq_filter = format!(
        r#"[.[] | select((.ports | length == 0) or ([.ports[]? | .availability != "not available"] | any)) | select("{fronted}" == "" or .name != "{fronted}")]"#
    );
    let sinks = jq_output(&sinks_json, &jq_filter);
    let sinks = match sinks {
        Some(s) => s,
        None => {
            let _ = Command::new("omarchy-osd").args(["-m", "No audio devices found"]).status();
            return 1;
        }
    };

    let sinks_count: i64 = jq_scalar_output(&sinks, "length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if sinks_count == 0 {
        let _ = Command::new("omarchy-osd").args(["-m", "No audio devices found"]).status();
        return 1;
    }

    let current_sink_name = Command::new("pactl")
        .arg("get-default-sink")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let current_index_str = jq_scalar_output(
        &sinks,
        &format!(r#"map(.name) | index("{current_sink_name}")"#),
    )
    .unwrap_or_else(|| "null".into());

    let next_index = if current_index_str == "null" {
        0
    } else {
        let idx: i64 = current_index_str.parse().unwrap_or(0);
        ((idx + 1) % sinks_count) as usize
    };

    let next_sink = jq_output(&sinks, &format!(".[{next_index}]")).unwrap_or_default();
    let next_sink_name = jq_scalar_output(&next_sink, ".name").unwrap_or_default();
    let next_sink_description = jq_scalar_output(
        &next_sink,
        r#".description // .properties."device.description" // .name"#,
    )
    .unwrap_or_else(|| next_sink_name.clone());
    let next_sink_index_val = jq_scalar_output(&next_sink, ".index").unwrap_or_default();

    // Get effective sink for volume reading
    let next_sink_effective = Command::new("omarchy-audio-output-sink")
        .arg(&next_sink_name)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| next_sink_name.clone());

    let next_volume = volume_percent(&next_sink_effective).unwrap_or_else(|| {
        jq_scalar_output(
            &next_sink,
            r#".volume | to_entries[0].value.value_percent | sub("%"; "") | tonumber"#,
        )
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
    });

    let next_muted = sink_muted(&next_sink_effective);

    let icon_state = if next_muted || next_volume == 0 {
        "muted"
    } else if next_volume <= 33 {
        "low"
    } else if next_volume <= 66 {
        "medium"
    } else {
        "high"
    };

    if next_sink_name != current_sink_name {
        let _ = Command::new("omarchy-audio-output-set-default")
            .args([&next_sink_index_val, &next_sink_name])
            .status();
    }

    let _ = Command::new("omarchy-osd")
        .args(["-i", &format!("volume-{icon_state}"), "-m", &next_sink_description])
        .status();
    0
}

fn jq_output(input: &str, filter: &str) -> Option<String> {
    let out = Command::new("jq")
        .args(["-c", filter])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    use std::io::Write;
    let mut child = out;
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        let _ = stdin.write_all(input.as_bytes());
    }
    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn jq_scalar_output(input: &str, filter: &str) -> Option<String> {
    let out = Command::new("jq")
        .args(["-r", filter])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    use std::io::Write;
    let mut child = out;
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        let _ = stdin.write_all(input.as_bytes());
    }
    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        let s = String::from_utf8(output.stdout).ok()?.trim().to_string();
        Some(s)
    } else {
        None
    }
}

// ─── audio input-mute ────────────────────────────────────────────────────

pub fn input_mute() -> i32 {
    let _ = Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
        .stdout(Stdio::null())
        .status();

    let out = run_output("wpctl", &["get-volume", "@DEFAULT_AUDIO_SOURCE@"]).unwrap_or_default();
    if out.contains("MUTED") {
        let _ = Command::new("omarchy-brightness-keyboard-mute").arg("on").status();
        let _ = Command::new("omarchy-osd")
            .args(["-i", "microphone-muted", "-m", "Microphone muted"])
            .status();
    } else {
        let _ = Command::new("omarchy-brightness-keyboard-mute").arg("off").status();
        let _ = Command::new("omarchy-osd")
            .args(["-i", "microphone", "-m", "Microphone on"])
            .status();
    }
    0
}

// ─── audio output-sink ───────────────────────────────────────────────────

pub fn output_sink(sink_arg: Option<&str>) -> i32 {
    let sink = match sink_arg {
        Some(s) => s.to_string(),
        None => Command::new("pactl")
            .args(["get-default-sink"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default(),
    };

    if sink.is_empty() || sink.starts_with("alsa_output.") {
        println!("{sink}");
        return 0;
    }

    // Try to resolve DSP sink downstream
    let list_out = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let downstream = resolve_downstream_sink(&list_out, &sink);

    if let Some(target_id) = downstream {
        // Get the name of the sink with this id
        let short_out = Command::new("pactl")
            .args(["list", "sinks", "short"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        for line in short_out.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[0] == target_id {
                println!("{}", parts[1]);
                return 0;
            }
        }
    }

    // Fall back to the sink itself
    println!("{sink}");
    0
}

fn resolve_downstream_sink(list_out: &str, virt: &str) -> Option<String> {
    // Parse pactl list sink-inputs output to find the sink target of a virtual sink
    let mut current_target: Option<String> = None;
    let mut in_sink_input = false;

    for line in list_out.lines() {
        if line.starts_with("Sink Input #") {
            in_sink_input = true;
            current_target = None;
        }
        if !in_sink_input {
            continue;
        }
        if let Some(rest) = line.trim().strip_prefix("Sink:") {
            current_target = Some(rest.trim().to_string());
        }
        if line.contains("node.name = ") {
            // Extract name between quotes
            if let Some(start) = line.find("node.name = \"") {
                let after = &line[start + 13..];
                if let Some(end) = after.find('"') {
                    let name = &after[..end];
                    if name.starts_with(virt) {
                        if let Some(ref t) = current_target {
                            return Some(t.clone());
                        }
                    }
                }
            }
        }
        if line.contains("application.name = \"EasyEffects\"") {
            if virt == "easyeffects_sink" {
                if let Some(ref t) = current_target {
                    return Some(t.clone());
                }
            }
        }
    }
    None
}

// ─── audio sink-availability ─────────────────────────────────────────────

pub fn sink_availability() -> i32 {
    let fronted = Command::new("omarchy-audio-tuning")
        .args(["fronted-sink"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let list_out = Command::new("pactl")
        .args(["list", "sinks"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    print_sink_availability(&list_out, &fronted);
    0
}

fn print_sink_availability(list_out: &str, fronted: &str) {
    let mut name = String::new();
    let mut in_ports = false;
    let mut port_count = 0u32;
    let mut available = false;

    let emit = |name: &str, port_count: u32, available: bool, fronted: &str| {
        if name.is_empty() {
            return;
        }
        if !fronted.is_empty() && name == fronted {
            println!("{name}\t0");
            return;
        }
        let avail = if port_count == 0 || available { 1 } else { 0 };
        println!("{name}\t{avail}");
    };

    for line in list_out.lines() {
        if line.starts_with("Sink #") {
            emit(&name, port_count, available, fronted);
            name.clear();
            in_ports = false;
            port_count = 0;
            available = false;
            continue;
        }
        if let Some(rest) = line.trim_start().strip_prefix("Name:") {
            name = rest.trim().to_string();
            continue;
        }
        if line.trim_start() == "Ports:" {
            in_ports = true;
            continue;
        }
        if in_ports && line.trim_start().starts_with("Active Port:") {
            in_ports = false;
            continue;
        }
        if in_ports && line.starts_with("\t\t") {
            port_count += 1;
            if !line.contains("not available") {
                available = true;
            }
        }
    }
    emit(&name, port_count, available, fronted);
}

// ─── audio source-switch ─────────────────────────────────────────────────

pub fn source_switch(direction: Option<&str>) -> i32 {
    let dir = direction.unwrap_or("next");
    match dir {
        "next" => {
            let status = Command::new("omarchy-shell")
                .args(["media", "sourceSwitch"])
                .status();
            if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
        }
        "previous" => {
            let status = Command::new("omarchy-shell")
                .args(["media", "sourceSwitchPrevious"])
                .status();
            if status.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
        }
        _ => {
            eprintln!("Usage: omarchy-audio-source-switch [next|previous]");
            1
        }
    }
}

// ─── audio input-set-default ─────────────────────────────────────────────

pub fn input_set_default(node_id: &str, source_name: &str) -> i32 {
    let _ = Command::new("wpctl")
        .args(["set-default", node_id])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("pactl")
        .args(["set-default-source", source_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // Move all source outputs
    let list_out = Command::new("pactl")
        .args(["list", "short", "source-outputs"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    for line in list_out.lines() {
        if let Some(id) = line.split_whitespace().next() {
            if !id.is_empty() {
                let _ = Command::new("pactl")
                    .args(["move-source-output", id, source_name])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
    }
    0
}

// ─── audio output-set-default ────────────────────────────────────────────

pub fn output_set_default(node_id: &str, sink_name: &str) -> i32 {
    let _ = Command::new("wpctl")
        .args(["set-default", node_id])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("pactl")
        .args(["set-default-sink", sink_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // Move real application streams (skipping EasyEffects and DSP chains)
    let list_out = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut current_id = String::new();
    for line in list_out.lines() {
        if line.starts_with("Sink Input #") {
            current_id = line.trim_start_matches("Sink Input #").to_string();
        }
        if line.contains("application.name = ") {
            if let Some(start) = line.find("application.name = \"") {
                let after = &line[start + 20..];
                if let Some(end) = after.find('"') {
                    let app = &after[..end];
                    if app != "EasyEffects" && !current_id.is_empty() {
                        let _ = Command::new("pactl")
                            .args(["move-sink-input", &current_id, sink_name])
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status();
                    }
                }
            }
        }
    }
    0
}

// ─── audio tuning ────────────────────────────────────────────────────────

pub fn tuning(action: &str, force: bool) -> i32 {
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let tunings_dir = format!("{omarchy_path}/default/audio/tunings");
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{home}/.config", home = std::env::var("HOME").unwrap_or_default()));
    let home = std::env::var("HOME").unwrap_or_default();

    let host_config_name = "omarchy-speaker-tuning.conf";
    let host_config = format!("{config_home}/pipewire/{host_config_name}");
    let host_source = format!("{omarchy_path}/default/audio/filter-chain-host.conf");
    let fragment = format!("{config_home}/pipewire/{host_config_name}.d/90-tuning.conf");
    let unit_name = "omarchy-speaker-tuning.service";
    let unit = format!("{config_home}/systemd/user/{unit_name}");
    let unit_source = format!("{omarchy_path}/default/systemd/user/{unit_name}");
    let sink_name = "omarchy_speaker_tuning";
    let stale_daemon = format!("{config_home}/pipewire/pipewire.conf.d/90-omarchy-speaker-tuning.conf");
    let stale_wireplumber = format!("{config_home}/wireplumber/wireplumber.conf.d/90-omarchy-speaker-tuning.conf");
    let stale_shared = format!("{config_home}/pipewire/filter-chain.conf.d/90-omarchy-speaker-tuning.conf");

    match action {
        "match" => {
            match tuning_match(&tunings_dir) {
                Some(dir) => { println!("{dir}"); 0 }
                None => 1,
            }
        }

        "fronted-sink" => {
            if !tuning_present(sink_name) {
                return 1;
            }
            match tuned_hardware_sink(&tunings_dir) {
                Some(s) => { println!("{s}"); 0 }
                None => 1,
            }
        }

        "status" => {
            if std::path::Path::new(&fragment).exists() {
                println!("Installed:    yes ({fragment})");
            } else {
                println!("Installed:    no");
            }
            let host_state = run_output("systemctl", &["--user", "is-active", unit_name])
                .unwrap_or_else(|| "inactive".into());
            let host_enabled = run_output("systemctl", &["--user", "is-enabled", unit_name])
                .unwrap_or_else(|| "disabled".into());
            println!("Host service: {host_state} ({host_enabled})");
            if tuning_present(sink_name) {
                println!("Tuning sink:  present");
            } else {
                println!("Tuning sink:  absent");
            }
            let default_sink = run_output("pactl", &["get-default-sink"]).unwrap_or_default();
            println!("Default sink: {default_sink}");
            if let Some(dir) = tuning_match(&tunings_dir) {
                let conf = parse_tuning_conf(&format!("{dir}/tuning.conf"));
                let description = conf.get("description").cloned().unwrap_or_else(|| "?".into());
                let basename = std::path::Path::new(&dir)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&dir)
                    .to_string();
                println!("Matches:      {description} ({basename})");
            } else {
                println!("Matches:      nothing ships for this laptop");
            }
            0
        }

        "off" => {
            let fragment_exists = std::path::Path::new(&fragment).exists();
            let unit_exists = std::path::Path::new(&unit).exists();
            let stale_d = std::path::Path::new(&stale_daemon).exists();
            let stale_wp = std::path::Path::new(&stale_wireplumber).exists();
            let stale_sh = std::path::Path::new(&stale_shared).exists();

            if !fragment_exists && !unit_exists && !stale_d && !stale_wp && !stale_sh {
                println!("No speaker tuning installed.");
                return 0;
            }

            let speakers = tuned_hardware_sink(&tunings_dir).unwrap_or_default();

            let _ = Command::new("systemctl")
                .args(["--user", "disable", "--now", unit_name])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
            let _ = fs::remove_file(&fragment);
            let _ = fs::remove_file(&host_config);
            let _ = fs::remove_file(&unit);
            let _ = fs::remove_file(&stale_shared);
            let _ = fs::remove_dir(format!("{config_home}/pipewire/{host_config_name}.d"));
            let _ = Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
            drop_stale_daemon_config(&stale_daemon, &stale_wireplumber);

            // Wait for tuning sink to disappear
            for _ in 0..20 {
                if !tuning_present(sink_name) { break; }
                std::thread::sleep(Duration::from_millis(250));
            }

            if !speakers.is_empty() {
                let _ = Command::new("pactl")
                    .args(["set-default-sink", &speakers])
                    .stdout(Stdio::null()).stderr(Stdio::null()).status();
                move_apps_to(&speakers);
            }
            println!("Speaker tuning removed.");
            0
        }

        "on" => {
            if !std::path::Path::new(&tunings_dir).exists() {
                eprintln!("No tunings shipped at {tunings_dir}");
                return 1;
            }

            let selected = match tuning_match(&tunings_dir) {
                Some(s) => s,
                None => {
                    println!("No speaker tuning matches this laptop.");
                    return 0;
                }
            };

            let conf = parse_tuning_conf(&format!("{selected}/tuning.conf"));
            let sink_pattern = match conf.get("sink_pattern") {
                Some(p) => p.clone(),
                None => {
                    eprintln!("tuning.conf missing sink_pattern");
                    return 1;
                }
            };
            let description = conf.get("description").cloned().unwrap_or_else(|| "?".into());

            // Wait for speaker sink to appear
            let mut speaker_sink = String::new();
            for _ in 0..20 {
                speaker_sink = sink_matching(&sink_pattern);
                if !speaker_sink.is_empty() { break; }
                std::thread::sleep(Duration::from_millis(500));
            }

            if speaker_sink.is_empty() {
                eprintln!("A tuning applies to this laptop but no sink matching {sink_pattern}");
                eprintln!("is present, so there is no audio server yet. Re-run after login:");
                eprintln!("  omarchy audio tuning on");
                return 1;
            }

            // Check EasyEffects
            if easyeffects_running() {
                eprintln!("EasyEffects is running. It moves any stream that follows the default sink to its");
                eprintln!("own sink, so a tuning installed now would be bypassed.");
                eprintln!();
                eprintln!("Stop it first:  systemctl --user disable --now easyeffects.service");
                return 1;
            }

            // Check LV2 plugin
            if !std::path::Path::new("/usr/lib/lv2/lsp-plugins.lv2/limiter_stereo.ttl").exists() {
                eprintln!("lsp-plugins-lv2 is required for the tuning limiter.");
                return 1;
            }

            // Render filter-chain.conf
            let filter_chain_src = format!("{selected}/filter-chain.conf");
            let filter_chain_content = match fs::read_to_string(&filter_chain_src) {
                Ok(c) => c.replace("@SPEAKER_SINK@", &speaker_sink),
                Err(e) => {
                    eprintln!("Cannot read {filter_chain_src}: {e}");
                    return 1;
                }
            };

            // Check if already current (unless --force)
            if !force
                && std::path::Path::new(&fragment).exists()
                && fs::read_to_string(&fragment).ok().as_deref() == Some(&filter_chain_content)
                && std::path::Path::new(&host_config).exists()
                && files_equal(&host_source, &host_config)
                && std::path::Path::new(&unit).exists()
                && files_equal(&unit_source, &unit)
                && run_ok("systemctl", &["--user", "is-active", "--quiet", unit_name])
                && run_ok("systemctl", &["--user", "is-enabled", "--quiet", unit_name])
                && tuning_downstream_sink(sink_name) == speaker_sink
            {
                println!("Speaker tuning already current: {description}");
                return 0;
            }

            drop_stale_daemon_config(&stale_daemon, &stale_wireplumber);
            let _ = fs::remove_file(&stale_shared);

            // Install files
            if let Err(e) = install_file(&host_source, &host_config, 0o644) {
                eprintln!("Cannot install {host_config}: {e}");
                return 1;
            }
            let fragment_dir = std::path::Path::new(&fragment).parent().unwrap();
            let _ = fs::create_dir_all(fragment_dir);
            if let Err(e) = fs::write(&fragment, &filter_chain_content) {
                eprintln!("Cannot write {fragment}: {e}");
                return 1;
            }
            if let Err(e) = install_file(&unit_source, &unit, 0o644) {
                eprintln!("Cannot install {unit}: {e}");
                return 1;
            }

            let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
            let _ = Command::new("systemctl").args(["--user", "enable", unit_name]).stdout(Stdio::null()).stderr(Stdio::null()).status();
            let _ = Command::new("systemctl").args(["--user", "restart", unit_name]).stdout(Stdio::null()).stderr(Stdio::null()).status();
            println!("Installed speaker tuning: {description}");

            // Wait for tuning sink
            for _ in 0..40 {
                if tuning_present(sink_name) { break; }
                std::thread::sleep(Duration::from_millis(250));
            }
            if !tuning_present(sink_name) {
                let _ = Command::new("systemctl").args(["--user", "disable", "--now", unit_name]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = fs::remove_file(&fragment);
                let _ = fs::remove_file(&host_config);
                let _ = fs::remove_file(&unit);
                let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                eprintln!("Tuning sink never appeared, so it was removed. Audio is untouched.");
                eprintln!("Check: systemctl --user status {unit_name}");
                return 1;
            }

            // Confirm downstream
            for _ in 0..20 {
                if tuning_downstream_sink(sink_name) == speaker_sink { break; }
                std::thread::sleep(Duration::from_millis(250));
            }
            let downstream = tuning_downstream_sink(sink_name);
            if downstream != speaker_sink {
                let _ = Command::new("systemctl").args(["--user", "disable", "--now", unit_name]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = fs::remove_file(&fragment);
                let _ = fs::remove_file(&host_config);
                let _ = fs::remove_file(&unit);
                let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                eprintln!("The tuning output linked to {} instead of", if downstream.is_empty() { "nothing".into() } else { downstream });
                eprintln!("{speaker_sink}, so it was removed rather than left tuning the wrong");
                eprintln!("device. Audio is untouched.");
                return 1;
            }

            let _ = Command::new("pactl").args(["set-default-sink", sink_name]).stdout(Stdio::null()).stderr(Stdio::null()).status();
            move_apps_to(sink_name);
            println!("Speakers now play through the tuning.");
            0
        }

        _ => {
            eprintln!("Usage: omarchy-audio-tuning <on|off|status|match|fronted-sink> [--force]");
            2
        }
    }
}

// ─── tuning helpers ────────────────────────────────────────────────────────

fn parse_tuning_conf(path: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let content = fs::read_to_string(path).unwrap_or_default();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        // Handle array assignments like: match_sku=(val1 val2)
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let val_raw = line[eq_pos + 1..].trim();
            // Strip parentheses for arrays, strip quotes for strings
            let val = if val_raw.starts_with('(') {
                val_raw.trim_matches('(').trim_matches(')').trim_matches('"').to_string()
            } else {
                val_raw.trim_matches('"').to_string()
            };
            map.insert(key, val);
        }
    }
    map
}

fn tuning_match(tunings_dir: &str) -> Option<String> {
    let dir = std::path::Path::new(tunings_dir);
    if !dir.exists() {
        return None;
    }

    let entries = fs::read_dir(dir).ok()?;
    let mut dirs: Vec<_> = entries.flatten().collect();
    dirs.sort_by_key(|e| e.file_name());

    for entry in dirs {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let conf_path = path.join("tuning.conf");
        if !conf_path.exists() { continue; }

        let conf = parse_tuning_conf(conf_path.to_str().unwrap_or(""));

        let matched = if let Some(cmd) = conf.get("match_command") {
            if cmd.is_empty() {
                false
            } else {
                run_ok(cmd, &[])
            }
        } else if let Some(sku_val) = conf.get("match_sku") {
            sku_matches(sku_val)
        } else if let Some(dmi_val) = conf.get("match_dmi") {
            dmi_matches(dmi_val)
        } else {
            false
        };

        if !matched { continue; }
        if conf.get("sink_pattern").map(|s| s.is_empty()).unwrap_or(true) { continue; }

        return Some(path.to_string_lossy().into_owned());
    }
    None
}

fn sku_matches(want_raw: &str) -> bool {
    let sku = fs::read_to_string("/sys/class/dmi/id/product_sku")
        .unwrap_or_default();
    let sku = sku.trim().to_lowercase();
    if sku.is_empty() { return false; }
    for want in want_raw.split_whitespace() {
        if sku == want.to_lowercase() { return true; }
    }
    false
}

fn dmi_matches(want_raw: &str) -> bool {
    for want in want_raw.split_whitespace() {
        if run_ok("omarchy-hw-match", &[want]) {
            return true;
        }
    }
    false
}

fn tuning_present(sink_name: &str) -> bool {
    let out = Command::new("pactl")
        .args(["list", "sinks", "short"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    out.lines().any(|l| l.split_whitespace().nth(1) == Some(sink_name))
}

fn tuned_hardware_sink(tunings_dir: &str) -> Option<String> {
    let dir = tuning_match(tunings_dir)?;
    let conf = parse_tuning_conf(&format!("{dir}/tuning.conf"));
    let pattern = conf.get("sink_pattern")?;
    let found = sink_matching(pattern);
    if found.is_empty() { None } else { Some(found) }
}

fn sink_matching(pattern: &str) -> String {
    let out = Command::new("pactl")
        .args(["list", "sinks", "short"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    for line in out.lines() {
        if let Some(name) = line.split_whitespace().nth(1) {
            // Use regex-like matching (pattern may contain *)
            if glob_match(pattern, name) {
                return name.to_string();
            }
        }
    }
    String::new()
}

fn glob_match(pattern: &str, text: &str) -> bool {
    // Convert simple glob (with *) to match
    if !pattern.contains('*') {
        return pattern == text;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() { continue; }
        if i == 0 {
            if !text.starts_with(part) { return false; }
            pos = part.len();
        } else if let Some(found) = text[pos..].find(part) {
            pos += found + part.len();
        } else {
            return false;
        }
    }
    true
}

fn app_streams() -> Vec<String> {
    let out = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut result = Vec::new();
    let mut current_id = String::new();
    for line in out.lines() {
        if line.starts_with("Sink Input #") {
            current_id = line.trim_start_matches("Sink Input #").to_string();
        }
        if line.contains("application.name = \"") {
            if let Some(start) = line.find("application.name = \"") {
                let after = &line[start + 20..];
                if let Some(end) = after.find('"') {
                    let app = &after[..end];
                    if app != "EasyEffects" && !current_id.is_empty() {
                        result.push(current_id.clone());
                    }
                }
            }
        }
    }
    result
}

fn move_apps_to(target: &str) {
    for id in app_streams() {
        let _ = Command::new("pactl")
            .args(["move-sink-input", &id, target])
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
    }
}

fn tuning_downstream_sink(sink_name: &str) -> String {
    Command::new("omarchy-audio-output-sink")
        .arg(sink_name)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn easyeffects_running() -> bool {
    let out = Command::new("pactl")
        .args(["list", "sinks", "short"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if out.lines().any(|l| l.split_whitespace().nth(1) == Some("easyeffects_sink")) {
        return true;
    }
    let uid = std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    run_ok("pgrep", &["-u", &uid, "-x", "easyeffects"])
        || run_ok("systemctl", &["--user", "is-active", "--quiet", "easyeffects.service"])
}

fn drop_stale_daemon_config(stale_daemon: &str, stale_wireplumber: &str) {
    let d = std::path::Path::new(stale_daemon).exists();
    let wp = std::path::Path::new(stale_wireplumber).exists();
    if !d && !wp { return; }
    let _ = fs::remove_file(stale_daemon);
    let _ = fs::remove_file(stale_wireplumber);
    let _ = Command::new("omarchy-restart-audio")
        .stdout(Stdio::null()).stderr(Stdio::null()).status();
    // Wait for pactl
    for _ in 0..40 {
        if run_ok("pactl", &["info"]) { break; }
        std::thread::sleep(Duration::from_millis(250));
    }
}

fn files_equal(a: &str, b: &str) -> bool {
    let fa = fs::read(a).unwrap_or_default();
    let fb = fs::read(b).unwrap_or_default();
    fa == fb
}

fn install_file(src: &str, dst: &str, _mode: u32) -> std::io::Result<()> {
    if let Some(parent) = std::path::Path::new(dst).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}
