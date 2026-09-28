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
#[command(name = "omarchy", about = "Omarchy command center")]
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
    #[command(about = "Install commands")]
    Install {
        #[command(subcommand)]
        subcmd: Option<InstallCmd>,
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
    #[command(about = "Omarchy state file management", long_about = "Omarchy state file management\n\nBinary: omarchy-state")]
    State {
        action: String,
        name: String,
    },
    #[command(about = "Done marker management", long_about = "Done marker management\n\nBinary: omarchy-done")]
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
    #[command(about = "Version and channel information")]
    Version {
        #[command(subcommand)]
        subcmd: Option<VersionCmd>,
    },
    #[command(about = "Power profile management")]
    Powerprofiles {
        #[command(subcommand)]
        subcmd: PowerprofilesCmd,
    },
    #[command(about = "On-screen display", long_about = "On-screen display\n\nBinary: omarchy-osd")]
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
    #[command(about = "System update helpers", long_about = "System update helpers\n\nBinary: omarchy-update")]
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
    #[command(about = "Run the Omarchy screensaver", long_about = "Run the Omarchy screensaver\n\nBinary: omarchy-screensaver")]
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
    #[command(about = "Set and show lightweight desktop reminders", long_about = "Set and show lightweight desktop reminders\n\nBinary: omarchy-reminder")]
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
    #[command(about = "Toggle commands")]
    Toggle {
        #[command(subcommand)]
        subcmd: Option<ToggleCmd>,
    },
    #[command(about = "Bluetooth device management")]
    Bluetooth {
        #[command(subcommand)]
        subcmd: BluetoothCmd,
    },
    #[command(about = "Bar configuration", long_about = "Bar configuration\n\nBinary: omarchy-bar")]
    Bar {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Print Omarchy ASCII art logo", long_about = "Print Omarchy ASCII art logo\n\nBinary: omarchy-ascii")]
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
    #[command(name = "shell", about = "Send IPC calls to the Omarchy shell", long_about = "Send IPC calls to the Omarchy shell\n\nBinary: omarchy-shell")]
    ShellIpc {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Run or check pending migrations")]
    Migrate {
        #[command(subcommand)]
        subcmd: MigrateCmd,
    },
    #[command(about = "Show or configure system DNS", long_about = "Show or configure system DNS\n\nBinary: omarchy-dns")]
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
    // ── Batch 4 ──────────────────────────────────────────────────────────────
    #[command(about = "Omarchy release channel management")]
    Channel {
        #[command(subcommand)]
        subcmd: ChannelCmd,
    },
    #[command(about = "Developer tools")]
    Dev {
        #[command(subcommand)]
        subcmd: DevCmd,
    },
    #[command(about = "Hibernation setup and removal")]
    Hibernation {
        #[command(subcommand)]
        subcmd: HibernationCmd,
    },
    #[command(about = "Plymouth boot theme management")]
    Plymouth {
        #[command(subcommand)]
        subcmd: PlymouthCmd,
    },
    #[command(about = "Drive information and management")]
    Drive {
        #[command(subcommand)]
        subcmd: DriveCmd,
    },
    #[command(name = "file-select", about = "Open a file chooser portal")]
    FileSelect {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Interactive menus")]
    Menu {
        #[command(subcommand)]
        subcmd: MenuCmd,
    },
    #[command(name = "openclaw-onboard", about = "OpenClaw onboarding wizard")]
    OpenclawOnboard {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Plugin management")]
    Plugin {
        #[command(subcommand)]
        subcmd: PluginCmd,
    },
    #[command(about = "Refresh configuration files")]
    Refresh {
        #[command(subcommand)]
        subcmd: RefreshCmd,
    },
    #[command(about = "Theme commands")]
    Theme {
        #[command(subcommand)]
        subcmd: ThemeCmd,
    },
    #[command(about = "Transcode media files")]
    Transcode {
        #[command(subcommand)]
        subcmd: TranscodeCmd,
    },
    #[command(about = "Voxtype dictation management")]
    Voxtype {
        #[command(subcommand)]
        subcmd: VoxtypeCmd,
    },
    #[command(name = "webapp-handler", about = "Web app URL handlers")]
    WebappHandler {
        #[command(subcommand)]
        subcmd: WebappHandlerCmd,
    },
    #[command(about = "List available commands")]
    Commands {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        check: bool,
    },
    #[command(about = "Take a screenshot", hide = true,
              long_about = "Take a screenshot\n\nBinary: omarchy-capture-screenshot")]
    Screenshot {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Share content (clipboard, file, or folder)", hide = true,
              long_about = "Share content (clipboard, file, or folder)\n\nBinary: omarchy-share\n\nUsage: omarchy share <clipboard|file|folder> [path...]")]
    Share {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Apply system configuration (install-time plumbing)", hide = true)]
    Apply {
        #[command(subcommand)]
        subcmd: ApplyCmd,
    },
}

#[derive(Subcommand)]
pub enum VersionCmd {
    #[command(name = "show", about = "Show Omarchy version (omarchy version show)")]
    Show,
    #[command(name = "branch", about = "Show current branch - dev checkout only (omarchy version branch)")]
    Branch,
}

#[derive(Subcommand)]
pub enum AudioCmd {
    #[command(name = "output-volume", about = "Adjust output volume and show OSD (omarchy audio output volume)")]
    OutputVolume {
        #[arg(help = "raise|lower|mute-toggle|+N|-N")]
        action: String,
    },
    #[command(name = "output-switch", about = "Switch between audio outputs (omarchy audio output switch)")]
    OutputSwitch,
    #[command(name = "input-mute", about = "Toggle microphone mute (omarchy audio input mute)")]
    InputMute,
    #[command(name = "output-sink", about = "Print the sink that carries volume for an output (omarchy audio output sink)")]
    OutputSink {
        #[arg(help = "Sink name (optional, defaults to current default)")]
        sink: Option<String>,
    },
    #[command(name = "sink-availability", about = "Print PulseAudio sink availability (omarchy audio sink availability)")]
    SinkAvailability,
    #[command(name = "source-switch", about = "Cycle media source (omarchy audio source switch)")]
    SourceSwitch {
        #[arg(help = "next|previous (default: next)")]
        direction: Option<String>,
    },
    #[command(name = "input-set-default", about = "Set default audio input and move streams (omarchy audio input set default)")]
    InputSetDefault {
        node_id: String,
        source_name: String,
    },
    #[command(name = "output-set-default", about = "Set default audio output and move streams (omarchy audio output set default)")]
    OutputSetDefault {
        node_id: String,
        sink_name: String,
    },
    #[command(name = "tuning", about = "Manage speaker tuning (omarchy audio tuning)")]
    Tuning {
        #[arg(help = "on|off|status|match|fronted-sink", default_value = "status")]
        action: String,
        #[arg(long, help = "Force reinstall even if already current")]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum BrightnessCmd {
    #[command(about = "Show or adjust display brightness (omarchy brightness display)")]
    Display {
        #[arg(long, help = "Skip OSD")]
        no_osd: bool,
        #[arg(long, help = "Monitor name")]
        monitor: Option<String>,
        #[arg(help = "+N%|N%-|N%|off|on")]
        step: Option<String>,
    },
    #[command(name = "display-apple", about = "Apple Studio Display brightness (omarchy brightness display apple)")]
    DisplayApple {
        #[arg(long, help = "Skip OSD")]
        no_osd: bool,
        #[arg(help = "+N%|N%-|N%")]
        step: Option<String>,
    },
    #[command(name = "display-ddc", about = "DDC/CI external monitor brightness (omarchy brightness display ddc)")]
    DisplayDdc {
        monitor: String,
        #[arg(help = "+N%|N%-|N%")]
        step: Option<String>,
    },
    #[command(about = "Adjust keyboard backlight (omarchy brightness keyboard)")]
    Keyboard {
        #[arg(long, help = "Skip OSD")]
        no_osd: bool,
        #[arg(help = "up|down|cycle|off|restore", default_value = "up")]
        direction: String,
    },
    #[command(name = "keyboard-mute", about = "Set mic-mute indicator LED (omarchy brightness keyboard mute)")]
    KeyboardMute {
        #[arg(help = "on|off")]
        state: String,
    },
}

#[derive(Subcommand)]
pub enum ToggleCmd {
    #[command(name = "flag", about = "Toggle a named feature flag (omarchy toggle flag)")]
    Flag {
        flag_name: String,
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(about = "Toggle bar visibility (omarchy toggle bar)")]
    Bar {
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(name = "crash-capture", about = "Toggle crash capture (omarchy toggle crash capture)")]
    CrashCapture,
    #[command(about = "Check if a toggle is enabled (omarchy toggle enabled)")]
    Enabled { flag_name: String },
    #[command(name = "fullscreen-desktop", about = "Toggle fullscreen desktop mode (omarchy toggle fullscreen desktop)")]
    FullscreenDesktop {
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(name = "hybrid-gpu", about = "Toggle dedicated vs integrated GPU (omarchy toggle hybrid gpu)")]
    HybridGpu,
    #[command(about = "Toggle idle/stay-awake behavior (omarchy toggle idle)")]
    Idle {
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(name = "input-device", about = "Enable/disable/toggle a Hyprland input device (omarchy toggle input device)")]
    InputDevice {
        kind: String,
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(about = "Toggle nightlight screen temperature (omarchy toggle nightlight)")]
    Nightlight {
        #[arg(long, help = "Print status JSON instead of toggling")]
        status: bool,
    },
    #[command(name = "notification-silencing", about = "Toggle notification do-not-disturb (omarchy toggle notification silencing)")]
    NotificationSilencing,
    #[command(about = "Toggle screensaver availability (omarchy toggle screensaver)")]
    Screensaver,
    #[command(about = "Toggle suspend in system menu (omarchy toggle suspend)")]
    Suspend,
    #[command(about = "Enable/disable/toggle touchpad (omarchy toggle touchpad)")]
    Touchpad {
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(about = "Enable/disable/toggle touchscreen (omarchy toggle touchscreen)")]
    Touchscreen {
        #[arg(default_value = "toggle")]
        action: String,
    },
}

#[derive(Subcommand)]
pub enum BluetoothCmd {
    #[command(about = "Control a Bluetooth device (omarchy bluetooth device)")]
    Device {
        #[arg(help = "pair|connect|disconnect|forget")]
        action: String,
        #[arg(help = "Device MAC address")]
        address: String,
    },
    #[command(about = "Turn Bluetooth on or off (omarchy bluetooth power)")]
    Power {
        #[arg(help = "on|off|toggle|is-on")]
        action: String,
    },
}

#[derive(Subcommand)]
pub enum FontCmd {
    #[command(about = "Show current monospace font (omarchy font current)")]
    Current,
    #[command(about = "List available monospace fonts (omarchy font list)")]
    List,
    #[command(about = "Set system monospace font (omarchy font set)")]
    Set {
        #[arg(help = "Font name")]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum WeatherCmd {
    #[command(about = "Return weather condition icon adjusted for sunrise/sunset (omarchy weather icon)")]
    Icon,
    #[command(about = "Show or set weather location (omarchy weather location)")]
    Location {
        #[arg(long, help = "Location name")]
        set: Option<String>,
        #[arg(long, help = "Coordinates lat,lon (used with --set)")]
        coords: Option<String>,
        #[arg(long, help = "Return to IP auto-detect")]
        clear: bool,
    },
    #[command(about = "Return formatted weather status string (omarchy weather status)")]
    Status,
}

#[derive(Subcommand)]
pub enum HwCmd {
    #[command(about = "Detect all hardware and print JSON summary (omarchy hw detect)")]
    Detect,
    #[command(about = "Check if a hardware feature is present (omarchy hw check)")]
    Check {
        #[arg(value_parser = PossibleValuesParser::new(HW_CHECKS))]
        name: String,
        #[arg(help = "Pattern for 'match' check")]
        pattern: Option<String>,
    },
    #[command(about = "Query dynamic hardware state (omarchy hw state)")]
    State {
        #[arg(value_parser = PossibleValuesParser::new(HW_STATES))]
        name: String,
    },
    #[command(name = "recover-internal-monitor", about = "Clear internal-monitor-disable toggle if no external display (omarchy hw recover internal monitor)")]
    RecoverInternalMonitor,
    // Per-hardware subcommands (exec-delegate to omarchy-hw-* scripts)
    #[command(name = "asus-rog", about = "Check ASUS ROG hardware (omarchy hw asus rog)")]
    AsusRog,
    #[command(name = "asus-expertbook-b9406", about = "Check ASUS ExpertBook B9406 (omarchy hw asus expertbook b9406)")]
    AsusExpertbookB9406,
    #[command(name = "asus-zenbook-ux5406aa", about = "Check ASUS Zenbook UX5406AA (omarchy hw asus zenbook ux5406aa)")]
    AsusZenbookUx5406aa,
    #[command(name = "clamshell", about = "Check clamshell mode (omarchy hw clamshell)")]
    Clamshell,
    #[command(name = "dell-xps13-sidecar-amps", about = "Check Dell XPS13 sidecar amps (omarchy hw dell xps13 sidecar amps)")]
    DellXps13SidecarAmps,
    #[command(name = "dell-xps-haptic-touchpad", about = "Check Dell XPS haptic touchpad (omarchy hw dell xps haptic touchpad)")]
    DellXpsHapticTouchpad,
    #[command(name = "dell-xps-oled", about = "Check Dell XPS OLED display (omarchy hw dell xps oled)")]
    DellXpsOled,
    #[command(name = "display", about = "Check display state (omarchy hw display)")]
    Display,
    #[command(name = "elgato-camlink-4k", about = "Check Elgato Camlink 4K (omarchy hw elgato camlink 4k)")]
    ElegatoCamlink4k,
    #[command(name = "external-monitors", about = "Check external monitors (omarchy hw external monitors)")]
    ExternalMonitors,
    #[command(name = "fingerprint", about = "Check fingerprint reader (omarchy hw fingerprint)")]
    Fingerprint,
    #[command(name = "framework16", about = "Check Framework 16 hardware (omarchy hw framework16)")]
    Framework16,
    #[command(name = "hybrid-gpu", about = "Check hybrid GPU (omarchy hw hybrid gpu)")]
    HybridGpu,
    #[command(name = "intel", about = "Check Intel CPU (omarchy hw intel)")]
    Intel,
    #[command(name = "intel-ptl", about = "Check Intel PTL CPU (omarchy hw intel ptl)")]
    IntelPtl,
    #[command(name = "intel-sof", about = "Check Intel SOF audio (omarchy hw intel sof)")]
    IntelSof,
    #[command(name = "laptop", about = "Check laptop hardware (omarchy hw laptop)")]
    Laptop,
    #[command(name = "laptop-closed", about = "Check laptop lid closed (omarchy hw laptop closed)")]
    LaptopClosed,
    #[command(name = "match", about = "Match hardware pattern (omarchy hw match)")]
    Match {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "nvidia", about = "Check NVIDIA GPU (omarchy hw nvidia)")]
    Nvidia,
    #[command(name = "nvidia-gsp", about = "Check NVIDIA GSP firmware (omarchy hw nvidia gsp)")]
    NvidiaGsp,
    #[command(name = "nvidia-without-gsp", about = "Check NVIDIA without GSP (omarchy hw nvidia without gsp)")]
    NvidiaWithoutGsp,
    #[command(name = "surface", about = "Check Microsoft Surface (omarchy hw surface)")]
    Surface,
    #[command(name = "touchpad", about = "Check touchpad state (omarchy hw touchpad)")]
    Touchpad,
    #[command(name = "touchscreen", about = "Check touchscreen (omarchy hw touchscreen)")]
    Touchscreen,
    #[command(name = "vulkan", about = "Check Vulkan support (omarchy hw vulkan)")]
    Vulkan,
    #[command(name = "webcam", about = "Check webcam (omarchy hw webcam)")]
    Webcam,
}

#[derive(Subcommand)]
pub enum InstallCmd {
    #[command(name = "gaming-steam", about = "Install Steam and graphics drivers (omarchy install gaming steam)")]
    GamingSteam,
    #[command(name = "gaming-heroic", about = "Install Heroic Games Launcher (omarchy install gaming heroic)")]
    GamingHeroic,
    #[command(name = "gaming-lutris", about = "Install Lutris with Wine (omarchy install gaming lutris)")]
    GamingLutris,
    #[command(name = "gaming-retroarch", about = "Install RetroArch with full core set (omarchy install gaming retroarch)")]
    GamingRetroarch,
    #[command(name = "gaming-xbox-controllers", about = "Install Xbox controller support (omarchy install gaming xbox controllers)")]
    GamingXboxControllers,
    #[command(name = "gaming-xbox-cloud", about = "Install Xbox Cloud Gaming web app (omarchy install gaming xbox cloud)")]
    GamingXboxCloud,
    #[command(name = "gaming-battlenet", about = "Install Battle.net via umu-launcher (omarchy install gaming battlenet)")]
    GamingBattlenet,
    #[command(name = "gaming-geforce-now", about = "Install GeForce NOW (omarchy install gaming geforce now)")]
    GamingGeforceNow,
    #[command(name = "gaming-gpu-lib32", about = "Install lib32 graphics drivers (omarchy install gaming gpu lib32)")]
    GamingGpuLib32,
    #[command(name = "editor-helix", about = "Install Helix editor (omarchy install editor helix)")]
    EditorHelix,
    #[command(name = "editor-vscode", about = "Install VS Code (omarchy install editor vscode)")]
    EditorVscode,
    #[command(name = "editor-emacs", about = "Install Emacs (omarchy install editor emacs)")]
    EditorEmacs,
    #[command(name = "editor-zed", about = "Install Zed editor (omarchy install editor zed)")]
    EditorZed,
    #[command(name = "ai-claude", about = "Install Claude desktop app (omarchy install ai claude)")]
    AiClaude,
    #[command(name = "ai-hermes", about = "Install Hermes desktop app (omarchy install ai hermes)")]
    AiHermes,
    #[command(name = "ai-t3-code", about = "Install T3 Code (omarchy install ai t3 code)")]
    AiT3code,
    #[command(name = "ai-chatgpt", about = "Install ChatGPT desktop app (omarchy install ai chatgpt)")]
    AiChatgpt,
    #[command(name = "browser", about = "Install a supported browser (omarchy install browser)")]
    Browser {
        #[arg(value_enum)]
        name: BrowserName,
    },
    #[command(name = "dev-env", about = "Install a development environment (omarchy install dev env)")]
    DevEnv {
        #[arg(value_enum)]
        name: DevEnvName,
    },
    #[command(name = "terminal", about = "Install a terminal emulator (omarchy install terminal)")]
    Terminal {
        #[arg(value_enum)]
        name: TerminalName,
    },
    #[command(name = "chromium-claude", about = "Install Claude extension for Chromium browsers (omarchy install chromium claude)")]
    ChromiumClaude,
    #[command(name = "chromium-copy-url", about = "Install Copy URL native messaging host (omarchy install chromium copy url)")]
    ChromiumCopyUrl,
    #[command(name = "chromium-ytdlp", about = "Install yt-dlp native messaging host (omarchy install chromium ytdlp)")]
    ChromiumYtdlp,
    #[command(name = "chromium-google-account", about = "Enable Google account sign-in in Chromium (omarchy install chromium google account)")]
    ChromiumGoogleAccount,
    #[command(name = "font", about = "Install a Nerd Font and switch the system to it (omarchy install font)")]
    Font {
        #[arg(help = "Display name (e.g. 'Cascadia Mono')")]
        name: String,
        #[arg(help = "Package name (e.g. ttf-cascadia-mono-nerd)")]
        package: String,
        #[arg(help = "Font family name for omarchy-font-set (e.g. 'CaskaydiaMono Nerd Font')")]
        family: String,
    },
    #[command(name = "and-launch", about = "Install packages then launch a desktop app (omarchy install and launch)")]
    AndLaunch {
        #[arg(help = "Display name")]
        name: String,
        #[arg(help = "Space-separated package list")]
        packages: String,
        #[arg(help = "Desktop application ID for gtk-launch")]
        desktop_id: String,
    },
    #[command(name = "app", about = "Install one or more packages (omarchy install app)")]
    App {
        #[arg(help = "Display name")]
        name: String,
        #[arg(help = "Space-separated package list")]
        packages: String,
    },
    #[command(name = "openclaw-cli", about = "Ensure the OpenClaw CLI is installed (omarchy install openclaw cli)")]
    OpenclawCli {
        #[arg(long, help = "Check if installed (exit 0 = present)", conflicts_with = "now")]
        check: bool,
        #[arg(long, help = "Install if missing (default)")]
        now: bool,
    },
    #[command(name = "docker-dbs", about = "Install a database in a Docker container (omarchy install docker dbs)")]
    DockerDbs {
        #[arg(help = "Database names to install (interactive if omitted)")]
        dbs: Vec<String>,
    },
    #[command(name = "preinstalls", about = "Restore preinstalled Omarchy applications (omarchy install preinstalls)")]
    Preinstalls,
    #[command(name = "service-once", about = "Install the ONCE service and launch it (omarchy install service once)")]
    ServiceOnce,
    #[command(name = "voxtype", about = "Install and configure Voxtype dictation (omarchy install voxtype)")]
    Voxtype,
    #[command(name = "tui", about = "Create a desktop launcher for a terminal UI app (omarchy install tui)")]
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
    #[command(name = "webapp", about = "Create a desktop launcher for a web app (omarchy install webapp)")]
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
    #[command(name = "ai-openclaw", about = "Install the OpenClaw agent platform (omarchy install ai openclaw)")]
    AiOpenclaw,
    #[command(name = "hermes-cli", about = "Ensure the Hermes agent CLI is installed (omarchy install hermes cli)")]
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
    #[command(name = "gaming-steam", about = "Remove Steam and its data (omarchy remove gaming steam)")]
    GamingSteam,
    #[command(name = "gaming-heroic", about = "Remove Heroic Games Launcher and its data (omarchy remove gaming heroic)")]
    GamingHeroic,
    #[command(name = "gaming-lutris", about = "Remove Lutris, Wine, and their data (omarchy remove gaming lutris)")]
    GamingLutris,
    #[command(name = "gaming-retroarch", about = "Remove RetroArch and all cores (omarchy remove gaming retroarch)")]
    GamingRetroarch,
    #[command(name = "gaming-minecraft", about = "Remove Minecraft launcher and its data (omarchy remove gaming minecraft)")]
    GamingMinecraft,
    #[command(name = "gaming-xbox-controllers", about = "Remove Xbox controller support (omarchy remove gaming xbox controllers)")]
    GamingXboxControllers,
    #[command(name = "gaming-xbox-cloud", about = "Remove Xbox Cloud Gaming web app (omarchy remove gaming xbox cloud)")]
    GamingXboxCloud,
    #[command(name = "gaming-battlenet", about = "Remove Battle.net and its prefix (omarchy remove gaming battlenet)")]
    GamingBattlenet,
    #[command(name = "gaming-geforce-now", about = "Remove GeForce NOW (omarchy remove gaming geforce now)")]
    GamingGeforceNow,
    #[command(name = "ai-claude", about = "Remove Claude desktop app (omarchy remove ai claude)")]
    AiClaude,
    #[command(name = "ai-hermes", about = "Remove Hermes desktop app (omarchy remove ai hermes)")]
    AiHermes,
    #[command(name = "ai-t3-code", about = "Remove T3 Code (omarchy remove ai t3 code)")]
    AiT3code,
    #[command(name = "ai-ollama", about = "Remove Ollama and all models (omarchy remove ai ollama)")]
    AiOllama,
    #[command(name = "ai-chatgpt", about = "Remove ChatGPT desktop app (omarchy remove ai chatgpt)")]
    AiChatgpt,
    #[command(name = "ai-lm-studio", about = "Remove LM Studio and all models (omarchy remove ai lm studio)")]
    AiLmStudio,
    #[command(name = "ai-grok-bot", about = "Remove Grok Bot (omarchy remove ai grok bot)")]
    AiGrokBot,
    #[command(name = "ai-perplexity", about = "Remove Perplexity desktop app (omarchy remove ai perplexity)")]
    AiPerplexity,
    #[command(name = "browser", about = "Remove a browser (omarchy remove browser)")]
    Browser {
        #[arg(value_enum)]
        name: BrowserName,
    },
    #[command(name = "dev-env", about = "Remove a development environment (omarchy remove dev env)")]
    DevEnv {
        #[arg(value_enum)]
        name: DevEnvName,
    },
    #[command(name = "preinstalls", about = "Remove preinstalled Omarchy applications (omarchy remove preinstalls)")]
    Preinstalls,
    #[command(name = "voxtype", about = "Remove Voxtype dictation (omarchy remove voxtype)")]
    Voxtype,
    #[command(name = "tui", about = "Remove a terminal UI desktop launcher (omarchy remove tui)")]
    Tui {
        #[arg(help = "App name (interactive if omitted)")]
        name: Option<String>,
        #[arg(long, help = "Remove all TUI launchers", conflicts_with = "name")]
        all: bool,
    },
    #[command(name = "webapp", about = "Remove a web app desktop launcher (omarchy remove webapp)")]
    Webapp {
        #[arg(help = "App name (interactive if omitted)")]
        name: Option<String>,
        #[arg(long, help = "Remove all web app launchers", conflicts_with = "name")]
        all: bool,
    },
    #[command(name = "launcher-entry", about = "Remove or uninstall a launcher entry (omarchy remove launcher entry)")]
    LauncherEntry {
        #[arg(help = "Desktop application ID")]
        desktop_id: String,
        #[arg(help = "Display name (optional)")]
        entry_name: Option<String>,
    },
    #[command(name = "ai-openclaw", about = "Remove the OpenClaw agent platform (omarchy remove ai openclaw)")]
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
    #[command(about = "Edit user configuration in $EDITOR (omarchy config edit)")]
    Edit,
    #[command(about = "Show current user configuration (omarchy config show)")]
    Show,
    #[command(about = "Validate configuration without applying (omarchy config check)")]
    Check,
}

#[derive(Subcommand)]
pub enum ServiceCmd {
    #[command(about = "Enable a service (omarchy service enable)")]
    Enable {
        #[arg(value_parser = PossibleValuesParser::new(services::KNOWN))]
        name: String,
    },
    #[command(about = "Disable a service (omarchy service disable)")]
    Disable {
        #[arg(value_parser = PossibleValuesParser::new(services::KNOWN))]
        name: String,
    },
    #[command(about = "List available and enabled services (omarchy service list)")]
    List,
    #[command(about = "Check if a service daemon is running (omarchy service active)")]
    Active {
        #[arg(value_parser = PossibleValuesParser::new(services::KNOWN))]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum PkgCmd {
    #[command(about = "Add a package (omarchy pkg add)", long_about = "Add a package\n\nBinary: omarchy-pkg-add\n\nUsage: omarchy pkg add <package>")]
    Add {
        name: String,
    },
    #[command(about = "Remove a package (omarchy pkg drop)")]
    Drop {
        name: String,
    },
    #[command(about = "List installed packages (omarchy pkg list)")]
    List,
    #[command(about = "Search nixpkgs for packages (omarchy pkg search)")]
    Search { query: String },
    #[command(about = "Sync system configuration (omarchy pkg sync)")]
    Sync,
    #[command(about = "Check if a package is installed (omarchy pkg present)")]
    Present { name: String },
    #[command(about = "Check if a package is missing (omarchy pkg missing)")]
    Missing { name: String },
    #[command(about = "Resolve package name to nixpkgs attribute (omarchy pkg resolve)")]
    Resolve { name: String },
}

// ---- New command group enums ----

#[derive(Subcommand)]
pub enum RestartCmd {
    #[command(about = "Restart an application (omarchy restart app)")]
    App {
        name: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    #[command(about = "Restart audio services and recover stuck USB audio (omarchy restart audio)")]
    Audio,
    #[command(about = "Unblock Bluetooth (omarchy restart bluetooth)")]
    Bluetooth,
    #[command(about = "Send SIGUSR2 to btop (omarchy restart btop)")]
    Btop,
    #[command(about = "Print gum environment variables from current theme (omarchy restart gum)")]
    Gum,
    #[command(about = "Send SIGUSR1 to helix (omarchy restart helix)")]
    Helix,
    #[command(about = "Reload herdr server config (omarchy restart herdr)")]
    Herdr,
    #[command(about = "Reload Hyprland config (omarchy restart hyprctl)")]
    Hyprctl,
    #[command(about = "Restart hyprsunset (omarchy restart hyprsunset)")]
    Hyprsunset,
    #[command(about = "Send SIGUSR2 to opencode (omarchy restart opencode)")]
    Opencode,
    #[command(about = "Restart the Omarchy shell (omarchy restart shell)")]
    Shell,
    #[command(about = "Reload terminal configs (omarchy restart terminal)")]
    Terminal,
    #[command(about = "Reload tmux config (omarchy restart tmux)")]
    Tmux,
    #[command(about = "Rebind i2c HID touchpad driver (omarchy restart trackpad)")]
    Trackpad,
    #[command(about = "Restart WiFi networking (omarchy restart wifi)")]
    Wifi,
    #[command(about = "Restart fcitx5 for XCompose support (omarchy restart xcompose)")]
    Xcompose,
}

#[derive(Subcommand)]
pub enum NotificationCmd {
    #[command(about = "Send a battery status notification (omarchy notification battery)")]
    Battery,
    #[command(about = "Send a time notification (omarchy notification time)")]
    Time,
    #[command(about = "Toggle weather notification (omarchy notification weather)")]
    Weather,
    #[command(about = "Dismiss a notification by summary (omarchy notification dismiss)")]
    Dismiss { summary: String },
    #[command(about = "Wait for the notification server to be ready (omarchy notification wait)")]
    Wait {
        #[arg(default_value = "10", help = "Timeout in seconds")]
        seconds: u64,
    },
    #[command(about = "Send a desktop notification (omarchy notification send)")]
    Send {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum BatteryCmd {
    #[command(about = "Check if a battery is present (omarchy battery present)")]
    Present,
    #[command(about = "Send a low battery notification (omarchy battery low)")]
    Low {
        #[arg(help = "Current battery percentage")]
        percentage: u32,
    },
    #[command(about = "Print battery status (omarchy battery status)")]
    Status {
        #[arg(long, help = "Output tab-separated fields for shell consumption")]
        shell: bool,
    },
}

#[derive(Subcommand)]
pub enum PowerCmd {
    #[command(about = "Check if AC power is present (omarchy power present)")]
    Present,
}

#[derive(Subcommand)]
pub enum CmdCheckCmd {
    #[command(about = "Check if any command is missing from PATH (omarchy cmd missing)")]
    Missing {
        #[arg(required = true)]
        cmds: Vec<String>,
    },
    #[command(about = "Check if all commands are present in PATH (omarchy cmd present)")]
    Present {
        #[arg(required = true)]
        cmds: Vec<String>,
    },
    #[command(name = "terminal-cwd", about = "Print CWD of active terminal (omarchy cmd terminal cwd)")]
    TerminalCwd,
}

#[derive(Subcommand)]
pub enum PowerprofilesCmd {
    #[command(about = "Initialize power profile based on current power source (omarchy powerprofiles init)")]
    Init,
    #[command(about = "List available power profiles (omarchy powerprofiles list)")]
    List {
        #[arg(long, help = "Include active state (tab-separated)")]
        active_state: bool,
    },
    #[command(about = "Set the active power profile (omarchy powerprofiles set)")]
    Set {
        #[arg(help = "Mode: autodetect, ac, or battery")]
        action: Option<String>,
        #[arg(help = "Profile name to set and save")]
        profile: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum TailscaleCmd {
    #[command(about = "Receive files via Taildrop (omarchy tailscale receive)")]
    Receive {
        #[arg(long, help = "Exit after receiving one batch")]
        once: bool,
        #[arg(help = "Download directory")]
        dir: Option<String>,
    },
    #[command(about = "Send files via Taildrop (omarchy tailscale send)")]
    Send {
        #[arg(help = "Target machine")]
        machine: String,
        #[arg(help = "Files to send")]
        files: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum UpdateCmd {
    #[command(about = "Pull latest dev checkout (omarchy update dev)")]
    Dev,
    #[command(about = "Update firmware via fwupd (omarchy update firmware)")]
    Firmware,
    #[command(about = "Manage the update lock (omarchy update lock)")]
    Lock {
        action: String,
        #[arg(trailing_var_arg = true)]
        cmd_args: Vec<String>,
    },
    #[command(name = "requires-free-space", about = "Check for 10GiB free space (omarchy update requires free space)")]
    RequiresFreeSpace,
    #[command(about = "Check for updates and refresh indicators (omarchy update status)")]
    Status,
    #[command(name = "stay-awake", about = "Inhibit sleep during updates (omarchy update stay awake)")]
    StayAwake { action: String },
    #[command(about = "Sync system clock (omarchy update time)")]
    Time,
    #[command(name = "user-notify", about = "Delegate to omarchy-migrate-notify (omarchy update user notify)")]
    UserNotify {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum SetupCmd {
    #[command(name = "direct-boot", about = "Manage EFI direct boot entry for Omarchy UKI (omarchy setup direct boot)")]
    DirectBoot,
}

#[derive(Subcommand)]
pub enum AgentCmd {
    #[command(about = "Launch the default coding agent (omarchy agent run)")]
    Run {
        #[arg(long, help = "Run agent inline (no TUI wrapper)")]
        inline: bool,
        #[arg(long, help = "Pick agent if none set")]
        pick: bool,
        #[arg(long, help = "Pass a prompt to the agent")]
        prompt: Option<String>,
    },
    #[command(about = "Diagnose a crashed process with the default coding agent (omarchy agent crash)")]
    Crash {
        pid: String,
        comm: Option<String>,
        exe: Option<String>,
        signal: Option<String>,
    },
    #[command(about = "Launch the default coding agent with a prompt (omarchy agent prompt)")]
    Prompt {
        #[arg(long, help = "Run agent inline (no TUI wrapper)")]
        inline: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        prompt: Vec<String>,
    },
    #[command(name = "usage-claude", about = "Show Claude API usage (omarchy agent usage claude)")]
    UsageClaude {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "usage-codex", about = "Show Codex usage (omarchy agent usage codex)")]
    UsageCodex {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "usage-fireworks", about = "Show Fireworks usage (omarchy agent usage fireworks)")]
    UsageFireworks {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "usage-update", about = "Regenerate agent usage data files (omarchy agent usage update)")]
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
    #[command(about = "Edit, set, or reset About branding (omarchy branding about)")]
    About {
        #[arg(help = "image|text|reset")]
        mode: String,
    },
    #[command(about = "Edit, set, or reset screensaver branding (omarchy branding screensaver)")]
    Screensaver {
        #[arg(help = "image|text|reset")]
        mode: String,
    },
}

#[derive(Subcommand)]
pub enum CaptureCmd {
    #[command(about = "Decode a QR code from a screenshot region (omarchy capture qr)")]
    Qr,
    #[command(about = "Pick a screen region (omarchy capture region)")]
    Region {
        #[arg(help = "region|windows|smart|fullscreen")]
        mode: Option<String>,
        #[arg(long, help = "Leave freeze running")]
        keep_freeze: bool,
        #[arg(long, help = "Print monitor:NAME when matched")]
        match_monitor: bool,
    },
    #[command(about = "Start or stop screen recording (omarchy capture screenrecording)")]
    Screenrecording {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "screenrecording-with-webcam", about = "Screen recording with webcam overlay (omarchy capture screenrecording with webcam)")]
    ScreenrecordingWithWebcam,
    #[command(about = "Take a screenshot (omarchy capture screenshot)", long_about = "Take a screenshot\n\nBinary: omarchy-capture-screenshot")]
    Screenshot {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Extract text from a screenshot region with OCR (omarchy capture text)")]
    Text,
    #[command(name = "webcam-list", about = "List webcam devices (omarchy capture webcam list)")]
    WebcamList,
    #[command(name = "webcam-resize", about = "Resize the webcam overlay (omarchy capture webcam resize)")]
    WebcamResize {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ClipboardCmd {
    #[command(about = "Open a clipboard history entry (omarchy clipboard open)")]
    Open {
        #[arg(long, help = "Index into clipboard history")]
        history_index: u64,
    },
    #[command(name = "paste-file", about = "Copy a file to clipboard and paste it (omarchy clipboard paste file)")]
    PasteFile {
        #[arg(long, help = "Only copy, don't paste")]
        copy_only: bool,
        #[arg(help = "MIME type")]
        mime_type: String,
        #[arg(help = "File path")]
        path: String,
    },
    #[command(name = "paste-text", about = "Copy text to clipboard and optionally type it (omarchy clipboard paste text)")]
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
    #[command(about = "Silence crash notifications for a program (omarchy crash mute)")]
    Mute {
        #[arg(help = "Program name (omit to list)")]
        program: Option<String>,
        #[arg(help = "on|off|toggle", default_value = "on")]
        action: String,
    },
    #[command(about = "Watch for process crashes and offer AI diagnosis (omarchy crash watch)")]
    Watch,
}

#[derive(Subcommand)]
pub enum DebugCmd {
    #[command(about = "Print debugging information (omarchy debug info)")]
    Info {
        #[arg(long, help = "Skip sudo commands")]
        no_sudo: bool,
        #[arg(long, help = "Print log instead of interactive menu")]
        print: bool,
    },
    #[command(about = "Show idle, screensaver, and lock diagnostics (omarchy debug idle)")]
    Idle {
        #[arg(help = "Number of log lines", default_value = "200")]
        log_lines: u64,
    },
}

#[derive(Subcommand)]
pub enum DefaultCmd {
    #[command(about = "Set and launch the default coding agent (omarchy default agent)")]
    Agent {
        #[arg(long, help = "Install mode (run inside floating terminal)")]
        install: bool,
        #[arg(help = "Agent name")]
        name: Option<String>,
    },
    #[command(about = "Set the default browser (omarchy default browser)")]
    Browser {
        #[arg(long, help = "Install mode")]
        install: bool,
        #[arg(help = "Browser name")]
        name: Option<String>,
    },
    #[command(about = "Set the default editor (omarchy default editor)")]
    Editor {
        #[arg(long, help = "Install mode")]
        install: bool,
        #[arg(help = "Editor name")]
        name: Option<String>,
    },
    #[command(about = "Set the default terminal (omarchy default terminal)")]
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
    #[command(name = "focus-app", about = "Focus a window by app class/title (omarchy hyprland focus app)")]
    FocusApp { app: String },
    #[command(name = "monitor-clamshell", about = "Apply clamshell display state (omarchy hyprland monitor clamshell)")]
    MonitorClamshell,
    #[command(name = "monitor-external-active", about = "Check if an external monitor is active (omarchy hyprland monitor external active)")]
    MonitorExternalActive,
    #[command(name = "monitor-focused", about = "Print focused monitor name (omarchy hyprland monitor focused)")]
    MonitorFocused,
    #[command(name = "monitor-focused-apple", about = "Check if focused monitor is Apple (omarchy hyprland monitor focused apple)")]
    MonitorFocusedApple {
        #[arg(help = "Monitor name (default: focused)")]
        monitor: Option<String>,
    },
    #[command(name = "monitor-internal", about = "Enable/disable/toggle internal display (omarchy hyprland monitor internal)")]
    MonitorInternal {
        #[arg(help = "on|off|toggle|recover")]
        action: String,
    },
    #[command(name = "monitor-internal-mirror", about = "Enable/disable/toggle internal mirror (omarchy hyprland monitor internal mirror)")]
    MonitorInternalMirror {
        #[arg(help = "on|off|toggle|recover")]
        action: String,
    },
    #[command(name = "monitor-laptop", about = "Print built-in laptop display name (omarchy hyprland monitor laptop)")]
    MonitorLaptop,
    #[command(name = "monitor-modeless", about = "Check for monitors with no mode (omarchy hyprland monitor modeless)")]
    MonitorModeless,
    #[command(name = "monitor-scaling", about = "Show/set/adjust monitor scaling (omarchy hyprland monitor scaling)")]
    MonitorScaling {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "monitor-watch", about = "Watch monitor events and reconcile toggles (omarchy hyprland monitor watch)")]
    MonitorWatch,
    #[command(name = "reload-guard", about = "Pause/resume/check Hyprland config auto-reload (omarchy hyprland reload guard)")]
    ReloadGuard {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "session-locked", about = "Check if session is locked (omarchy hyprland session locked)")]
    SessionLocked,
    #[command(name = "toggle", about = "Toggle a Hyprland feature flag lua file (omarchy hyprland toggle)")]
    Toggle {
        flag_name: String,
        #[arg(default_value = "toggle")]
        action: String,
    },
    #[command(name = "toggle-disabled", about = "Check if hyprland toggle flag is absent (omarchy hyprland toggle disabled)")]
    ToggleDisabled { flag_name: String },
    #[command(name = "toggle-enabled", about = "Check if hyprland toggle flag exists (omarchy hyprland toggle enabled)")]
    ToggleEnabled { flag_name: String },
    #[command(name = "window-close-all", about = "Close all windows and go to workspace 1 (omarchy hyprland window close all)")]
    WindowCloseAll,
    #[command(name = "window-gaps-toggle", about = "Toggle window gaps globally (omarchy hyprland window gaps toggle)")]
    WindowGapsToggle,
    #[command(name = "window-pop", about = "Float/pin/size/center the active window (omarchy hyprland window pop)")]
    WindowPop {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "window-single-square-aspect-toggle", about = "Toggle single-window square aspect ratio (omarchy hyprland window single square aspect toggle)")]
    WindowSingleSquareAspectToggle,
    #[command(name = "window-tiled-fullscreen-toggle", about = "Toggle tiled fullscreen for active window (omarchy hyprland window tiled fullscreen toggle)")]
    WindowTiledFullscreenToggle,
    #[command(name = "window-transparency-toggle", about = "Toggle opacity for active window (omarchy hyprland window transparency toggle)")]
    WindowTransparencyToggle,
    #[command(name = "window-width", about = "Save or restore focused window width (omarchy hyprland window width)")]
    WindowWidth {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "workspace-layout-toggle", about = "Toggle workspace layout between dwindle and scrolling (omarchy hyprland workspace layout toggle)")]
    WorkspaceLayoutToggle,
}

#[derive(Subcommand)]
pub enum SystemCmd {
    #[command(name = "factory-reset", about = "Factory reset via btrfs snapshot (omarchy system factory reset)")]
    FactoryReset {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "factory-reset-finish", about = "Complete factory reset (omarchy system factory reset finish)")]
    FactoryResetFinish {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "lid-close", about = "Handle lid close event (omarchy system lid close)")]
    LidClose,
    #[command(about = "Lock the session (omarchy system lock)")]
    Lock,
    #[command(about = "Log out of the session (omarchy system logout)")]
    Logout,
    #[command(about = "Reboot the system (omarchy system reboot)", long_about = "Reboot the system\n\nBinary: omarchy-system-reboot")]
    Reboot,
    #[command(about = "Shut down the system (omarchy system shutdown)")]
    Shutdown,
    #[command(name = "sleep-lock", about = "Lock before suspend (omarchy system sleep lock)")]
    SleepLock {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "sleep-monitor", about = "Monitor sleep events and lock before suspend (omarchy system sleep monitor)")]
    SleepMonitor {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Print CPU/memory stats (omarchy system stats)")]
    Stats {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Wake displays and restore brightness (omarchy system wake)")]
    Wake,
}

#[derive(Subcommand)]
pub enum LaunchCmd {
    #[command(name = "1password", about = "Launch 1Password or its installer (omarchy launch 1password)")]
    Onepassword,
    #[command(about = "Show system info with animation (omarchy launch about)")]
    About {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch Battle.net client (omarchy launch battlenet)")]
    Battlenet {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch the default browser (omarchy launch browser)")]
    Browser {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "config-editor", about = "Open a config file in the default editor (omarchy launch config editor)")]
    ConfigEditor {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "discord-community", about = "Open Omarchy Discord community (omarchy launch discord community)")]
    DiscordCommunity,
    #[command(name = "docker-tui", about = "Open lazydocker TUI (omarchy launch docker tui)")]
    DockerTui,
    #[command(about = "Launch the default editor (omarchy launch editor)")]
    Editor {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "floating-terminal-with-presentation", about = "Launch floating terminal with Omarchy presentation (omarchy launch floating terminal with presentation)")]
    FloatingTerminalWithPresentation {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch Files (Nautilus) (omarchy launch nautilus)")]
    Nautilus,
    #[command(name = "nautilus-cwd", about = "Launch Files in the terminal's current directory (omarchy launch nautilus cwd)")]
    NautilusCwd,
    #[command(about = "Launch OpenClaw gateway/TUI (omarchy launch openclaw)")]
    Openclaw {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "or-focus", about = "Launch or focus an existing window (omarchy launch or focus)")]
    OrFocus {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "or-focus-tui", about = "Launch or focus a TUI window (omarchy launch or focus tui)")]
    OrFocusTui {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "or-focus-webapp", about = "Launch or focus a web app window (omarchy launch or focus webapp)")]
    OrFocusWebapp {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch screensaver on all monitors (omarchy launch screensaver)")]
    Screensaver {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Start or restart the Omarchy shell (omarchy launch shell)")]
    Shell {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Focus or launch Signal (omarchy launch signal)")]
    Signal,
    #[command(about = "Focus or launch Spotify (omarchy launch spotify)")]
    Spotify,
    #[command(about = "Launch a terminal in the current directory (omarchy launch terminal)")]
    Terminal {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "terminal-herdr", about = "Launch terminal with herdr (omarchy launch terminal herdr)")]
    TerminalHerdr,
    #[command(name = "terminal-tmux", about = "Launch terminal attached to tmux (omarchy launch terminal tmux)")]
    TerminalTmux,
    #[command(about = "Launch a TUI command in the default terminal (omarchy launch tui)")]
    Tui {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Launch a URL as a web app (omarchy launch webapp)")]
    Webapp {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum MigrateCmd {
    #[command(about = "Run pending migrations (omarchy migrate run)")]
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Notify user of pending migrations (omarchy migrate notify)")]
    Notify,
}

#[derive(Subcommand)]
pub enum NetworkCmd {
    #[command(about = "Show or pin the Wi-Fi band (omarchy network band)")]
    Band {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Print the active Wi-Fi password (omarchy network password)")]
    Password {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Generate a Wi-Fi QR code (omarchy network qr)")]
    Qr {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Measure internet speed (omarchy network speedtest)")]
    Speedtest {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Print active network status (omarchy network status)")]
    Status {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum HookCmd {
    #[command(about = "Run a named hook (omarchy hook run)")]
    Run {
        name: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    #[command(about = "Install a hook file (omarchy hook install)")]
    Install {
        hook_type: String,
        file: String,
    },
}

#[derive(Subcommand)]
pub enum SudoCmd {
    #[command(about = "Check if docker socket is writable (omarchy sudo docker)")]
    Docker {
        #[arg(long, help = "Check group membership instead of socket writability")]
        configured: bool,
    },
    #[command(about = "Keep sudo credentials alive in the background (omarchy sudo keepalive)")]
    Keepalive,
    #[command(about = "Toggle passwordless sudo (omarchy sudo passwordless)")]
    Passwordless {
        #[arg(help = "Duration in minutes (default 15)")]
        minutes: Option<u32>,
    },
}

// ── Batch 4 enums ─────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum DevCmd {
    #[command(name = "add-migration", about = "Create a new timestamped migration script (omarchy dev add migration)")]
    AddMigration {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "benchmark-cli", about = "Benchmark CLI commands (omarchy dev benchmark cli)")]
    BenchmarkCli {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "benchmark-theme-switcher", about = "Benchmark theme switching (omarchy dev benchmark theme switcher)")]
    BenchmarkThemeSwitcher {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Font development tool (omarchy dev font)")]
    Font {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "theme-preview", about = "Preview themes (omarchy dev theme preview)")]
    ThemePreview {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "ui-preview", about = "Preview UI components (omarchy dev ui preview)")]
    UiPreview {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum DriveCmd {
    #[command(about = "Print drive information (omarchy drive info)")]
    Info {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Manage LUKS drive password (omarchy drive password)")]
    Password {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Interactively select a drive (omarchy drive select)")]
    Select {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum MenuCmd {
    #[command(name = "main", about = "Show the main Omarchy menu (omarchy menu main)")]
    Main {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Clipboard history menu (omarchy menu clipboard)")]
    Clipboard {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Emoji picker (omarchy menu emoji)")]
    Emoji {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "emoji-insert", about = "Insert emoji at cursor (omarchy menu emoji insert)")]
    EmojiInsert {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "File picker menu (omarchy menu file)")]
    File {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "herdr-keybindings", about = "Herdr keybindings reference (omarchy menu herdr keybindings)")]
    HerdrKeybindings {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Image browser menu (omarchy menu images)")]
    Images {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Text input prompt (omarchy menu input)")]
    Input {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Keybindings reference (omarchy menu keybindings)")]
    Keybindings {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Plugin management menu (omarchy menu plugin)")]
    Plugin {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Generic selection menu (omarchy menu select)")]
    Select {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Share content menu (omarchy menu share)")]
    Share {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Timezone selection menu (omarchy menu timezone)")]
    Timezone {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "tmux-keybindings", about = "Tmux keybindings reference (omarchy menu tmux keybindings)")]
    TmuxKeybindings {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum PluginCmd {
    #[command(about = "Add a plugin (omarchy plugin add)")]
    Add {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Browse the plugin catalog (omarchy plugin catalog)")]
    Catalog {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Clone a plugin repository (omarchy plugin clone)")]
    Clone {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Disable a plugin (omarchy plugin disable)")]
    Disable {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Enable a plugin (omarchy plugin enable)")]
    Enable {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "List installed plugins (omarchy plugin list)")]
    List {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Remove a plugin (omarchy plugin remove)")]
    Remove {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Update plugins (omarchy plugin update)")]
    Update {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Validate a plugin (omarchy plugin validate)")]
    Validate {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum RefreshCmd {
    #[command(about = "Refresh application launchers (omarchy refresh applications)")]
    Applications {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh Chromium configuration (omarchy refresh chromium)")]
    Chromium {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh a configuration file (omarchy refresh config)")]
    Config {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh Herdr configuration (omarchy refresh herdr)")]
    Herdr {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh Hyprland configuration (omarchy refresh hyprland)")]
    Hyprland {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh Hyprsunset configuration (omarchy refresh hyprsunset)")]
    Hyprsunset {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh shell configuration (omarchy refresh shell)")]
    Shell {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh tmux configuration (omarchy refresh tmux)")]
    Tmux {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh Plymouth boot theme configuration (omarchy refresh plymouth)")]
    Plymouth {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh SDDM display manager theme (omarchy refresh sddm)")]
    Sddm {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ThemeCmd {
    #[command(name = "bg-cache", about = "Cache background images (omarchy theme bg cache)")]
    BgCache {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "bg-current", about = "Show current background (omarchy theme bg current)")]
    BgCurrent {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "bg-install", about = "Install background images (omarchy theme bg install)")]
    BgInstall {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "bg-next", about = "Switch to next background (omarchy theme bg next)")]
    BgNext {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "bg-set", about = "Set background image (omarchy theme bg set)")]
    BgSet {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "bg-switcher", about = "Run background switcher loop (omarchy theme bg switcher)")]
    BgSwitcher {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Show a theme color value (omarchy theme color)")]
    Color {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "colors-from-alacritty", about = "Generate theme colors from Alacritty config (omarchy theme colors from alacritty)")]
    ColorsFromAlacritty {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Show current theme name (omarchy theme current)")]
    Current {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Print theme directory path (omarchy theme dir)")]
    Dir {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Show theme extras (omarchy theme extras)")]
    Extras {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Install a theme (omarchy theme install)")]
    Install {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "List available themes (omarchy theme list)")]
    List {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Emit OSC color sequences for current theme (omarchy theme osc)")]
    Osc {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Refresh theme application (omarchy theme refresh)")]
    Refresh {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Remove a theme (omarchy theme remove)")]
    Remove {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Set the active theme (omarchy theme set)", long_about = "Set the active theme\n\nBinary: omarchy-theme-set\n\nUsage: omarchy theme set <theme-name>")]
    Set {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-browser", about = "Apply theme to browser (omarchy theme set browser)")]
    SetBrowser {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-browser-policy", about = "Apply browser theme policy (omarchy theme set browser policy)")]
    SetBrowserPolicy {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-claude", about = "Apply theme to Claude (omarchy theme set claude)")]
    SetClaude {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-foot", about = "Apply theme to Foot terminal (omarchy theme set foot)")]
    SetFoot {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-gnome", about = "Apply theme to GNOME apps (omarchy theme set gnome)")]
    SetGnome {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-hermes", about = "Apply theme to Hermes (omarchy theme set hermes)")]
    SetHermes {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-keyboard", about = "Apply theme to keyboard backlight (omarchy theme set keyboard)")]
    SetKeyboard {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-keyboard-asus-rog", about = "Apply theme to ASUS ROG keyboard (omarchy theme set keyboard asus rog)")]
    SetKeyboardAsusRog {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-keyboard-f16", about = "Apply theme to Framework 16 keyboard (omarchy theme set keyboard f16)")]
    SetKeyboardF16 {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-obsidian", about = "Apply theme to Obsidian (omarchy theme set obsidian)")]
    SetObsidian {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-pi", about = "Apply theme to Pi displays (omarchy theme set pi)")]
    SetPi {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-t3code", about = "Apply theme to T3 Code (omarchy theme set t3code)")]
    SetT3code {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-templates", about = "Apply theme templates (omarchy theme set templates)")]
    SetTemplates {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-tmux", about = "Apply theme to tmux (omarchy theme set tmux)")]
    SetTmux {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-vscode", about = "Apply theme to VS Code (omarchy theme set vscode)")]
    SetVscode {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Run the interactive theme switcher (omarchy theme switcher)")]
    Switcher {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Update themes from upstream (omarchy theme update)")]
    Update {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum TranscodeCmd {
    #[command(about = "Transcode a media file (omarchy transcode convert)")]
    Convert {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Convert media to ASCII art (omarchy transcode ascii)")]
    Ascii {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum VoxtypeCmd {
    #[command(about = "Configure Voxtype (omarchy voxtype config)")]
    Config {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Select Voxtype model (omarchy voxtype model)")]
    Model {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Show Voxtype status (omarchy voxtype status)")]
    Status {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum WebappHandlerCmd {
    #[command(about = "Handle Hey web app URLs (omarchy webapp handler hey)")]
    Hey {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Handle Zoom web app URLs (omarchy webapp handler zoom)")]
    Zoom {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ChannelCmd {
    #[command(about = "Print the active Omarchy package channel (omarchy channel current)")]
    Current {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Set the Omarchy package channel (omarchy channel set)")]
    Set {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum HibernationCmd {
    #[command(about = "Check if hibernation is supported (omarchy hibernation available)")]
    Available,
    #[command(about = "Remove hibernation support (omarchy hibernation remove)")]
    Remove {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Set up hibernation support (omarchy hibernation setup)")]
    Setup {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum PlymouthCmd {
    #[command(about = "Show which theme is styling the Plymouth boot screen (omarchy plymouth current)")]
    Current,
    #[command(about = "List available Plymouth themes (omarchy plymouth list)")]
    List,
    #[command(about = "Preview a Plymouth theme (omarchy plymouth preview)")]
    Preview {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Reset Plymouth to default theme (omarchy plymouth reset)")]
    Reset {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Set the Plymouth boot theme (omarchy plymouth set)")]
    Set {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "set-by-theme", about = "Set Plymouth theme based on Omarchy theme (omarchy plymouth set by theme)")]
    SetByTheme {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Run the interactive Plymouth theme switcher (omarchy plymouth switcher)")]
    Switcher {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ApplyCmd {
    #[command(about = "Apply hardware configuration", hide = true,
              long_about = "Apply hardware configuration\n\nBinary: omarchy-apply-hardware")]
    Hardware {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Apply system configuration", hide = true,
              long_about = "Apply system configuration\n\nBinary: omarchy-apply-system")]
    System {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Apply lock configuration", hide = true,
              long_about = "Apply lock configuration\n\nBinary: omarchy-apply-lock")]
    Lock {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}
