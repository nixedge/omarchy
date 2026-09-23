mod cli;
mod cmds;
mod filter;
mod output;
mod socket;
mod theme;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Cmd, PkgCmd};
use std::process;

fn main() {
    let cli = Cli::parse();
    let code = match cli.cmd {
        Cmd::Pkg { subcmd } => match subcmd {
            PkgCmd::Add { name, async_flag } => cmds::add::run(&name, async_flag),
            PkgCmd::Drop { name, async_flag } => cmds::drop::run(&name, async_flag),
            PkgCmd::List => cmds::list::run(),
            PkgCmd::Search { query } => cmds::search::run(&query),
            PkgCmd::Sync => cmds::sync::run(),
            PkgCmd::Present { name } => cmds::present::run(&name),
            PkgCmd::Missing { name } => cmds::missing::run(&name),
            PkgCmd::Resolve { name } => cmds::resolve::run(&name),
        },
        Cmd::Completions { shell } => {
            generate(shell, &mut Cli::command(), "omarchy-cli", &mut std::io::stdout());
            0
        }
    };
    process::exit(code);
}
