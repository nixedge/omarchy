mod cli;
mod cmds;
mod desktop;
mod filter;
mod output;
mod socket;
mod theme;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{
    AudioCmd, BluetoothCmd, BrightnessCmd, Cli, Cmd, BatteryCmd, CmdCheckCmd, ConfigCmd,
    FontCmd, HwCmd, InstallCmd, NotificationCmd, PkgCmd, PowerCmd, PowerprofilesCmd,
    RemoveCmd, RestartCmd, ServiceCmd, SetupCmd, SudoCmd, TailscaleCmd, ToggleCmd, UpdateCmd,
    WeatherCmd,
};
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
        // hw-* → hw check / hw state
        "omarchy-hw-asus-rog" => Some(&["hw", "check", "asus-rog"]),
        "omarchy-hw-asus-expertbook-b9406" => Some(&["hw", "check", "asus-expertbook"]),
        "omarchy-hw-asus-zenbook-ux5406aa" => Some(&["hw", "check", "asus-zenbook"]),
        "omarchy-hw-dell-xps13-sidecar-amps" => Some(&["hw", "check", "dell-xps13-sidecar-amps"]),
        "omarchy-hw-dell-xps-haptic-touchpad" => Some(&["hw", "check", "dell-xps-haptic"]),
        "omarchy-hw-dell-xps-oled" => Some(&["hw", "check", "dell-xps-oled"]),
        "omarchy-hw-elgato-camlink-4k" => Some(&["hw", "check", "elgato-camlink"]),
        "omarchy-hw-fingerprint" => Some(&["hw", "check", "fingerprint"]),
        "omarchy-hw-framework16" => Some(&["hw", "check", "framework16"]),
        "omarchy-hw-hybrid-gpu" => Some(&["hw", "check", "hybrid-gpu"]),
        "omarchy-hw-intel" => Some(&["hw", "check", "intel"]),
        "omarchy-hw-intel-ptl" => Some(&["hw", "check", "intel-ptl"]),
        "omarchy-hw-intel-sof" => Some(&["hw", "check", "intel-sof"]),
        "omarchy-hw-laptop" => Some(&["hw", "check", "laptop"]),
        "omarchy-hw-match" => Some(&["hw", "check", "match"]),
        "omarchy-hw-nvidia" => Some(&["hw", "check", "nvidia"]),
        "omarchy-hw-nvidia-gsp" => Some(&["hw", "check", "nvidia-gsp"]),
        "omarchy-hw-nvidia-without-gsp" => Some(&["hw", "check", "nvidia-without-gsp"]),
        "omarchy-hw-surface" => Some(&["hw", "check", "surface"]),
        "omarchy-hw-vulkan" => Some(&["hw", "check", "vulkan"]),
        "omarchy-hw-clamshell" => Some(&["hw", "state", "clamshell"]),
        "omarchy-hw-display" => Some(&["hw", "state", "display"]),
        "omarchy-hw-external-monitors" => Some(&["hw", "state", "external-monitors"]),
        "omarchy-hw-laptop-closed" => Some(&["hw", "state", "lid"]),
        "omarchy-hw-touchpad" => Some(&["hw", "state", "touchpad"]),
        "omarchy-hw-touchscreen" => Some(&["hw", "state", "touchscreen"]),
        "omarchy-hw-webcam" => Some(&["hw", "state", "webcam"]),
        "omarchy-hw-recover-internal-monitor" => Some(&["hw", "recover-internal-monitor"]),
        // restart-* → restart <subcmd>
        "omarchy-restart-app" => Some(&["restart", "app"]),
        "omarchy-restart-audio" => Some(&["restart", "audio"]),
        "omarchy-restart-bluetooth" => Some(&["restart", "bluetooth"]),
        "omarchy-restart-btop" => Some(&["restart", "btop"]),
        "omarchy-restart-gum" => Some(&["restart", "gum"]),
        "omarchy-restart-helix" => Some(&["restart", "helix"]),
        "omarchy-restart-herdr" => Some(&["restart", "herdr"]),
        "omarchy-restart-hyprctl" => Some(&["restart", "hyprctl"]),
        "omarchy-restart-hyprsunset" => Some(&["restart", "hyprsunset"]),
        "omarchy-restart-opencode" => Some(&["restart", "opencode"]),
        "omarchy-restart-shell" => Some(&["restart", "shell"]),
        "omarchy-restart-terminal" => Some(&["restart", "terminal"]),
        "omarchy-restart-tmux" => Some(&["restart", "tmux"]),
        "omarchy-restart-trackpad" => Some(&["restart", "trackpad"]),
        "omarchy-restart-wifi" => Some(&["restart", "wifi"]),
        "omarchy-restart-xcompose" => Some(&["restart", "xcompose"]),
        // notification-* → notification <subcmd>
        "omarchy-notification-battery" => Some(&["notification", "battery"]),
        "omarchy-notification-time" => Some(&["notification", "time"]),
        "omarchy-notification-weather" => Some(&["notification", "weather"]),
        "omarchy-notification-dismiss" => Some(&["notification", "dismiss"]),
        "omarchy-notification-wait" => Some(&["notification", "wait"]),
        "omarchy-notification-send" => Some(&["notification", "send"]),
        // battery-* → battery <subcmd>
        "omarchy-battery-present" => Some(&["battery", "present"]),
        "omarchy-battery-low" => Some(&["battery", "low"]),
        "omarchy-battery-status" => Some(&["battery", "status"]),
        // power-* → power <subcmd>
        "omarchy-power-present" => Some(&["power", "present"]),
        // cmd-* → cmd <subcmd>
        "omarchy-cmd-missing" => Some(&["cmd", "missing"]),
        "omarchy-cmd-present" => Some(&["cmd", "present"]),
        "omarchy-cmd-terminal-cwd" => Some(&["cmd", "terminal-cwd"]),
        // state
        "omarchy-state" => Some(&["state"]),
        // done
        "omarchy-done" => Some(&["done"]),
        "omarchy-show-done" => Some(&["show-done"]),
        "omarchy-show-logo" => Some(&["show-logo"]),
        // version
        "omarchy-version" => Some(&["version"]),
        "omarchy-version-branch" => Some(&["version-branch"]),
        // powerprofiles
        "omarchy-powerprofiles-init" => Some(&["powerprofiles", "init"]),
        "omarchy-powerprofiles-list" => Some(&["powerprofiles", "list"]),
        "omarchy-powerprofiles-set" => Some(&["powerprofiles", "set"]),
        // osd
        "omarchy-osd" => Some(&["osd"]),
        // windows
        "omarchy-windows-key" => Some(&["windows-key"]),
        "omarchy-windows-vm" => Some(&["windows-vm"]),
        // tailscale
        "omarchy-tailscale-receive" => Some(&["tailscale", "receive"]),
        "omarchy-tailscale-send" => Some(&["tailscale", "send"]),
        // update
        "omarchy-update-dev" => Some(&["update", "dev"]),
        "omarchy-update-firmware" => Some(&["update", "firmware"]),
        "omarchy-update-lock" => Some(&["update", "lock"]),
        "omarchy-update-requires-free-space" => Some(&["update", "requires-free-space"]),
        "omarchy-update-status" => Some(&["update", "status"]),
        "omarchy-update-stay-awake" => Some(&["update", "stay-awake"]),
        "omarchy-update-time" => Some(&["update", "time"]),
        "omarchy-update-user-notify" => Some(&["update", "user-notify"]),
        // setup
        "omarchy-setup-direct-boot" => Some(&["setup", "direct-boot"]),
        // sudo
        "omarchy-sudo-docker" => Some(&["sudo", "docker"]),
        "omarchy-sudo-keepalive" => Some(&["sudo", "keepalive"]),
        "omarchy-sudo-passwordless" => Some(&["sudo", "passwordless"]),
        // screensaver
        "omarchy-screensaver" => Some(&["screensaver"]),
        // misc
        "omarchy-git-url-check" => Some(&["git-url-check"]),
        "omarchy-games-retro-cores" => Some(&["games-retro-cores"]),
        "omarchy-games-retro-install" => Some(&["games-retro-install"]),
        "omarchy-monitor-state" => Some(&["monitor-state"]),
        "omarchy-disk-speedtest" => Some(&["disk-speedtest"]),
        "omarchy-reminder" => Some(&["reminder"]),
        "omarchy-display-text-size" => Some(&["display-text-size"]),
        // audio
        "omarchy-audio-output-volume" => Some(&["audio", "output-volume"]),
        "omarchy-audio-output-switch" => Some(&["audio", "output-switch"]),
        "omarchy-audio-input-mute" => Some(&["audio", "input-mute"]),
        "omarchy-audio-output-sink" => Some(&["audio", "output-sink"]),
        "omarchy-audio-sink-availability" => Some(&["audio", "sink-availability"]),
        "omarchy-audio-source-switch" => Some(&["audio", "source-switch"]),
        "omarchy-audio-input-set-default" => Some(&["audio", "input-set-default"]),
        "omarchy-audio-output-set-default" => Some(&["audio", "output-set-default"]),
        "omarchy-audio-tuning" => Some(&["audio", "tuning"]),
        // brightness
        "omarchy-brightness-display" => Some(&["brightness", "display"]),
        "omarchy-brightness-display-apple" => Some(&["brightness", "display-apple"]),
        "omarchy-brightness-display-ddc" => Some(&["brightness", "display-ddc"]),
        "omarchy-brightness-keyboard" => Some(&["brightness", "keyboard"]),
        "omarchy-brightness-keyboard-mute" => Some(&["brightness", "keyboard-mute"]),
        // toggle
        "omarchy-toggle" => Some(&["toggle", "flag"]),
        "omarchy-toggle-bar" => Some(&["toggle", "bar"]),
        "omarchy-toggle-crash-capture" => Some(&["toggle", "crash-capture"]),
        "omarchy-toggle-enabled" => Some(&["toggle", "enabled"]),
        "omarchy-toggle-fullscreen-desktop" => Some(&["toggle", "fullscreen-desktop"]),
        "omarchy-toggle-hybrid-gpu" => Some(&["toggle", "hybrid-gpu"]),
        "omarchy-toggle-idle" => Some(&["toggle", "idle"]),
        "omarchy-toggle-input-device" => Some(&["toggle", "input-device"]),
        "omarchy-toggle-nightlight" => Some(&["toggle", "nightlight"]),
        "omarchy-toggle-notification-silencing" => Some(&["toggle", "notification-silencing"]),
        "omarchy-toggle-screensaver" => Some(&["toggle", "screensaver"]),
        "omarchy-toggle-suspend" => Some(&["toggle", "suspend"]),
        "omarchy-toggle-touchpad" => Some(&["toggle", "touchpad"]),
        "omarchy-toggle-touchscreen" => Some(&["toggle", "touchscreen"]),
        // bluetooth
        "omarchy-bluetooth-device" => Some(&["bluetooth", "device"]),
        "omarchy-bluetooth-power" => Some(&["bluetooth", "power"]),
        // bar
        "omarchy-bar" => Some(&["bar"]),
        "omarchy-bar-text-color" => Some(&["bar-text-color"]),
        // ascii
        "omarchy-ascii" => Some(&["ascii"]),
        // font
        "omarchy-font-current" => Some(&["font", "current"]),
        "omarchy-font-list" => Some(&["font", "list"]),
        "omarchy-font-set" => Some(&["font", "set"]),
        // weather
        "omarchy-weather-icon" => Some(&["weather", "icon"]),
        "omarchy-weather-location" => Some(&["weather", "location"]),
        "omarchy-weather-status" => Some(&["weather", "status"]),
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
        Cmd::Hw { subcmd } => match subcmd {
            HwCmd::Detect => cmds::hw::detect::run(),
            HwCmd::Check { name, pattern } => cmds::hw::check::run(&name, pattern.as_deref()),
            HwCmd::State { name } => cmds::hw::state::run(&name),
            HwCmd::RecoverInternalMonitor => {
                let toggle = format!(
                    "{}/.local/state/omarchy/toggles/hypr/internal-monitor-disable.lua",
                    std::env::var("HOME").unwrap_or_default()
                );
                if std::path::Path::new(&toggle).exists()
                    && cmds::hw::state::run("external-monitors") != 0
                {
                    let _ = std::fs::remove_file(&toggle);
                }
                0
            }
        },
        Cmd::Restart { subcmd } => match subcmd {
            RestartCmd::App { name, args } => cmds::restart::app(&name, &args),
            RestartCmd::Audio => cmds::restart::audio::run(),
            RestartCmd::Bluetooth => cmds::restart::bluetooth(),
            RestartCmd::Btop => cmds::restart::btop(),
            RestartCmd::Gum => cmds::restart::gum(),
            RestartCmd::Helix => cmds::restart::helix(),
            RestartCmd::Herdr => cmds::restart::herdr(),
            RestartCmd::Hyprctl => cmds::restart::hyprctl(),
            RestartCmd::Hyprsunset => cmds::restart::hyprsunset(),
            RestartCmd::Opencode => cmds::restart::opencode(),
            RestartCmd::Shell => cmds::restart::shell(),
            RestartCmd::Terminal => cmds::restart::terminal(),
            RestartCmd::Tmux => cmds::restart::tmux(),
            RestartCmd::Trackpad => cmds::restart::trackpad(),
            RestartCmd::Wifi => cmds::restart::wifi(),
            RestartCmd::Xcompose => cmds::restart::xcompose(),
        },
        Cmd::Notification { subcmd } => match subcmd {
            NotificationCmd::Battery => cmds::notification::battery(),
            NotificationCmd::Time => cmds::notification::time(),
            NotificationCmd::Weather => cmds::notification::weather(),
            NotificationCmd::Dismiss { summary } => cmds::notification::dismiss(&summary),
            NotificationCmd::Wait { seconds } => cmds::notification::wait(seconds),
            NotificationCmd::Send { args } => cmds::notification::send::run(&args),
        },
        Cmd::Battery { subcmd } => match subcmd {
            BatteryCmd::Present => cmds::battery::present(),
            BatteryCmd::Low { percentage } => cmds::battery::low(percentage),
            BatteryCmd::Status { shell } => cmds::battery::status(shell),
        },
        Cmd::Power { subcmd } => match subcmd {
            PowerCmd::Present => cmds::power::present(),
        },
        Cmd::CmdCheck { subcmd } => match subcmd {
            CmdCheckCmd::Missing { cmds: c } => cmds::cmd_check::missing(&c),
            CmdCheckCmd::Present { cmds: c } => cmds::cmd_check::present(&c),
            CmdCheckCmd::TerminalCwd => cmds::cmd_check::terminal_cwd(),
        },
        Cmd::State { action, name } => cmds::state_cmd::run(&action, &name),
        Cmd::Done { action, name } => cmds::done_cmd::done(&action, &name),
        Cmd::ShowDone { exit_code } => cmds::done_cmd::show_done(exit_code),
        Cmd::ShowLogo => cmds::done_cmd::show_logo(),
        Cmd::Version => cmds::version_cmd::version(),
        Cmd::VersionBranch => cmds::version_cmd::branch(),
        Cmd::Powerprofiles { subcmd } => match subcmd {
            PowerprofilesCmd::Init => cmds::powerprofiles::init(),
            PowerprofilesCmd::List { active_state } => cmds::powerprofiles::list(active_state),
            PowerprofilesCmd::Set { action, profile } => {
                cmds::powerprofiles::set(action.as_deref(), profile.as_deref())
            }
        },
        Cmd::Osd { icon, message, progress, duration } => {
            cmds::osd::run(icon.as_deref(), message.as_deref(), progress.as_deref(), duration.as_deref())
        }
        Cmd::WindowsKey => cmds::windows::windows_key(),
        Cmd::WindowsVm { args } => cmds::windows::windows_vm(&args),
        Cmd::Tailscale { subcmd } => match subcmd {
            TailscaleCmd::Receive { once, dir } => cmds::tailscale::receive(once, dir.as_deref()),
            TailscaleCmd::Send { machine, files } => cmds::tailscale::send(&machine, &files),
        },
        Cmd::Update { subcmd } => match subcmd {
            UpdateCmd::Dev => cmds::update::dev(),
            UpdateCmd::Firmware => cmds::update::firmware(),
            UpdateCmd::Lock { action, cmd_args } => cmds::update::lock(&action, &cmd_args),
            UpdateCmd::RequiresFreeSpace => cmds::update::requires_free_space(),
            UpdateCmd::Status => cmds::update::status(),
            UpdateCmd::StayAwake { action } => cmds::update::stay_awake(&action),
            UpdateCmd::Time => cmds::update::time(),
            UpdateCmd::UserNotify { args } => cmds::update::user_notify(&args),
        },
        Cmd::Setup { subcmd } => match subcmd {
            SetupCmd::DirectBoot => cmds::setup_cmd::direct_boot(),
        },
        Cmd::Sudo { subcmd } => match subcmd {
            SudoCmd::Docker { configured } => cmds::sudo_cmd::docker(configured),
            SudoCmd::Keepalive => cmds::sudo_cmd::keepalive(),
            SudoCmd::Passwordless { minutes } => cmds::sudo_cmd::passwordless(minutes),
        },
        Cmd::Screensaver { force } => cmds::screensaver::run(force),
        Cmd::GitUrlCheck { url } => cmds::git_url_check::run(&url),
        Cmd::GamesRetroCores => cmds::games::retro_cores(),
        Cmd::GamesRetroInstall { core, game } => {
            cmds::games::retro_install(core.as_deref(), game.as_deref())
        }
        Cmd::MonitorState => cmds::monitor_state::run(),
        Cmd::DiskSpeedtest { dir } => cmds::disk_speedtest::run(dir.as_deref()),
        Cmd::Reminder { args } => cmds::reminder::run(&args),
        Cmd::DisplayTextSize { size } => cmds::display_text_size::run(size.as_deref()),
        Cmd::Audio { subcmd } => match subcmd {
            AudioCmd::OutputVolume { action } => cmds::audio::output_volume(&action),
            AudioCmd::OutputSwitch => cmds::audio::output_switch(),
            AudioCmd::InputMute => cmds::audio::input_mute(),
            AudioCmd::OutputSink { sink } => cmds::audio::output_sink(sink.as_deref()),
            AudioCmd::SinkAvailability => cmds::audio::sink_availability(),
            AudioCmd::SourceSwitch { direction } => cmds::audio::source_switch(direction.as_deref()),
            AudioCmd::InputSetDefault { node_id, source_name } => {
                cmds::audio::input_set_default(&node_id, &source_name)
            }
            AudioCmd::OutputSetDefault { node_id, sink_name } => {
                cmds::audio::output_set_default(&node_id, &sink_name)
            }
            AudioCmd::Tuning { action, force } => cmds::audio::tuning(&action, force),
        },
        Cmd::Brightness { subcmd } => match subcmd {
            BrightnessCmd::Display { no_osd, monitor, step } => {
                cmds::brightness::display(no_osd, monitor.as_deref(), step.as_deref())
            }
            BrightnessCmd::DisplayApple { no_osd, step } => {
                cmds::brightness::display_apple(no_osd, step.as_deref())
            }
            BrightnessCmd::DisplayDdc { monitor, step } => {
                cmds::brightness::display_ddc(&monitor, step.as_deref())
            }
            BrightnessCmd::Keyboard { no_osd, direction } => {
                cmds::brightness::keyboard(no_osd, &direction)
            }
            BrightnessCmd::KeyboardMute { state } => cmds::brightness::keyboard_mute(&state),
        },
        Cmd::Toggle { subcmd } => match subcmd {
            ToggleCmd::Flag { flag_name, action } => cmds::toggle::flag(&flag_name, &action),
            ToggleCmd::Bar { action } => cmds::toggle::bar(&action),
            ToggleCmd::CrashCapture => cmds::toggle::crash_capture(),
            ToggleCmd::Enabled { flag_name } => cmds::toggle::enabled(&flag_name),
            ToggleCmd::FullscreenDesktop { action } => cmds::toggle::fullscreen_desktop(&action),
            ToggleCmd::HybridGpu => cmds::toggle::hybrid_gpu(),
            ToggleCmd::Idle { action } => cmds::toggle::idle(&action),
            ToggleCmd::InputDevice { kind, action } => cmds::toggle::input_device(&kind, &action),
            ToggleCmd::Nightlight { status } => cmds::toggle::nightlight(status),
            ToggleCmd::NotificationSilencing => cmds::toggle::notification_silencing(),
            ToggleCmd::Screensaver => cmds::toggle::screensaver(),
            ToggleCmd::Suspend => cmds::toggle::suspend(),
            ToggleCmd::Touchpad { action } => cmds::toggle::touchpad(&action),
            ToggleCmd::Touchscreen { action } => cmds::toggle::touchscreen(&action),
        },
        Cmd::Bluetooth { subcmd } => match subcmd {
            BluetoothCmd::Device { action, address } => cmds::bluetooth::device(&action, &address),
            BluetoothCmd::Power { action } => cmds::bluetooth::power(&action),
        },
        Cmd::Bar { args } => cmds::bar::run(&args),
        Cmd::BarTextColor { position, bar_size, text_color, background_color, background, screen } => {
            cmds::bar::text_color(&position, &bar_size, &text_color, &background_color, background.as_deref(), screen.as_deref())
        }
        Cmd::Ascii { args } => cmds::ascii::run(&args),
        Cmd::Font { subcmd } => match subcmd {
            FontCmd::Current => cmds::font::current(),
            FontCmd::List => cmds::font::list(),
            FontCmd::Set { name } => cmds::font::set_font(&name),
        },
        Cmd::Weather { subcmd } => match subcmd {
            WeatherCmd::Icon => cmds::weather::icon(),
            WeatherCmd::Location { set, coords, clear } => {
                if clear {
                    cmds::weather::location(Some("--clear"), None, None)
                } else if let Some(name) = set {
                    cmds::weather::location(Some("--set"), Some(&name), coords.as_deref())
                } else {
                    cmds::weather::location(None, None, None)
                }
            }
            WeatherCmd::Status => cmds::weather::status(),
        },
        Cmd::Completions { shell } => {
            generate(shell, &mut Cli::command(), "omarchy-cli", &mut std::io::stdout());
            0
        }
    };
    process::exit(code);
}
