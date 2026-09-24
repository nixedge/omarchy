use crate::cli::DevEnvName;
use std::process::Command;

pub fn install(name: DevEnvName) -> i32 {
    // Dev environments are managed via mise on both Arch and NixOS.
    // Delegate to the shell logic for each environment.
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

    let status = Command::new("mise").args(["use", "-g", &format!("{env_name}@latest")]).status();

    match status {
        Ok(s) if s.success() => 0,
        _ => {
            // Fall back to env-specific logic embedded here
            install_env(env_name)
        }
    }
}

fn install_env(name: &str) -> i32 {
    match name {
        "ruby" => {
            println!("Installing Ruby on Rails…");
            let _ = Command::new("mise").args(["settings", "add", "ruby.compile", "false"]).status();
            let _ = Command::new("mise").args(["use", "--global", "ruby@latest"]).status();
            let _ = std::fs::write(super::home_path(".gemrc"), "gem: --no-document\n");
            let _ = Command::new("mise").args(["x", "ruby", "--", "gem", "install", "rails", "--no-document"]).status();
            println!("\nYou can now run: rails new myproject");
            0
        }
        "node" => {
            println!("Installing Node.js…");
            let _ = Command::new("mise").args(["use", "--global", "node"]).status();
            0
        }
        "bun" => {
            println!("Installing Bun…");
            let _ = Command::new("mise").args(["use", "-g", "bun@latest"]).status();
            0
        }
        "deno" => {
            println!("Installing Deno…");
            let _ = Command::new("mise").args(["use", "-g", "deno@latest"]).status();
            0
        }
        "go" => {
            println!("Installing Go…");
            let _ = Command::new("mise").args(["use", "--global", "go@latest"]).status();
            0
        }
        "python" => {
            println!("Installing Python…");
            let _ = Command::new("mise").args(["use", "--global", "python@latest"]).status();
            println!("Installing uv…");
            let _ = Command::new("sh")
                .arg("-c")
                .arg("curl -fsSL https://astral.sh/uv/install.sh | sh")
                .status();
            0
        }
        "rust" => {
            println!("Installing Rust…");
            let s = Command::new("sh")
                .arg("-c")
                .arg("bash -c \"$(curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs)\" -- -y")
                .status();
            if s.map(|s| s.success()).unwrap_or(false) { 0 } else { 1 }
        }
        "java" => {
            println!("Installing Java…");
            let _ = Command::new("mise").args(["use", "--global", "java@latest"]).status();
            0
        }
        "zig" => {
            println!("Installing Zig…");
            let _ = Command::new("mise").args(["use", "--global", "zig@latest"]).status();
            let _ = Command::new("mise").args(["use", "-g", "zls@latest"]).status();
            0
        }
        "elixir" | "phoenix" => {
            println!("Installing Elixir…");
            let _ = Command::new("mise").args(["use", "--global", "erlang@latest"]).status();
            let _ = Command::new("mise").args(["use", "--global", "elixir@latest"]).status();
            let _ = Command::new("mise").args(["x", "elixir", "--", "mix", "local.hex", "--force"]).status();
            if name == "phoenix" {
                let _ = Command::new("mise").args(["x", "elixir", "--", "mix", "local.rebar", "--force"]).status();
                let _ = Command::new("mise").args(["x", "elixir", "--", "mix", "archive.install", "hex", "phx_new", "--force"]).status();
                println!("\nYou can now run: mix phx.new my_app");
            }
            0
        }
        "dotnet" => {
            println!("Installing .NET…");
            let _ = Command::new("mise").args(["use", "--global", "dotnet@latest"]).status();
            0
        }
        "clojure" => {
            println!("Installing Clojure…");
            let _ = Command::new("mise").args(["use", "--global", "clojure@latest"]).status();
            0
        }
        "scala" => {
            println!("Installing Scala…");
            let _ = Command::new("mise").args(["use", "--global", "java@latest"]).status();
            let _ = Command::new("mise").args(["use", "--global", "scala@latest"]).status();
            let _ = Command::new("mise").args(["use", "--global", "scala-cli@latest"]).status();
            0
        }
        _ => {
            eprintln!("Unknown environment: {name}");
            1
        }
    }
}
