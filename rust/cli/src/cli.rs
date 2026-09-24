use clap::{builder::PossibleValuesParser, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use omarchy_lib::services;

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
    #[command(about = "User NixOS configuration")]
    Config {
        #[command(subcommand)]
        subcmd: ConfigCmd,
    },
    #[command(about = "Optional system services")]
    Service {
        #[command(subcommand)]
        subcmd: ServiceCmd,
    },
    #[command(about = "Install optional software")]
    Install {
        #[command(subcommand)]
        subcmd: InstallCmd,
    },
    #[command(about = "Remove optional software")]
    Remove {
        #[command(subcommand)]
        subcmd: RemoveCmd,
    },
    #[command(hide = true, about = "Print shell completion script")]
    Completions { shell: Shell },
}

#[derive(Subcommand)]
pub enum InstallCmd {
    #[command(name = "gaming-steam", about = "Install Steam and graphics drivers")]
    GamingSteam,
    #[command(name = "gaming-heroic", about = "Install Heroic Games Launcher")]
    GamingHeroic,
    #[command(name = "gaming-lutris", about = "Install Lutris with Wine")]
    GamingLutris,
    #[command(name = "gaming-retroarch", about = "Install RetroArch with full core set")]
    GamingRetroarch,
    #[command(name = "gaming-xbox-controllers", about = "Install Xbox controller support")]
    GamingXboxControllers,
    #[command(name = "gaming-xbox-cloud", about = "Install Xbox Cloud Gaming web app")]
    GamingXboxCloud,
    #[command(name = "gaming-battlenet", about = "Install Battle.net via umu-launcher")]
    GamingBattlenet,
    #[command(name = "gaming-geforce-now", about = "Install GeForce NOW")]
    GamingGeforceNow,
    #[command(name = "gaming-gpu-lib32", about = "Install lib32 graphics drivers")]
    GamingGpuLib32,
    #[command(name = "editor-helix", about = "Install Helix editor")]
    EditorHelix,
    #[command(name = "editor-vscode", about = "Install VS Code")]
    EditorVscode,
    #[command(name = "editor-emacs", about = "Install Emacs")]
    EditorEmacs,
    #[command(name = "editor-zed", about = "Install Zed editor")]
    EditorZed,
    #[command(name = "ai-claude", about = "Install Claude desktop app")]
    AiClaude,
    #[command(name = "ai-hermes", about = "Install Hermes desktop app")]
    AiHermes,
    #[command(name = "ai-t3-code", about = "Install T3 Code")]
    AiT3code,
    #[command(name = "ai-chatgpt", about = "Install ChatGPT desktop app")]
    AiChatgpt,
    #[command(name = "browser", about = "Install a supported browser")]
    Browser {
        #[arg(value_enum)]
        name: BrowserName,
    },
    #[command(name = "dev-env", about = "Install a development environment")]
    DevEnv {
        #[arg(value_enum)]
        name: DevEnvName,
    },
    #[command(name = "terminal", about = "Install a terminal emulator")]
    Terminal {
        #[arg(value_enum)]
        name: TerminalName,
    },
    #[command(name = "chromium-claude", about = "Install Claude extension for Chromium browsers")]
    ChromiumClaude,
    #[command(name = "chromium-copy-url", about = "Install Copy URL native messaging host")]
    ChromiumCopyUrl,
    #[command(name = "chromium-ytdlp", about = "Install yt-dlp native messaging host")]
    ChromiumYtdlp,
    #[command(name = "chromium-google-account", about = "Enable Google account sign-in in Chromium")]
    ChromiumGoogleAccount,
    #[command(name = "font", about = "Install a Nerd Font and switch the system to it")]
    Font {
        #[arg(help = "Display name (e.g. 'Cascadia Mono')")]
        name: String,
        #[arg(help = "Package name (e.g. ttf-cascadia-mono-nerd)")]
        package: String,
        #[arg(help = "Font family name for omarchy-font-set (e.g. 'CaskaydiaMono Nerd Font')")]
        family: String,
    },
    #[command(name = "and-launch", about = "Install packages then launch a desktop app")]
    AndLaunch {
        #[arg(help = "Display name")]
        name: String,
        #[arg(help = "Space-separated package list")]
        packages: String,
        #[arg(help = "Desktop application ID for gtk-launch")]
        desktop_id: String,
    },
    #[command(name = "app", about = "Install one or more packages")]
    App {
        #[arg(help = "Display name")]
        name: String,
        #[arg(help = "Space-separated package list")]
        packages: String,
    },
    #[command(name = "openclaw-cli", about = "Ensure the OpenClaw CLI is installed")]
    OpenclawCli {
        #[arg(long, help = "Check if installed (exit 0 = present)", conflicts_with = "now")]
        check: bool,
        #[arg(long, help = "Install if missing (default)")]
        now: bool,
    },
    #[command(name = "docker-dbs", about = "Install a database in a Docker container")]
    DockerDbs {
        #[arg(help = "Database names to install (interactive if omitted)")]
        dbs: Vec<String>,
    },
    #[command(name = "preinstalls", about = "Restore preinstalled Omarchy applications")]
    Preinstalls,
    #[command(name = "service-once", about = "Install the ONCE service and launch it")]
    ServiceOnce,
    #[command(name = "voxtype", about = "Install and configure Voxtype dictation")]
    Voxtype,
    #[command(name = "tui", about = "Create a desktop launcher for a terminal UI app")]
    Tui {
        #[arg(help = "App name (interactive if omitted)")]
        name: Option<String>,
        #[arg(help = "Launch command")]
        command: Option<String>,
        #[arg(help = "Window style: float or tile")]
        window_style: Option<String>,
        #[arg(help = "Icon URL, file path, or icon name")]
        icon: Option<String>,
    },
    #[command(name = "webapp", about = "Create a desktop launcher for a web app")]
    Webapp {
        #[arg(help = "App name (interactive if omitted)")]
        name: Option<String>,
        #[arg(help = "URL")]
        url: Option<String>,
        #[arg(help = "Icon URL, file path, or icon name")]
        icon: Option<String>,
        #[arg(help = "Custom exec command")]
        custom_exec: Option<String>,
        #[arg(help = "MIME types")]
        mime_types: Option<String>,
    },
    #[command(name = "hermes-cli", about = "Ensure the Hermes agent CLI is installed")]
    HermesCli {
        #[arg(long, help = "Check if ready (exit 0 = ready)", conflicts_with_all = ["owns", "remove"])]
        check: bool,
        #[arg(long, help = "Exit 0 if this installer owns hermes", conflicts_with_all = ["check", "remove"])]
        owns: bool,
        #[arg(long, help = "Remove what this installer put in place", conflicts_with_all = ["check", "owns"])]
        remove: bool,
    },
}

