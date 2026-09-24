mod cli;
mod cmds;
mod desktop;
mod filter;
mod output;
mod socket;
mod theme;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Cmd, ConfigCmd, InstallCmd, PkgCmd, RemoveCmd, ServiceCmd};
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
        // install-gaming-* → install gaming-*
        "omarchy-install-gaming-steam" => Some(&["install", "gaming-steam"]),
        "omarchy-install-gaming-heroic" => Some(&["install", "gaming-heroic"]),
        "omarchy-install-gaming-lutris" => Some(&["install", "gaming-lutris"]),
        "omarchy-install-gaming-retroarch" => Some(&["install", "gaming-retroarch"]),
        "omarchy-install-gaming-xbox-controllers" => Some(&["install", "gaming-xbox-controllers"]),
        "omarchy-install-gaming-xbox-cloud" => Some(&["install", "gaming-xbox-cloud"]),
        "omarchy-install-gaming-battlenet" => Some(&["install", "gaming-battlenet"]),
        "omarchy-install-gaming-geforce-now" => Some(&["install", "gaming-geforce-now"]),
        "omarchy-install-gaming-gpu-lib32" => Some(&["install", "gaming-gpu-lib32"]),
        // remove-gaming-* → remove gaming-*
        "omarchy-remove-gaming-steam" => Some(&["remove", "gaming-steam"]),
        "omarchy-remove-gaming-heroic" => Some(&["remove", "gaming-heroic"]),
        "omarchy-remove-gaming-lutris" => Some(&["remove", "gaming-lutris"]),
        "omarchy-remove-gaming-retroarch" => Some(&["remove", "gaming-retroarch"]),
        "omarchy-remove-gaming-minecraft" => Some(&["remove", "gaming-minecraft"]),
        "omarchy-remove-gaming-xbox-controllers" => Some(&["remove", "gaming-xbox-controllers"]),
        "omarchy-remove-gaming-xbox-cloud" => Some(&["remove", "gaming-xbox-cloud"]),
        "omarchy-remove-gaming-battlenet" => Some(&["remove", "gaming-battlenet"]),
        "omarchy-remove-gaming-geforce-now" => Some(&["remove", "gaming-geforce-now"]),
        // install-editor-* → install editor-*
        "omarchy-install-editor-helix" => Some(&["install", "editor-helix"]),
        "omarchy-install-editor-vscode" => Some(&["install", "editor-vscode"]),
        "omarchy-install-editor-emacs" => Some(&["install", "editor-emacs"]),
        "omarchy-install-editor-zed" => Some(&["install", "editor-zed"]),
        // install-ai-* → install ai-*
        "omarchy-install-ai-claude" => Some(&["install", "ai-claude"]),
        "omarchy-install-ai-hermes" => Some(&["install", "ai-hermes"]),
        "omarchy-install-ai-t3-code" => Some(&["install", "ai-t3-code"]),
        "omarchy-install-ai-chatgpt" => Some(&["install", "ai-chatgpt"]),
        // remove-ai-* → remove ai-*
        "omarchy-remove-ai-claude" => Some(&["remove", "ai-claude"]),
        "omarchy-remove-ai-hermes" => Some(&["remove", "ai-hermes"]),
        "omarchy-remove-ai-t3-code" => Some(&["remove", "ai-t3-code"]),
        "omarchy-remove-ai-ollama" => Some(&["remove", "ai-ollama"]),
        "omarchy-remove-ai-chatgpt" => Some(&["remove", "ai-chatgpt"]),
        "omarchy-remove-ai-lm-studio" => Some(&["remove", "ai-lm-studio"]),
        "omarchy-remove-ai-grok-bot" => Some(&["remove", "ai-grok-bot"]),
        "omarchy-remove-ai-perplexity" => Some(&["remove", "ai-perplexity"]),
        // install-chromium-* → install chromium-*
        "omarchy-install-chromium-claude" => Some(&["install", "chromium-claude"]),
        "omarchy-install-chromium-copy-url" => Some(&["install", "chromium-copy-url"]),
        "omarchy-install-chromium-ytdlp" => Some(&["install", "chromium-ytdlp"]),
        "omarchy-install-chromium-google-account" => Some(&["install", "chromium-google-account"]),
        // install/remove with arguments — inject the subcommand prefix, args follow from argv
        "omarchy-install-browser" => Some(&["install", "browser"]),
        "omarchy-remove-browser" => Some(&["remove", "browser"]),
        "omarchy-install-dev-env" => Some(&["install", "dev-env"]),
        "omarchy-remove-dev-env" => Some(&["remove", "dev-env"]),
        "omarchy-install-terminal" => Some(&["install", "terminal"]),
        "omarchy-install-font" => Some(&["install", "font"]),
        "omarchy-install-and-launch" => Some(&["install", "and-launch"]),
        "omarchy-install-app" => Some(&["install", "app"]),
        "omarchy-install-openclaw-cli" => Some(&["install", "openclaw-cli"]),
        "omarchy-install-docker-dbs" => Some(&["install", "docker-dbs"]),
        "omarchy-install-preinstalls" => Some(&["install", "preinstalls"]),
        "omarchy-install-service-once" => Some(&["install", "service-once"]),
        "omarchy-voxtype-install" => Some(&["install", "voxtype"]),
        "omarchy-tui-install" => Some(&["install", "tui"]),
        "omarchy-webapp-install" => Some(&["install", "webapp"]),
        "omarchy-remove-preinstalls" => Some(&["remove", "preinstalls"]),
        "omarchy-voxtype-remove" => Some(&["remove", "voxtype"]),
        "omarchy-tui-remove" => Some(&["remove", "tui"]),
        "omarchy-tui-remove-all" => Some(&["remove", "tui", "--all"]),
        "omarchy-webapp-remove" => Some(&["remove", "webapp"]),
        "omarchy-webapp-remove-all" => Some(&["remove", "webapp", "--all"]),
        "omarchy-remove-launcher-entry" => Some(&["remove", "launcher-entry"]),
        "omarchy-install-hermes-cli" => Some(&["install", "hermes-cli"]),
        "omarchy-remove-ai-openclaw" => Some(&["remove", "ai-openclaw"]),
        "omarchy-installed-service-tailscale" => Some(&["service", "active", "tailscale"]),
        "omarchy-installed-service-dropbox" => Some(&["service", "active", "dropbox"]),
        "omarchy-install-ai-openclaw" => Some(&["install", "ai-openclaw"]),
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
            ServiceCmd::Active { name } => cmds::service::active::run(&name),
        },
        Cmd::Install { subcmd } => match subcmd {
            InstallCmd::GamingSteam => cmds::install::gaming::steam(),
            InstallCmd::GamingHeroic => cmds::install::gaming::heroic(),
            InstallCmd::GamingLutris => cmds::install::gaming::lutris(),
            InstallCmd::GamingRetroarch => cmds::install::gaming::retroarch(),
            InstallCmd::GamingXboxControllers => cmds::install::gaming::xbox_controllers(),
            InstallCmd::GamingXboxCloud => cmds::install::gaming::xbox_cloud(),
            InstallCmd::GamingBattlenet => cmds::install::gaming::battlenet(),
            InstallCmd::GamingGeforceNow => cmds::install::gaming::geforce_now(),
            InstallCmd::GamingGpuLib32 => { cmds::install::gaming::gpu_lib32(); 0 },
            InstallCmd::EditorHelix => cmds::install::editor::helix(),
            InstallCmd::EditorVscode => cmds::install::editor::vscode(),
            InstallCmd::EditorEmacs => cmds::install::editor::emacs(),
            InstallCmd::EditorZed => cmds::install::editor::zed(),
            InstallCmd::AiClaude => cmds::install::ai::claude(),
            InstallCmd::AiHermes => cmds::install::ai::hermes(),
            InstallCmd::AiT3code => cmds::install::ai::t3code(),
            InstallCmd::AiChatgpt => cmds::install::ai::chatgpt(),
            InstallCmd::Browser { name } => cmds::install::browser::install(name),
            InstallCmd::DevEnv { name } => cmds::install::devenv::install(name),
            InstallCmd::Terminal { name } => cmds::install::terminal::install(name),
            InstallCmd::ChromiumClaude => cmds::install::chromium::claude(),
            InstallCmd::ChromiumCopyUrl => cmds::install::chromium::copy_url(),
            InstallCmd::ChromiumYtdlp => cmds::install::chromium::ytdlp(),
            InstallCmd::ChromiumGoogleAccount => cmds::install::chromium::google_account(),
            InstallCmd::Font { name, package, family } => cmds::install::font::install(&name, &package, &family),
            InstallCmd::AndLaunch { name, packages, desktop_id } => cmds::install::and_launch::run(&name, &packages, &desktop_id),
            InstallCmd::App { name, packages } => cmds::install::app::run(&name, &packages),
            InstallCmd::OpenclawCli { check, .. } => cmds::install::openclaw_cli::run(check),
            InstallCmd::DockerDbs { dbs } => cmds::install::docker_dbs::run(&dbs),
            InstallCmd::Preinstalls => cmds::install::preinstalls::run(),
            InstallCmd::ServiceOnce => cmds::install::service_once::run(),
            InstallCmd::Voxtype => cmds::install::voxtype::run(),
            InstallCmd::Tui { name, command, window_style, icon } => {
                cmds::install::tui::run(name.as_deref(), command.as_deref(), window_style.as_deref(), icon.as_deref())
            }
            InstallCmd::Webapp { name, url, icon, custom_exec, mime_types } => {
                cmds::install::webapp::run(name.as_deref(), url.as_deref(), icon.as_deref(), custom_exec.as_deref(), mime_types.as_deref())
            }
            InstallCmd::AiOpenclaw => cmds::install::ai::openclaw(),
            InstallCmd::HermesCli { check, owns, remove } => cmds::install::hermes_cli::run(check, owns, remove),
        },
        Cmd::Remove { subcmd } => match subcmd {
            RemoveCmd::GamingSteam => cmds::remove::gaming::steam(),
            RemoveCmd::GamingHeroic => cmds::remove::gaming::heroic(),
            RemoveCmd::GamingLutris => cmds::remove::gaming::lutris(),
            RemoveCmd::GamingRetroarch => cmds::remove::gaming::retroarch(),
            RemoveCmd::GamingMinecraft => cmds::remove::gaming::minecraft(),
            RemoveCmd::GamingXboxControllers => cmds::remove::gaming::xbox_controllers(),
            RemoveCmd::GamingXboxCloud => cmds::remove::gaming::xbox_cloud(),
            RemoveCmd::GamingBattlenet => cmds::remove::gaming::battlenet(),
            RemoveCmd::GamingGeforceNow => cmds::remove::gaming::geforce_now(),
            RemoveCmd::AiClaude => cmds::remove::ai::claude(),
            RemoveCmd::AiHermes => cmds::remove::ai::hermes(),
            RemoveCmd::AiT3code => cmds::remove::ai::t3code(),
            RemoveCmd::AiOllama => cmds::remove::ai::ollama(),
            RemoveCmd::AiChatgpt => cmds::remove::ai::chatgpt(),
            RemoveCmd::AiLmStudio => cmds::remove::ai::lm_studio(),
            RemoveCmd::AiGrokBot => cmds::remove::ai::grok_bot(),
            RemoveCmd::AiPerplexity => cmds::remove::ai::perplexity(),
            RemoveCmd::Browser { name } => cmds::remove::browser::remove(name),
            RemoveCmd::DevEnv { name } => cmds::remove::devenv::remove(name),
            RemoveCmd::Preinstalls => cmds::remove::preinstalls::run(),
            RemoveCmd::Voxtype => cmds::remove::voxtype::run(),
            RemoveCmd::Tui { name, all } => {
                if all {
                    cmds::remove::tui::remove_all();
                    0
                } else {
                    cmds::remove::tui::remove(name.as_deref(), true)
                }
            }
            RemoveCmd::Webapp { name, all } => {
                if all {
                    cmds::remove::webapp::remove_all();
                    0
                } else {
                    cmds::remove::webapp::remove(name.as_deref(), true)
                }
            }
            RemoveCmd::LauncherEntry { desktop_id, entry_name } => {
                cmds::remove::launcher_entry::run(&desktop_id, entry_name.as_deref())
            }
            RemoveCmd::AiOpenclaw => cmds::remove::ai::openclaw(),
        },
        Cmd::Completions { shell } => {
            generate(shell, &mut Cli::command(), "omarchy-cli", &mut std::io::stdout());
            0
        }
    };
    process::exit(code);
}
