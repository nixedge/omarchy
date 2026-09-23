mod cli;
mod cmds;
mod filter;
mod output;
mod socket;
mod theme;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Cmd, ConfigCmd, PkgCmd, ServiceCmd};
use std::process;

// Maps legacy omarchy-* binary names to the subcommand args they expand to,
// enabling argv[0] dispatch when the binary is invoked via a symlink.
fn argv0_subcmds(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "omarchy-pkg-add" | "omarchy-pkg-install" => Some(&["pkg", "add"]),
        "omarchy-pkg-drop" => Some(&["pkg", "drop"]),
        "omarchy-pkg-list" => Some(&["pkg", "list"]),
        "omarchy-pkg-search" => Some(&["pkg", "search"]),
        "omarchy-pkg-sync" => Some(&["pkg", "sync"]),
        "omarchy-pkg-present" => Some(&["pkg", "present"]),
        "omarchy-pkg-missing" => Some(&["pkg", "missing"]),
        "omarchy-pkg-resolve" => Some(&["pkg", "resolve"]),
        // install-service-* → service enable <name>
        "omarchy-install-service-tailscale" => Some(&["service", "enable", "tailscale"]),
        "omarchy-install-service-signal" => Some(&["service", "enable", "signal"]),
        "omarchy-install-service-spotify" => Some(&["service", "enable", "spotify"]),
        "omarchy-install-service-1password" => Some(&["service", "enable", "1password"]),
        "omarchy-install-service-dropbox" => Some(&["service", "enable", "dropbox"]),
        "omarchy-install-service-nordvpn" => Some(&["service", "enable", "nordvpn"]),
        "omarchy-install-service-sunshine" => Some(&["service", "enable", "sunshine"]),
        // remove-service-* → service disable <name>
        "omarchy-remove-service-tailscale" => Some(&["service", "disable", "tailscale"]),
        "omarchy-remove-service-1password" => Some(&["service", "disable", "1password"]),
        "omarchy-remove-service-dropbox" => Some(&["service", "disable", "dropbox"]),
        "omarchy-remove-service-sunshine" => Some(&["service", "disable", "sunshine"]),
        // setup-security-* → service enable <name>
        "omarchy-setup-security-fingerprint" => Some(&["service", "enable", "fingerprint"]),
        "omarchy-setup-security-fido2" => Some(&["service", "enable", "fido2"]),
        "omarchy-setup-security-sshd" => Some(&["service", "enable", "sshd"]),
        "omarchy-setup-security-sudoless-docker" => Some(&["service", "enable", "sudoless-docker"]),
        // remove-security-* → service disable <name>
        "omarchy-remove-security-fingerprint" => Some(&["service", "disable", "fingerprint"]),
        "omarchy-remove-security-fido2" => Some(&["service", "disable", "fido2"]),
        "omarchy-remove-security-sshd" => Some(&["service", "disable", "sshd"]),
        "omarchy-remove-security-sudoless-docker" => Some(&["service", "disable", "sudoless-docker"]),
        _ => None,
    }
}

fn main() {
    let raw: Vec<String> = std::env::args().collect();

    let args = std::path::Path::new(&raw[0])
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(argv0_subcmds)
        .map(|subcmds| {
            let mut v = vec![raw[0].clone()];
            v.extend(subcmds.iter().map(|s| s.to_string()));
            v.extend_from_slice(&raw[1..]);
            v
        })
        .unwrap_or(raw);

    let cli = Cli::parse_from(&args);
    let code = match cli.cmd {
        Cmd::Pkg { subcmd } => match subcmd {
            PkgCmd::Add { name, sync } => cmds::add::run(&name, sync),
            PkgCmd::Drop { name, sync } => cmds::drop::run(&name, sync),
            PkgCmd::List => cmds::list::run(),
            PkgCmd::Search { query } => cmds::search::run(&query),
            PkgCmd::Sync => cmds::sync::run(),
            PkgCmd::Present { name } => cmds::present::run(&name),
            PkgCmd::Missing { name } => cmds::missing::run(&name),
            PkgCmd::Resolve { name } => cmds::resolve::run(&name),
        },
        Cmd::Config { subcmd } => match subcmd {
            ConfigCmd::Edit => cmds::config::edit::run(),
            ConfigCmd::Show => cmds::config::show::run(),
            ConfigCmd::Check => cmds::config::check::run(),
        },
        Cmd::Service { subcmd } => match subcmd {
            ServiceCmd::Enable { name } => cmds::service::enable::run(&name),
            ServiceCmd::Disable { name } => cmds::service::disable::run(&name),
            ServiceCmd::List => cmds::service::list::run(),
        },
        Cmd::Completions { shell } => {
            generate(shell, &mut Cli::command(), "omarchy-cli", &mut std::io::stdout());
            0
        }
    };
    process::exit(code);
}