#[derive(Subcommand)]
pub enum RemoveCmd {
    #[command(name = "gaming-steam", about = "Remove Steam and its data")]
    GamingSteam,
    #[command(name = "gaming-heroic", about = "Remove Heroic Games Launcher and its data")]
    GamingHeroic,
    #[command(name = "gaming-lutris", about = "Remove Lutris, Wine, and their data")]
    GamingLutris,
    #[command(name = "gaming-retroarch", about = "Remove RetroArch and all cores")]
    GamingRetroarch,
    #[command(name = "gaming-minecraft", about = "Remove Minecraft launcher and its data")]
    GamingMinecraft,
    #[command(name = "gaming-xbox-controllers", about = "Remove Xbox controller support")]
    GamingXboxControllers,
    #[command(name = "gaming-xbox-cloud", about = "Remove Xbox Cloud Gaming web app")]
    GamingXboxCloud,
    #[command(name = "gaming-battlenet", about = "Remove Battle.net and its prefix")]
    GamingBattlenet,
    #[command(name = "gaming-geforce-now", about = "Remove GeForce NOW")]
    GamingGeforceNow,
    #[command(name = "ai-claude", about = "Remove Claude desktop app")]
    AiClaude,
    #[command(name = "ai-hermes", about = "Remove Hermes desktop app")]
    AiHermes,
    #[command(name = "ai-t3-code", about = "Remove T3 Code")]
    AiT3code,
    #[command(name = "ai-ollama", about = "Remove Ollama and all models")]
    AiOllama,
    #[command(name = "ai-chatgpt", about = "Remove ChatGPT desktop app")]
    AiChatgpt,
    #[command(name = "ai-lm-studio", about = "Remove LM Studio and all models")]
    AiLmStudio,
    #[command(name = "ai-grok-bot", about = "Remove Grok Bot")]
    AiGrokBot,
    #[command(name = "ai-perplexity", about = "Remove Perplexity desktop app")]
    AiPerplexity,
    #[command(name = "browser", about = "Remove a browser")]
    Browser {
        #[arg(value_enum)]
        name: BrowserName,
    },
    #[command(name = "dev-env", about = "Remove a development environment")]
    DevEnv {
        #[arg(value_enum)]
        name: DevEnvName,
    },
    #[command(name = "preinstalls", about = "Remove preinstalled Omarchy applications")]
    Preinstalls,
    #[command(name = "voxtype", about = "Remove Voxtype dictation")]
    Voxtype,
    #[command(name = "tui", about = "Remove a terminal UI desktop launcher")]
    Tui {
        #[arg(help = "App name (interactive if omitted)")]
        name: Option<String>,
        #[arg(long, help = "Remove all TUI launchers", conflicts_with = "name")]
        all: bool,
    },
    #[command(name = "webapp", about = "Remove a web app desktop launcher")]
    Webapp {
        #[arg(help = "App name (interactive if omitted)")]
        name: Option<String>,
        #[arg(long, help = "Remove all web app launchers", conflicts_with = "name")]
        all: bool,
    },
    #[command(name = "launcher-entry", about = "Remove or uninstall a launcher entry")]
    LauncherEntry {
        #[arg(help = "Desktop application ID")]
        desktop_id: String,
        #[arg(help = "Display name (optional)")]
        entry_name: Option<String>,
    },
    #[command(name = "ai-openclaw", about = "Remove the OpenClaw agent platform")]
    AiOpenclaw,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum BrowserName {
    Chromium,
    Chrome,
    Brave,
    #[value(name = "brave-origin")]
    BraveOrigin,
    Edge,
    Firefox,
    Zen,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum DevEnvName {
    Ruby,
    Node,
    Bun,
    Deno,
    Go,
    Php,
    Laravel,
    Symfony,
    Python,
    Elixir,
    Phoenix,
    Rust,
    Java,
    Zig,
    Ocaml,
    Dotnet,
    Clojure,
    Scala,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum TerminalName {
    Alacritty,
    Foot,
    Ghostty,
    Kitty,
}

#[derive(Subcommand)]
pub enum ConfigCmd {
    #[command(about = "Edit user configuration in $EDITOR")]
    Edit,
    #[command(about = "Show current user configuration")]
    Show,
    #[command(about = "Validate configuration without applying (dry-activate)")]
    Check,
}

#[derive(Subcommand)]
pub enum ServiceCmd {
    #[command(about = "Enable a service")]
    Enable {
        #[arg(value_parser = PossibleValuesParser::new(services::KNOWN))]
        name: String,
    },
    #[command(about = "Disable a service")]
    Disable {
        #[arg(value_parser = PossibleValuesParser::new(services::KNOWN))]
        name: String,
    },
    #[command(about = "List available and enabled services")]
    List,
}

#[derive(Subcommand)]
pub enum PkgCmd {
    #[command(about = "Add a package")]
    Add {
        name: String,
        #[arg(long, help = "Block until rebuild completes")]
        sync: bool,
    },
    #[command(about = "Remove a package")]
    Drop {
        name: String,
        #[arg(long, help = "Block until rebuild completes")]
        sync: bool,
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
