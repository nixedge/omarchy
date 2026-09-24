use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub fn run(target_dir: Option<&str>) -> i32 {
    let target = target_dir
        .map(|d| d.to_string())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_default();
            std::env::var("XDG_CACHE_HOME")
                .unwrap_or_else(|_| format!("{home}/.cache"))
                + "/omarchy"
        });

    if let Err(e) = fs::create_dir_all(&target) {
        eprintln!("Cannot create target dir {target}: {e}");
        return 2;
    }

    let phase_seconds = 8u64;
    let parallel = 4;
    let chunk_mb = 4;
    let file_mb = 256;

    // Find block device backing target
    let source_dev = Command::new("findmnt")
        .args(["-no", "SOURCE", "--target", &target])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    // Strip btrfs subvolume suffix
    let source_dev = if let Some(pos) = source_dev.find('[') {
        source_dev[..pos].to_string()
    } else {
        source_dev
    };

    if !source_dev.starts_with("/dev/") {
        eprintln!("Cannot find a disk behind {target}");
        return 1;
    }

    let dev = Path::new(&source_dev).file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let stat_path = format!("/sys/class/block/{dev}/stat");
    if !Path::new(&stat_path).exists() {
        eprintln!("No I/O statistics for {dev}");
        return 1;
    }

    // Walk dm-crypt/LVM to physical disk
    let disk = find_physical_disk(&dev);
    let model = Command::new("lsblk")
        .args(["-dno", "MODEL", &format!("/dev/{disk}")])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    println!("disk {}", if model.is_empty() { disk.clone() } else { model });

    // Check available space
    let available_mb = get_available_mb(&target);
    let needed = parallel * file_mb * 2;
    if available_mb < needed as u64 {
        eprintln!("Need at least {needed}MB free on {target}");
        return 1;
    }

    // Create temp files in shared memory for chunk and test files
    let chunk_file = create_temp_file("/dev/shm", "omarchy-disk-speedtest-", ".src");
    let test_files: Vec<String> = (0..parallel)
        .map(|_| create_temp_file(&target, "disk-speedtest-", ".dat"))
        .collect();

    // Setup cleanup
    let chunk_file_c = chunk_file.clone();
    let test_files_c = test_files.clone();

    // Fill chunk with random data
    println!("Preparing test data...");
    let dd_status = Command::new("dd")
        .args([
            "if=/dev/urandom",
            &format!("of={chunk_file}"),
            &format!("bs={chunk_mb}M"),
            &format!("count={}", file_mb / chunk_mb),
            "status=none",
        ])
        .status();
    if !dd_status.map(|s| s.success()).unwrap_or(false) {
        eprintln!("Direct disk I/O is not available on {target}");
        cleanup(&chunk_file_c, &test_files_c);
        return 1;
    }

    // Stage test files for read phase
    let stage_ok = stage_files(&chunk_file, &test_files, chunk_mb, file_mb);
    if !stage_ok {
        eprintln!("Direct disk I/O is not available on {target}");
        cleanup(&chunk_file_c, &test_files_c);
        return 1;
    }

    // Run read phase
    run_phase("read", &test_files, &chunk_file, &dev, phase_seconds, chunk_mb);

    // Run write phase
    run_phase("write", &test_files, &chunk_file, &dev, phase_seconds, chunk_mb);

    cleanup(&chunk_file_c, &test_files_c);
    0
}

fn create_temp_file(dir: &str, prefix: &str, suffix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos()).unwrap_or(0);
    let pid = std::process::id();
    let path = format!("{dir}/{prefix}{pid}{ts}{suffix}");
    let _ = fs::write(&path, "");
    path
}

fn cleanup(chunk: &str, files: &[String]) {
    let _ = fs::remove_file(chunk);
    for f in files { let _ = fs::remove_file(f); }
}

