use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "omarchy-cli", about = "Omarchy command-line interface")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    #[command(about = "Package management")]
    Pkg {
        #[command(subcommand)]
        subcmd: PkgCmd,
    },
    #[command(hide = true, about = "Print shell completion script")]
    Completions { shell: Shell },
}

#[derive(Subcommand)]
pub enum PkgCmd {
    #[command(about = "Add a package")]
    Add {
        name: String,
        #[arg(long = "async", help = "Queue rebuild in background")]
        async_flag: bool,
    },
    #[command(about = "Remove a package")]
    Drop {
        name: String,
        #[arg(long = "async", help = "Queue rebuild in background")]
        async_flag: bool,
    },
    #[command(about = "List installed packages")]
    List,
    #[command(about = "Search nixpkgs for packages")]
    Search { query: String },
    #[command(about = "Sync system configuration")]
    Sync,
    #[command(about = "Check if a package is installed (exit 0 = present)")]
    Present { name: String },
    #[command(about = "Check if a package is missing (exit 0 = missing)")]
    Missing { name: String },
    #[command(about = "Resolve package name to nixpkgs attribute")]
    Resolve { name: String },
}
