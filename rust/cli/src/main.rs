mod cli;
mod cmds;
mod filter;
mod output;
mod socket;
mod theme;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Cmd, ConfigCmd, PkgCmd};
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
        Cmd::Completions { shell } => {
            generate(shell, &mut Cli::command(), "omarchy-cli", &mut std::io::stdout());
            0
        }
    };
    process::exit(code);
}
