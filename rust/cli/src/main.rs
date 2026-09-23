mod cli;
mod cmds;
mod filter;
mod output;
mod socket;
mod theme;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Cmd};
use std::process;

fn main() {
    let cli = Cli::parse();
    let code = match cli.cmd {
        Cmd::Add { name, async_flag } => cmds::add::run(&name, async_flag),
        Cmd::Drop { name, async_flag } => cmds::drop::run(&name, async_flag),
        Cmd::List => cmds::list::run(),
        Cmd::Search { query } => cmds::search::run(&query),
        Cmd::Sync => cmds::sync::run(),
        Cmd::Present { name } => cmds::present::run(&name),
        Cmd::Missing { name } => cmds::missing::run(&name),
        Cmd::Resolve { name } => cmds::resolve::run(&name),
        Cmd::Completions { shell } => {
            generate(shell, &mut Cli::command(), "omarchy-pkg", &mut std::io::stdout());
            0
        }
    };
    process::exit(code);
}