fn stage_files(chunk: &str, files: &[String], chunk_mb: usize, file_mb: usize) -> bool {
    let handles: Vec<_> = files.iter().map(|f| {
        let chunk = chunk.to_string();
        let f = f.clone();
        let chunk_mb = chunk_mb;
        let file_mb = file_mb;
        std::thread::spawn(move || {
            Command::new("dd")
                .args([
                    &format!("if={chunk}"),
                    &format!("of={f}"),
                    &format!("bs={chunk_mb}M"),
                    "oflag=direct",
                    "conv=notrunc",
                    "status=none",
                ])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        })
    }).collect();

    handles.into_iter().all(|h| h.join().unwrap_or(false))
}

fn read_sectors(dev: &str, which: &str) -> u64 {
    let stat = fs::read_to_string(format!("/sys/class/block/{dev}/stat"))
        .unwrap_or_default();
    let fields: Vec<&str> = stat.split_whitespace().collect();
    match which {
        "read" => fields.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
        "write" => fields.get(6).and_then(|s| s.parse().ok()).unwrap_or(0),
        _ => 0,
    }
}

fn run_phase(phase: &str, files: &[String], chunk: &str, dev: &str, seconds: u64, chunk_mb: usize) {
    let chunk = chunk.to_string();
    let phase_str = phase.to_string();

    // Spawn workers
    let running = Arc::new(Mutex::new(true));
    let handles: Vec<_> = files.iter().map(|f| {
        let f = f.clone();
        let chunk = chunk.clone();
        let phase = phase_str.clone();
        let running = running.clone();
        let chunk_mb = chunk_mb;
        std::thread::spawn(move || {
            while *running.lock().unwrap() {
                let ok = if phase == "write" {
                    Command::new("dd")
                        .args([
                            &format!("if={chunk}"),
                            &format!("of={f}"),
                            &format!("bs={chunk_mb}M"),
                            "oflag=direct",
                            "conv=notrunc",
                            "status=none",
                        ])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                } else {
                    Command::new("dd")
                        .args([
                            &format!("if={f}"),
                            "of=/dev/null",
                            &format!("bs={chunk_mb}M"),
                            "iflag=direct",
                            "status=none",
                        ])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                };
                if !ok { break; }
            }
        })
    }).collect();

    let before = read_sectors(dev, phase);
    let start = Instant::now();
    let deadline = Instant::now() + Duration::from_secs(seconds);

    let mut baseline_sectors = before;
    let mut baseline_time = start;
    let mut sample = 0;
    let mut last_before = before;
    let mut last_after = before;

    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_secs(1));
        let after = read_sectors(dev, phase);
        let rate = (after.saturating_sub(last_before)) * 512 / 1_000_000;
        println!("{phase} {rate}");
        sample += 1;
        if sample == 1 {
            baseline_sectors = after;
            baseline_time = Instant::now();
        }
        last_before = after;
        last_after = after;
    }

    *running.lock().unwrap() = false;
    for h in handles { let _ = h.join(); }

    // Final steady-state mean
    if sample > 1 {
        let elapsed = baseline_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            let final_rate = (last_after.saturating_sub(baseline_sectors)) as f64 * 512.0 / 1_000_000.0 / elapsed;
            println!("{phase} {:.0}", final_rate);
        }
    }
}

fn find_physical_disk(dev: &str) -> String {
    let mut disk = dev.to_string();
    loop {
        let slaves_path = format!("/sys/class/block/{disk}/slaves");
        let slave = fs::read_dir(&slaves_path).ok()
            .and_then(|mut d| d.next())
            .and_then(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string());

        match slave {
            Some(s) => disk = s,
            None => break,
        }
    }

    // Strip partition number
    if Path::new(&format!("/sys/class/block/{disk}/partition")).exists() {
        let parent = fs::read_link(format!("/sys/class/block/{disk}"))
            .map(|p| p.parent().map(|pp| pp.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()).unwrap_or_default())
            .unwrap_or_default();
        if !parent.is_empty() {
            disk = parent;
        }
    }

    disk
}

fn get_available_mb(path: &str) -> u64 {
    Command::new("df")
        .args(["--output=avail", "-m", path])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .nth(1)
                .and_then(|l| l.trim().parse().ok())
                .unwrap_or(0)
        })
        .unwrap_or(0)
}
