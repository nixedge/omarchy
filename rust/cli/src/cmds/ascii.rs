use std::process::{Command, Stdio};

pub fn run(args: &[String]) -> i32 {
    // Parse args: collect words, handle --help, --, -?*
    let mut words: Vec<String> = Vec::new();
    let mut i = 0;
    let mut end_of_opts = false;

    while i < args.len() {
        let arg = &args[i];
        if end_of_opts {
            words.push(arg.clone());
        } else if arg == "--help" {
            print_usage();
            return 0;
        } else if arg == "--" {
            end_of_opts = true;
        } else if arg.starts_with('-') {
            eprintln!("Unknown option: {arg}");
            print_usage_err();
            return 1;
        } else {
            words.push(arg.clone());
        }
        i += 1;
    }

    // If no words, check stdin
    let text = if !words.is_empty() {
        words.join(" ")
    } else {
        // Check if stdin is a tty (is_terminal)
        if is_stdin_tty() {
            print_usage_err();
            return 1;
        }
        // Read from stdin
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).unwrap_or(0);
        buf.trim().to_string()
    };

    if text.trim().is_empty() {
        eprintln!("Nothing to render");
        return 1;
    }

    // The original script embeds the FIGlet font inline and passes it via fd 3.
    // We delegate to the original bash script to preserve full fidelity including
    // the embedded font data.
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script_path = if !omarchy_path.is_empty() {
        format!("{omarchy_path}/bin/omarchy-ascii")
    } else {
        "omarchy-ascii".to_string()
    };

    // Check if the script exists and we can use it
    if std::path::Path::new(&script_path).exists() {
        let mut cmd = Command::new("bash");
        cmd.arg(&script_path);
        for word in &words {
            cmd.arg(word);
        }
        if words.is_empty() {
            // pipe stdin through
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).unwrap_or(0);
            cmd.stdin(Stdio::piped());
            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to run omarchy-ascii: {e}");
                    return 1;
                }
            };
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(buf.as_bytes());
            }
            return child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
        }
        let status = cmd.status();
        return status.map(|s| s.code().unwrap_or(1)).unwrap_or(1);
    }

    // Fallback: delegate to figlet if available
    let figlet_ok = Command::new("figlet")
        .args(["-f", "delta_corps_priest_1"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok();

    if figlet_ok {
        let status = Command::new("figlet")
            .args(["-f", "delta_corps_priest_1", &text])
            .status();
        return status.map(|s| s.code().unwrap_or(1)).unwrap_or(1);
    }

    eprintln!("omarchy-ascii: cannot render (no font embedded in this build path)");
    1
}

fn print_usage() {
    println!("Usage: omarchy-ascii [text...]");
    println!();
    println!("Renders text as ASCII art in Delta Corps Priest 1, the FIGlet font the Omarchy");
    println!("logo is drawn in. Reads the text from stdin when given none.");
    println!();
    println!("Options:");
    println!("      --help  Show this help");
    println!();
    println!("Delta Corps Priest 1 draws letters and spaces only. Digits and punctuation have");
    println!("no glyph in it, so they are skipped.");
}

fn print_usage_err() {
    eprintln!("Usage: omarchy-ascii [text...]");
    eprintln!();
    eprintln!("Renders text as ASCII art in Delta Corps Priest 1, the FIGlet font the Omarchy");
    eprintln!("logo is drawn in. Reads the text from stdin when given none.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("      --help  Show this help");
}

fn is_stdin_tty() -> bool {
    use std::os::unix::io::AsRawFd;
    unsafe { libc_isatty(std::io::stdin().as_raw_fd()) != 0 }
}

#[link(name = "c")]
extern "C" {
    fn isatty(fd: i32) -> i32;
}

fn libc_isatty(fd: i32) -> i32 {
    unsafe { isatty(fd) }
}
