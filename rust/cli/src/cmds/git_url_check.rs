pub fn run(url: &str) -> i32 {
    // Reject dangerous git URL helpers
    if url.contains("::") {
        eprintln!("Rejected: URL contains '::'");
        return 1;
    }

    // Check for allowed schemes
    let allowed = ["ssh://", "git://", "http://", "https://", "git@"];
    let allowed_no_scheme = ["git@"];

    for prefix in &allowed {
        if url.starts_with(prefix) {
            return 0;
        }
    }

    // Also allow bare scp-style: user@host:path
    if url.contains('@') && !url.starts_with('-') {
        return 0;
    }

    eprintln!("Rejected git URL: {url}");
    1
}
