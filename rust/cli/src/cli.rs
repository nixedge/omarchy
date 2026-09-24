use clap::{builder::PossibleValuesParser, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use omarchy_lib::services;

pub const HW_CHECKS: &[&str] = &[
    "nvidia", "nvidia-gsp", "nvidia-without-gsp", "intel", "intel-ptl", "intel-sof",
    "laptop", "fingerprint", "vulkan", "hybrid-gpu", "asus-rog", "asus-expertbook",
    "asus-zenbook", "framework16", "surface", "dell-xps-oled", "dell-xps-haptic",
    "dell-xps13-sidecar-amps", "elgato-camlink", "match",
];

pub const HW_STATES: &[&str] = &[
    "lid", "external-monitors", "clamshell", "display", "touchpad", "touchscreen", "webcam",
];

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
    #[command(about = "Hardware detection")]
    Hw {
        #[command(subcommand)]
        subcmd: HwCmd,
    },
    #[command(about = "Restart a component")]
    Restart {
        #[command(subcommand)]
        subcmd: RestartCmd,
    },
    #[command(about = "Desktop notifications")]
    Notification {
        #[command(subcommand)]
        subcmd: NotificationCmd,
    },
    #[command(about = "Battery information")]
    Battery {
        #[command(subcommand)]
        subcmd: BatteryCmd,
    },
    #[command(about = "Power supply detection")]
    Power {
        #[command(subcommand)]
        subcmd: PowerCmd,
    },
    #[command(name = "cmd", about = "Command presence checks")]
    CmdCheck {
        #[command(subcommand)]
        subcmd: CmdCheckCmd,
    },
    #[command(about = "Omarchy state file management")]
    State {
        action: String,
        name: String,
    },
    #[command(about = "Done marker management")]
    Done {
        action: String,
        name: String,
    },
    #[command(name = "show-done", about = "Print colored Done!/Failed! to tty")]
    ShowDone {
        #[arg(help = "Optional exit code (0 = Done, nonzero = Failed)")]
        exit_code: Option<i32>,
    },
    #[command(name = "show-logo", about = "Print the Omarchy logo")]
    ShowLogo,
    #[command(about = "Show Omarchy version")]
    Version,
    #[command(name = "version-branch", about = "Show current branch (dev checkout only)")]
    VersionBranch,
    #[command(about = "Power profile management")]
    Powerprofiles {
        #[command(subcommand)]
        subcmd: PowerprofilesCmd,
    },
    #[command(about = "On-screen display")]
    Osd {
        #[arg(short = 'i', long, help = "Icon")]
        icon: Option<String>,
        #[arg(short = 'm', long, help = "Message")]
        message: Option<String>,
        #[arg(short = 'p', long, help = "Progress (0.0–1.0)")]
        progress: Option<String>,
        #[arg(short = 'd', long, help = "Duration in ms")]
        duration: Option<String>,
    },
    #[command(name = "windows-key", about = "Extract Windows product key from MSDM table")]
    WindowsKey,
    #[command(name = "windows-vm", about = "Manage the Windows VM")]
    WindowsVm {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Tailscale file transfer")]
    Tailscale {
        #[command(subcommand)]
        subcmd: TailscaleCmd,
    },
    #[command(about = "System update helpers")]
    Update {
        #[command(subcommand)]
        subcmd: UpdateCmd,
    },
    #[command(about = "System setup helpers")]
    Setup {
        #[command(subcommand)]
        subcmd: SetupCmd,
    },
    #[command(about = "Sudo helpers")]
    Sudo {
        #[command(subcommand)]
        subcmd: SudoCmd,
    },
    #[command(about = "Run the Omarchy screensaver")]
    Screensaver {
        #[arg(help = "Force start")]
        force: bool,
    },
    #[command(name = "git-url-check", about = "Validate a git URL")]
    GitUrlCheck { url: String },
    #[command(name = "games-retro-cores", about = "List installed RetroArch cores")]
    GamesRetroCores,
    #[command(name = "games-retro-install", about = "Create a desktop launcher for a RetroArch game")]
    GamesRetroInstall {
        #[arg(help = "Core name or path")]
        core: Option<String>,
        #[arg(help = "Path to game ROM")]
        game: Option<String>,
    },
    #[command(name = "monitor-state", about = "Print monitor panel state for the shell")]
    MonitorState,
    #[command(name = "disk-speedtest", about = "Measure live disk read and write speed")]
    DiskSpeedtest {
        #[arg(help = "Target directory for test files")]
        dir: Option<String>,
    },
    #[command(about = "Set and show lightweight desktop reminders")]
    Reminder {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "display-text-size", about = "Scale text in shell, GTK apps, and terminals")]
    DisplayTextSize {
        #[arg(help = "Size in px (9-20), 'reset', or omit to show current")]
        size: Option<String>,
    },
    #[command(about = "Audio output/input control")]
    Audio {
        #[command(subcommand)]
        subcmd: AudioCmd,
    },
    #[command(about = "Display and keyboard brightness")]
    Brightness {
        #[command(subcommand)]
        subcmd: BrightnessCmd,
    },
    #[command(about = "Toggle Omarchy features")]
    Toggle {
        #[command(subcommand)]
        subcmd: ToggleCmd,
    },
    #[command(about = "Bluetooth device management")]
    Bluetooth {
        #[command(subcommand)]
        subcmd: BluetoothCmd,
    },
    #[command(about = "Bar configuration")]
    Bar {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Print Omarchy ASCII art logo")]
    Ascii {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Font management")]
    Font {
        #[command(subcommand)]
        subcmd: FontCmd,
    },
    #[command(about = "Weather information")]
    Weather {
        #[command(subcommand)]
        subcmd: WeatherCmd,
    },
    #[command(about = "Coding agent management")]
    Agent {
        #[command(subcommand)]
        subcmd: AgentCmd,
    },
    #[command(about = "Branding customization")]
    Branding {
        #[command(subcommand)]
        subcmd: BrandingCmd,
    },
    #[command(about = "Screen capture tools")]
    Capture {
        #[command(subcommand)]
        subcmd: CaptureCmd,
    },
    #[command(name = "chromium-copy-url-host", about = "Native messaging host: copy tab URL", hide = true)]
    ChromiumCopyUrlHost,
    #[command(name = "chromium-ytdlp-host", about = "Native messaging host: yt-dlp downloader", hide = true)]
    ChromiumYtdlpHost {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Clipboard management")]
    Clipboard {
        #[command(subcommand)]
        subcmd: ClipboardCmd,
    },
    #[command(about = "Crash notification management")]
    Crash {
        #[command(subcommand)]
        subcmd: CrashCmd,
    },
    #[command(about = "Debug and diagnostics")]
    Debug {
        #[command(subcommand)]
        subcmd: DebugCmd,
    },
    #[command(name = "default", about = "Manage default applications")]
    DefaultApp {
        #[command(subcommand)]
        subcmd: DefaultCmd,
    },
    #[command(about = "Run and install user hooks")]
    Hook {
        #[command(subcommand)]
        subcmd: HookCmd,
    },
    #[command(name = "bar-text-color", about = "Choose legible bar text color", hide = true)]
    BarTextColor {
        position: String,
        bar_size: String,
        text_color: String,
        background_color: String,
        #[arg(long, help = "Background image path")]
        background: Option<String>,
        #[arg(long, help = "Screen dimensions WxH")]
        screen: Option<String>,
    },
    #[command(hide = true, about = "Print shell completion script")]
    Completions { shell: Shell },
    // ── Batch 3 ──────────────────────────────────────────────────────────────
    #[command(about = "Hyprland window/monitor helpers")]
    Hyprland {
        #[command(subcommand)]
        subcmd: HyprlandCmd,
    },
    #[command(about = "System power and session commands")]
    System {
        #[command(subcommand)]
        subcmd: SystemCmd,
    },
    #[command(about = "Launch applications")]
    Launch {
        #[command(subcommand)]
        subcmd: LaunchCmd,
    },
    #[command(name = "shell", about = "Send IPC calls to the Omarchy shell")]
    ShellIpc {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Run or check pending migrations")]
    Migrate {
        #[command(subcommand)]
        subcmd: MigrateCmd,
    },
    #[command(about = "Show or configure system DNS")]
    Dns {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "mise-install", about = "Install a mise-backed tool wrapper")]
    MiseInstall {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Network status and configuration")]
    Network {
        #[command(subcommand)]
        subcmd: NetworkCmd,
    },
}

#[derive(Subcommand)]
pub enum AudioCmd {
    #[command(name = "output-volume", about = "Adjust output volume and show OSD")]
    OutputVolume {
        #[arg(help = "raise|lower|mute-toggle|+N|-N")]
        action: String,
    },
    #[command(name = "output-switch", about = "Switch between audio outputs")]
    OutputSwitch,
    #[command(name = "input-mute", about = "Toggle microphone mute")]
    InputMute,
    #[command(name = "output-sink", about = "Print the sink that carries volume for an output")]
    OutputSink {
        #[arg(help = "Sink name (optional, defaults to current default)")]
        sink: Option<String>,
    },
    #[command(name = "sink-availability", about = "Print PulseAudio sink availability")]
    SinkAvailability,
    #[command(name = "source-switch", about = "Cycle media source")]
    SourceSwitch {
        #[arg(help = "next|previous (default: next)")]
        direction: Option<String>,
    },
    #[command(name = "input-set-default", about = "Set default audio input and move streams")]
    InputSetDefault {
        node_id: String,
        source_name: String,
    },
    #[command(name = "output-set-default", about = "Set default audio output and move streams")]
    OutputSetDefault {
        node_id: String,
        sink_name: String,
    },
    #[command(name = "tuning", about = "Manage speaker tuning")]
    Tuning {
        #[arg(help = "on|off|status|match|fronted-sink", default_value = "status")]
        action: String,
        #[arg(long, help = "Force reinstall even if already current")]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum BrightnessCmd {
    #[command(about = "Show or adjust display brightness")]
    Display {
        #[arg(long, help = "Skip OSD")]
        no_osd: bool,
        #[arg(long, help = "Monitor name")]
        monitor: Option<String>,
        #[arg(help = "+N%|N%-|N%|off|on")]
        step: Option<String>,
    },
    #[command(name = "display-apple", about = "Apple Studio Display brightness")]
    DisplayApple {
        #[arg(long, help = "Skip OSD")]
        no_osd: bool,
        #[arg(help = "+N%|N%-|N%")]
        step: Option<String>,
    },
    #[command(name = "display-ddc", about = "DDC/CI external monitor brightness")]
    DisplayDdc {
        monitor: String,
        #[arg(help = "+N%|N%-|N%")]
        step: Option<String>,
    },
    #[command(about = "Adjust keyboard backlight")]
    Keyboard {
        #[arg(long, help = "Skip OSD")]
        no_osd: bool,
        #[arg(help = "up|down|cycle|off|restore", default_value = "up")]
        direction: String,
    },
    #[command(name = "keyboard-mute", about = "Set mic-mute indicator LED")]
    KeyboardMute {
        #[arg(help = "on|off")]
        state: String,
    },
}

#[derive(Subcommand)]
pub enum ToggleCmd {
    #[command(name = "flag", about = "Toggle a named feature flag")]
    Flag {
        flag_name: String,
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(about = "Toggle bar visibility")]
    Bar {
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(name = "crash-capture", about = "Toggle crash capture")]
    CrashCapture,
    #[command(about = "Check if a toggle is enabled (exit 0 = enabled)")]
    Enabled { flag_name: String },
    #[command(name = "fullscreen-desktop", about = "Toggle fullscreen desktop mode")]
    FullscreenDesktop {
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(name = "hybrid-gpu", about = "Toggle dedicated vs integrated GPU")]
    HybridGpu,
    #[command(about = "Toggle idle/stay-awake behavior")]
    Idle {
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(name = "input-device", about = "Enable/disable/toggle a Hyprland input device")]
    InputDevice {
        kind: String,
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(about = "Toggle nightlight screen temperature")]
    Nightlight {
        #[arg(long, help = "Print status JSON instead of toggling")]
        status: bool,
    },
    #[command(name = "notification-silencing", about = "Toggle notification do-not-disturb")]
    NotificationSilencing,
    #[command(about = "Toggle screensaver availability")]
    Screensaver,
    #[command(about = "Toggle suspend in system menu")]
    Suspend,
    #[command(about = "Enable/disable/toggle touchpad")]
    Touchpad {
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(about = "Enable/disable/toggle touchscreen")]
    Touchscreen {
        #[arg(default_value = "toggle")]
        action: String,
    },
}

#[derive(Subcommand)]
pub enum BluetoothCmd {
    #[command(about = "Control a Bluetooth device")]
    Device {
        #[arg(help = "pair|connect|disconnect|forget")]
        action: String,
        #[arg(help = "Device MAC address")]
        address: String,
    },
    #[command(about = "Turn Bluetooth on or off")]
    Power {
        #[arg(help = "on|off|toggle|is-on")]
        action: String,
    },
}

#[derive(Subcommand)]
pub enum FontCmd {
    #[command(about = "Show current monospace font")]
    Current,
    #[command(about = "List available monospace fonts")]
    List,
    #[command(about = "Set system monospace font")]
    Set {
        #[arg(help = "Font name")]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum WeatherCmd {
    #[command(about = "Return weather condition icon adjusted for sunrise/sunset")]
    Icon,
    #[command(about = "Show or set weather location")]
    Location {
        #[arg(long, help = "Location name")]
        set: Option<String>,
        #[arg(long, help = "Coordinates lat,lon (used with --set)")]
        coords: Option<String>,
        #[arg(long, help = "Return to IP auto-detect")]
        clear: bool,
    },
    #[command(about = "Return formatted weather status string")]
    Status,
}

#[derive(Subcommand)]
pub enum HwCmd {
    #[command(about = "Detect all hardware and print JSON summary")]
    Detect,
    #[command(about = "Check if a hardware feature is present (exit 0 = present)")]
    Check {
        #[arg(value_parser = PossibleValuesParser::new(HW_CHECKS))]
        name: String,
        #[arg(help = "Pattern for 'match' check")]
        pattern: Option<String>,
    },
    #[command(about = "Query dynamic hardware state (exit 0 = true)")]
    State {
        #[arg(value_parser = PossibleValuesParser::new(HW_STATES))]
        name: String,
    },
    #[command(name = "recover-internal-monitor", about = "Clear internal-monitor-disable toggle if no external display")]
    RecoverInternalMonitor,
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
    #[command(name = "ai-openclaw", about = "Install the OpenClaw agent platform")]
    AiOpenclaw,
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
    #[command(about = "Check if a service daemon is running (exit 0 = active)")]
    Active {
        #[arg(value_parser = PossibleValuesParser::new(services::KNOWN))]
        name: String,
    },
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

// ---- New command group enums ----

#[derive(Subcommand)]
pub enum RestartCmd {
    #[command(about = "Restart an application")]
    App {
        name: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    #[command(about = "Restart audio services and recover stuck USB audio")]
    Audio,
    #[command(about = "Unblock Bluetooth")]
    Bluetooth,
    #[command(about = "Send SIGUSR2 to btop")]
    Btop,
    #[command(about = "Print gum environment variables from current theme")]
    Gum,
    #[command(about = "Send SIGUSR1 to helix")]
    Helix,
    #[command(about = "Reload herdr server config")]
    Herdr,
    #[command(about = "Reload Hyprland config")]
    Hyprctl,
    #[command(about = "Restart hyprsunset")]
    Hyprsunset,
    #[command(about = "Send SIGUSR2 to opencode")]
    Opencode,
    #[command(about = "Restart the Omarchy shell")]
    Shell,
    #[command(about = "Reload terminal configs")]
    Terminal,
    #[command(about = "Reload tmux config")]
    Tmux,
    #[command(about = "Rebind i2c HID touchpad driver")]
    Trackpad,
    #[command(about = "Restart WiFi networking")]
    Wifi,
    #[command(about = "Restart fcitx5 for XCompose support")]
    Xcompose,
}

#[derive(Subcommand)]
pub enum NotificationCmd {
    #[command(about = "Send a battery status notification")]
    Battery,
    #[command(about = "Send a time notification")]
    Time,
    #[command(about = "Toggle weather notification")]
    Weather,
    #[command(about = "Dismiss a notification by summary")]
    Dismiss { summary: String },
    #[command(about = "Wait for the notification server to be ready")]
    Wait {
        #[arg(default_value = "10", help = "Timeout in seconds")]
        seconds: u64,
    },
    #[command(about = "Send a desktop notification")]
    Send {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum BatteryCmd {
    #[command(about = "Check if a battery is present (exit 0 = present)")]
    Present,
    #[command(about = "Send a low battery notification")]
    Low {
        #[arg(help = "Current battery percentage")]
        percentage: u32,
    },
    #[command(about = "Print battery status")]
    Status {
        #[arg(long, help = "Output tab-separated fields for shell consumption")]
        shell: bool,
    },
}

#[derive(Subcommand)]
pub enum PowerCmd {
    #[command(about = "Check if AC power is present (exit 0 = present)")]
    Present,
}

#[derive(Subcommand)]
pub enum CmdCheckCmd {
    #[command(about = "Check if any command is missing from PATH (exit 0 = missing)")]
    Missing {
        #[arg(required = true)]
        cmds: Vec<String>,
    },
    #[command(about = "Check if all commands are present in PATH (exit 0 = all present)")]
    Present {
        #[arg(required = true)]
        cmds: Vec<String>,
    },
    #[command(name = "terminal-cwd", about = "Print CWD of active terminal")]
    TerminalCwd,
}

#[derive(Subcommand)]
pub enum PowerprofilesCmd {
    #[command(about = "Initialize power profile based on current power source")]
    Init,
    #[command(about = "List available power profiles")]
    List {
        #[arg(long, help = "Include active state (tab-separated)")]
        active_state: bool,
    },
    #[command(about = "Set the active power profile")]
    Set {
        #[arg(help = "Mode: autodetect, ac, or battery")]
        action: Option<String>,
        #[arg(help = "Profile name to set and save")]
        profile: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum TailscaleCmd {
    #[command(about = "Receive files via Taildrop")]
    Receive {
        #[arg(long, help = "Exit after receiving one batch")]
        once: bool,
        #[arg(help = "Download directory")]
        dir: Option<String>,
    },
    #[command(about = "Send files via Taildrop")]
    Send {
        #[arg(help = "Target machine")]
        machine: String,
        #[arg(help = "Files to send")]
        files: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum UpdateCmd {
    #[command(about = "Pull latest dev checkout")]
    Dev,
    #[command(about = "Update firmware via fwupd")]
    Firmware,
    #[command(about = "Manage the update lock")]
    Lock {
        action: String,
        #[arg(trailing_var_arg = true)]
        cmd_args: Vec<String>,
    },
    #[command(name = "requires-free-space", about = "Check for 10GiB free space")]
    RequiresFreeSpace,
    #[command(about = "Check for updates and refresh indicators")]
    Status,
    #[command(name = "stay-awake", about = "Inhibit sleep during updates")]
    StayAwake { action: String },
    #[command(about = "Sync system clock")]
    Time,
    #[command(name = "user-notify", about = "Delegate to omarchy-migrate-notify")]
    UserNotify {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum SetupCmd {
    #[command(name = "direct-boot", about = "Manage EFI direct boot entry for Omarchy UKI")]
    DirectBoot,
}

#[derive(Subcommand)]
pub enum AgentCmd {
    #[command(about = "Launch the default coding agent")]
    Run {
        #[arg(long, help = "Run agent inline (no TUI wrapper)")]
        inline: bool,
        #[arg(long, help = "Pick agent if none set")]
        pick: bool,
        #[arg(long, help = "Pass a prompt to the agent")]
        prompt: Option<String>,
    },
    #[command(about = "Diagnose a crashed process with the default coding agent")]
    Crash {
        pid: String,
        comm: Option<String>,
        exe: Option<String>,
        signal: Option<String>,
    },
    #[command(about = "Launch the default coding agent with a prompt")]
    Prompt {
        #[arg(long, help = "Run agent inline (no TUI wrapper)")]
        inline: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        prompt: Vec<String>,
    },
    #[command(name = "usage-claude", about = "Show Claude API usage")]
    UsageClaude {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "usage-codex", about = "Show Codex usage")]
    UsageCodex {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "usage-fireworks", about = "Show Fireworks usage")]
    UsageFireworks {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "usage-update", about = "Regenerate agent usage data files")]
    UsageUpdate {
        #[arg(long, help = "Force refresh")]
        force: bool,
        #[arg(long, help = "Only update usage limits")]
        limits_only: bool,
        #[arg(long, value_name = "AGENT", help = "Exclude this agent")]
        except: Vec<String>,
        #[arg(help = "Agent(s) to update (default: all)")]
        agents: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum BrandingCmd {
    #[command(about = "Edit, set, or reset About branding")]
    About {
        #[arg(help = "image|text|reset")]
        mode: String,
    },
    #[command(about = "Edit, set, or reset screensaver branding")]
    Screensaver {
        #[arg(help = "image|text|reset")]
        mode: String,
    },
}

#[derive(Subcommand)]
pub enum CaptureCmd {
    #[command(about = "Decode a QR code from a screenshot region")]
    Qr,
    #[command(about = "Pick a screen region (shared helper)")]
    Region {
        #[arg(help = "region|windows|smart|fullscreen")]
        mode: Option<String>,
        #[arg(long, help = "Leave freeze running")]
        keep_freeze: bool,
        #[arg(long, help = "Print monitor:NAME when matched")]
        match_monitor: bool,
    },
    #[command(about = "Start or stop screen recording")]
    Screenrecording {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "screenrecording-with-webcam", about = "Screen recording with webcam overlay")]
    ScreenrecordingWithWebcam,
    #[command(about = "Take a screenshot")]
    Screenshot {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Extract text from a screenshot region with OCR")]
    Text,
    #[command(name = "webcam-list", about = "List webcam devices")]
    WebcamList,
    #[command(name = "webcam-resize", about = "Resize the webcam overlay")]
    WebcamResize {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ClipboardCmd {
    #[command(about = "Open a clipboard history entry")]
    Open {
        #[arg(long, help = "Index into clipboard history")]
        history_index: u64,
    },
    #[command(name = "paste-file", about = "Copy a file to clipboard and paste it")]
    PasteFile {
        #[arg(long, help = "Only copy, don't paste")]
        copy_only: bool,
        #[arg(help = "MIME type")]
        mime_type: String,
        #[arg(help = "File path")]
        path: String,
    },
    #[command(name = "paste-text", about = "Copy text to clipboard and optionally type it")]
    PasteText {
        #[arg(long, help = "Use Shift+Insert to paste")]
        shift_insert: bool,
        #[arg(long, help = "Only copy, don't paste")]
        copy_only: bool,
        #[arg(long, help = "Use history entry at index")]
        history_index: Option<u64>,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        text: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum CrashCmd {
    #[command(about = "Silence crash notifications for a program")]
    Mute {
        #[arg(help = "Program name (omit to list)")]
        program: Option<String>,
        #[arg(help = "on|off|toggle", default_value = "on")]
        action: String,
    },
    #[command(about = "Watch for process crashes and offer AI diagnosis")]
    Watch,
}

#[derive(Subcommand)]
pub enum DebugCmd {
    #[command(about = "Print debugging information")]
    Info {
        #[arg(long, help = "Skip sudo commands")]
        no_sudo: bool,
        #[arg(long, help = "Print log instead of interactive menu")]
        print: bool,
    },
    #[command(about = "Show idle, screensaver, and lock diagnostics")]
    Idle {
        #[arg(help = "Number of log lines", default_value = "200")]
        log_lines: u64,
    },
}

#[derive(Subcommand)]
pub enum DefaultCmd {
    #[command(about = "Set and launch the default coding agent")]
    Agent {
        #[arg(long, help = "Install mode (run inside floating terminal)")]
        install: bool,
        #[arg(help = "Agent name")]
        name: Option<String>,
    },
    #[command(about = "Set the default browser")]
    Browser {
        #[arg(long, help = "Install mode")]
        install: bool,
        #[arg(help = "Browser name")]
        name: Option<String>,
    },
    #[command(about = "Set the default editor")]
    Editor {
        #[arg(long, help = "Install mode")]
        install: bool,
        #[arg(help = "Editor name")]
        name: Option<String>,
    },
    #[command(about = "Set the default terminal")]
    Terminal {
        #[arg(long, help = "Install mode")]
        install: bool,
        #[arg(help = "Terminal name")]
        name: Option<String>,
    },
}

// ── Batch 3 enums ─────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum HyprlandCmd {
    #[command(name = "focus-app", about = "Focus a window by app class/title")]
    FocusApp { app: String },
    #[command(name = "monitor-clamshell", about = "Apply clamshell display state")]
    MonitorClamshell,
    #[command(name = "monitor-external-active", about = "Check if an external monitor is active (exit 0 = active)")]
    MonitorExternalActive,
    #[command(name = "monitor-focused", about = "Print focused monitor name")]
    MonitorFocused,
    #[command(name = "monitor-focused-apple", about = "Check if focused monitor is Apple (exit 0 = yes)")]
    MonitorFocusedApple {
        #[arg(help = "Monitor name (default: focused)")]
        monitor: Option<String>,
    },
    #[command(name = "monitor-internal", about = "Enable/disable/toggle internal display")]
    MonitorInternal {
        #[arg(help = "on|off|toggle|recover")]
        action: String,
    },
    #[command(name = "monitor-internal-mirror", about = "Enable/disable/toggle internal mirror")]
    MonitorInternalMirror {
        #[arg(help = "on|off|toggle|recover")]
        action: String,
    },
    #[command(name = "monitor-laptop", about = "Print built-in laptop display name")]
    MonitorLaptop,
    #[command(name = "monitor-modeless", about = "Check for monitors with no mode (exit 0 = modeless found)")]
    MonitorModeless,
    #[command(name = "monitor-scaling", about = "Show/set/adjust monitor scaling")]
    MonitorScaling {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "monitor-watch", about = "Watch monitor events and reconcile toggles")]
    MonitorWatch,
    #[command(name = "reload-guard", about = "Pause/resume/check Hyprland config auto-reload")]
    ReloadGuard {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "session-locked", about = "Check if session is locked (exit 0 = locked)")]
    SessionLocked,
    #[command(name = "toggle", about = "Toggle a Hyprland feature flag lua file")]
    Toggle {
        flag_name: String,
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(name = "toggle-disabled", about = "Check if hyprland toggle flag is absent (exit 0 = disabled)")]
    ToggleDisabled { flag_name: String },
    #[command(name = "toggle-enabled", about = "Check if hyprland toggle flag exists (exit 0 = enabled)")]
    ToggleEnabled { flag_name: String },
    #[command(name = "window-close-all", about = "Close all windows and go to workspace 1")]
    WindowCloseAll,
    #[command(name = "window-gaps-toggle", about = "Toggle window gaps globally")]
    WindowGapsToggle,
    #[command(name = "window-pop", about = "Float/pin/size/center the active window")]
    WindowPop {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "window-single-square-aspect-toggle", about = "Toggle single-window square aspect ratio")]
    WindowSingleSquareAspectToggle,
    #[command(name = "window-tiled-fullscreen-toggle", about = "Toggle tiled fullscreen for active window")]
    WindowTiledFullscreenToggle,
    #[command(name = "window-transparency-toggle", about = "Toggle opacity for active window")]
    WindowTransparencyToggle,
    #[command(name = "window-width", about = "Save or restore focused window width")]
    WindowWidth {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "workspace-layout-toggle", about = "Toggle workspace layout between dwindle and scrolling")]
    WorkspaceLayoutToggle,
}

#[derive(Subcommand)]
pub enum SystemCmd {
    #[command(name = "factory-reset", about = "Factory reset via btrfs snapshot")]
    FactoryReset {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "factory-reset-finish", about = "Complete factory reset")]
    FactoryResetFinish {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "lid-close", about = "Handle lid close event")]
    LidClose,
    #[command(about = "Lock the session")]
    Lock,
    #[command(about = "Log out of the session")]
    Logout,
    #[command(about = "Reboot the system")]
    Reboot,
    #[command(about = "Shut down the system")]
    Shutdown,
    #[command(name = "sleep-lock", about = "Lock before suspend")]
    SleepLock {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "sleep-monitor", about = "Monitor sleep events and lock before suspend")]
    SleepMonitor {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Print CPU/memory stats")]
    Stats {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Wake displays and restore brightness")]
    Wake,
}

#[derive(Subcommand)]
pub enum LaunchCmd {
    #[command(name = "1password", about = "Launch 1Password or its installer")]
    Onepassword,
    #[command(about = "Show system info with animation")]
    About {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch Battle.net client")]
    Battlenet {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch the default browser")]
    Browser {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "config-editor", about = "Open a config file in the default editor")]
    ConfigEditor {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "discord-community", about = "Open Omarchy Discord community")]
    DiscordCommunity,
    #[command(name = "docker-tui", about = "Open lazydocker TUI")]
    DockerTui,
    #[command(about = "Launch the default editor")]
    Editor {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "floating-terminal-with-presentation", about = "Launch floating terminal with Omarchy presentation")]
    FloatingTerminalWithPresentation {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch Files (Nautilus)")]
    Nautilus,
    #[command(name = "nautilus-cwd", about = "Launch Files in the terminal's current directory")]
    NautilusCwd,
    #[command(about = "Launch OpenClaw gateway/TUI")]
    Openclaw {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "or-focus", about = "Launch or focus an existing window")]
    OrFocus {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "or-focus-tui", about = "Launch or focus a TUI window")]
    OrFocusTui {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "or-focus-webapp", about = "Launch or focus a web app window")]
    OrFocusWebapp {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch screensaver on all monitors")]
    Screensaver {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Start or restart the Omarchy shell")]
    Shell {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Focus or launch Signal")]
    Signal,
    #[command(about = "Focus or launch Spotify")]
    Spotify,
    #[command(about = "Launch a terminal in the current directory")]
    Terminal {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "terminal-herdr", about = "Launch terminal with herdr")]
    TerminalHerdr,
    #[command(name = "terminal-tmux", about = "Launch terminal attached to tmux")]
    TerminalTmux,
    #[command(about = "Launch a TUI command in the default terminal")]
    Tui {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch a URL as a web app")]
    Webapp {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum MigrateCmd {
    #[command(about = "Run pending migrations")]
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Notify user of pending migrations")]
    Notify,
}

#[derive(Subcommand)]
pub enum NetworkCmd {
    #[command(about = "Show or pin the Wi-Fi band")]
    Band {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Print the active Wi-Fi password")]
    Password {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Generate a Wi-Fi QR code")]
    Qr {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Measure internet speed")]
    Speedtest {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Print active network status")]
    Status {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum HookCmd {
    #[command(about = "Run a named hook")]
    Run {
        name: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    #[command(about = "Install a hook file")]
    Install {
        hook_type: String,
        file: String,
    },
}

#[derive(Subcommand)]
pub enum SudoCmd {
    #[command(about = "Check if docker socket is writable")]
    Docker {
        #[arg(long, help = "Check group membership instead of socket writability")]
        configured: bool,
    },
    #[command(about = "Keep sudo credentials alive in the background")]
    Keepalive,
    #[command(about = "Toggle passwordless sudo")]
    Passwordless {
        #[arg(help = "Duration in minutes (default 15)")]
        minutes: Option<u32>,
    },
}
