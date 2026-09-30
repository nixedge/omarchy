pub fn run(url: &str) -> i32 {
    let allowed_transports = [
        "ssh", "git", "git+ssh", "ssh+git", "http", "https", "ftp", "ftps", "file",
    ];

    if url.is_empty() {
        eprintln!("omarchy-git-url-check: a git URL is required");
        return 1;
    }

    // Leading dash looks like a git option; `<helper>::<address>` runs a program.
    if url.starts_with('-') || is_helper_url(url) {
        eprintln!(
            "omarchy-git-url-check: '{}' names a git option or transport helper, not a repository.",
            url
        );
        return 1;
    }

    // `<scheme>://<address>` — allowlist git's own transports.
    if let Some(scheme) = extract_scheme(url) {
        if allowed_transports.contains(&scheme.as_str()) {
            return 0;
        }
        eprintln!(
            "omarchy-git-url-check: '{}' names the '{}' transport, which Omarchy does not clone from.",
            url, scheme
        );
        return 1;
    }

    // Bare path or scp-style user@host:path — always safe.
    0
}

/// Return true if the URL matches `<alphanum+>::<rest>` (the git helper pattern).
fn is_helper_url(url: &str) -> bool {
    let Some(idx) = url.find("::") else { return false };
    let prefix = &url[..idx];
    !prefix.is_empty()
        && prefix
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-')
}

/// Extract the scheme from a `scheme://...` URL, returning `None` for scp-style
/// and bare paths.
fn extract_scheme(url: &str) -> Option<String> {
    let idx = url.find("://")?;
    let prefix = &url[..idx];
    if prefix.is_empty()
        || !prefix
            .chars()
            .next()
            .map(|c| c.is_ascii_alphanumeric())
            .unwrap_or(false)
    {
        return None;
    }
    if prefix
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-')
    {
        Some(prefix.to_owned())
    } else {
        None
    }
}
