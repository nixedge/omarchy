use crate::cli::DevEnvName;
use std::process::Command;

pub fn remove(name: DevEnvName) -> i32 {
    let env_name = match name {
        DevEnvName::Ruby => "ruby",
        DevEnvName::Node => "node",
        DevEnvName::Bun => "bun",
        DevEnvName::Deno => "deno",
        DevEnvName::Go => "go",
        DevEnvName::Php => "php",
        DevEnvName::Laravel => "laravel",
        DevEnvName::Symfony => "symfony",
        DevEnvName::Python => "python",
        DevEnvName::Elixir => "elixir",
        DevEnvName::Phoenix => "phoenix",
        DevEnvName::Rust => "rust",
        DevEnvName::Java => "java",
        DevEnvName::Zig => "zig",
        DevEnvName::Ocaml => "ocaml",
        DevEnvName::Dotnet => "dotnet",
        DevEnvName::Clojure => "clojure",
        DevEnvName::Scala => "scala",
    };

    remove_env(env_name)
}

fn mise_uninstall(tool: &str) {
    let _ = Command::new("mise").args(["uninstall", tool, "--all"]).status();
    let _ = Command::new("mise").args(["rm", "-g", tool]).status();
}

fn remove_env(name: &str) -> i32 {
    match name {
        "ruby" => {
            println!("Removing Ruby…");
            mise_uninstall("ruby");
            let _ = std::fs::remove_file(super::home_path(".gemrc"));
            0
        }
        "node" => {
            println!("Removing Node.js…");
            mise_uninstall("node");
            0
        }
        "bun" => {
            println!("Removing Bun…");
            mise_uninstall("bun");
            0
        }
        "deno" => {
            println!("Removing Deno…");
            mise_uninstall("deno");
            0
        }
        "go" => {
            println!("Removing Go…");
            mise_uninstall("go");
            0
        }
        "php" | "laravel" | "symfony" => {
            println!("Removing PHP/Laravel/Symfony…");
            if name == "laravel" {
                let _ = Command::new("mise").args(["x", "php", "--", "composer", "global", "remove", "laravel/installer"]).status();
                let home = std::env::var("HOME").unwrap_or_default();
                let _ = std::fs::remove_file(format!("{home}/.local/bin/laravel"));
            }
            if name == "symfony" {
                // symfony-cli is a nixpkg on NixOS
                let _ = crate::cmds::drop::run("symfony-cli");
            }
            if name == "php" {
                mise_uninstall("php");
            }
            0
        }
        "python" => {
            println!("Removing Python…");
            mise_uninstall("python");
            let home = std::env::var("HOME").unwrap_or_default();
            let _ = std::fs::remove_file(format!("{home}/.local/bin/uv"));
            let _ = std::fs::remove_file(format!("{home}/.local/bin/uvx"));
            0
        }
        "elixir" | "phoenix" => {
            println!("Removing Elixir/Erlang…");
            mise_uninstall("elixir");
            mise_uninstall("erlang");
            0
        }
        "zig" => {
            println!("Removing Zig…");
            mise_uninstall("zig");
            mise_uninstall("zls");
            0
        }
        "rust" => {
            println!("Removing Rust…");
            let _ = Command::new("rustup").args(["self", "uninstall", "-y"]).status();
            0
        }
        "java" => {
            println!("Removing Java…");
            mise_uninstall("java");
            0
        }
        "dotnet" => {
            println!("Removing .NET…");
            mise_uninstall("dotnet");
            0
        }
        "ocaml" => {
            println!("Removing OCaml…");
            let _ = Command::new("opam").args(["switch", "remove", "default", "-y"]).status();
            let _ = std::fs::remove_dir_all(super::home_path(".opam"));
            0
        }
        "clojure" => {
            println!("Removing Clojure…");
            mise_uninstall("clojure");
            0
        }
        "scala" => {
            println!("Removing Scala…");
            mise_uninstall("scala");
            mise_uninstall("scala-cli");
            0
        }
        _ => {
            eprintln!("Unknown environment: {name}");
            1
        }
    }
}
