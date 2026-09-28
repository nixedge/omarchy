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
    AgentCmd, ApplyCmd, AudioCmd, BluetoothCmd, BrandingCmd, BrightnessCmd, CaptureCmd,
    ChannelCmd, Cli, Cmd, BatteryCmd, ClipboardCmd, CmdCheckCmd, ConfigCmd, CrashCmd, DebugCmd,
    DefaultCmd, DevCmd, DriveCmd, FontCmd, HibernationCmd, HookCmd, HwCmd, HyprlandCmd,
    InstallCmd, LaunchCmd, MenuCmd, MigrateCmd, NetworkCmd, NotificationCmd, PkgCmd, PluginCmd,
    PlymouthCmd, PowerCmd, PowerprofilesCmd, RefreshCmd, RemoveCmd, RestartCmd, ServiceCmd,
    SetupCmd, SudoCmd, SystemCmd, TailscaleCmd, ThemeCmd, ToggleCmd, TranscodeCmd, UpdateCmd,
    VersionCmd, VoxtypeCmd, WeatherCmd, WebappHandlerCmd,
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
        "omarchy-version-branch" => Some(&["version", "branch"]),
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
        // agent
        "omarchy-agent" => Some(&["agent", "run"]),
        "omarchy-agent-crash" => Some(&["agent", "crash"]),
        "omarchy-agent-prompt" => Some(&["agent", "prompt"]),
        "omarchy-agent-usage-claude" => Some(&["agent", "usage-claude"]),
        "omarchy-agent-usage-codex" => Some(&["agent", "usage-codex"]),
        "omarchy-agent-usage-fireworks" => Some(&["agent", "usage-fireworks"]),
        "omarchy-agent-usage-update" => Some(&["agent", "usage-update"]),
        // branding
        "omarchy-branding-about" => Some(&["branding", "about"]),
        "omarchy-branding-screensaver" => Some(&["branding", "screensaver"]),
        // capture
        "omarchy-capture-qr" => Some(&["capture", "qr"]),
        "omarchy-capture-region" => Some(&["capture", "region"]),
        "omarchy-capture-screenrecording" => Some(&["capture", "screenrecording"]),
        "omarchy-capture-screenrecording-with-webcam" => Some(&["capture", "screenrecording-with-webcam"]),
        "omarchy-capture-screenshot" => Some(&["capture", "screenshot"]),
        "omarchy-capture-text" => Some(&["capture", "text"]),
        "omarchy-capture-webcam-list" => Some(&["capture", "webcam-list"]),
        "omarchy-capture-webcam-resize" => Some(&["capture", "webcam-resize"]),
        // chromium hosts
        "omarchy-chromium-copy-url-host" => Some(&["chromium-copy-url-host"]),
        "omarchy-chromium-ytdlp-host" => Some(&["chromium-ytdlp-host"]),
        // clipboard
        "omarchy-clipboard-open" => Some(&["clipboard", "open"]),
        "omarchy-clipboard-paste-file" => Some(&["clipboard", "paste-file"]),
        "omarchy-clipboard-paste-text" => Some(&["clipboard", "paste-text"]),
        // crash
        "omarchy-crash-mute" => Some(&["crash", "mute"]),
        "omarchy-crash-watch" => Some(&["crash", "watch"]),
        // debug
        "omarchy-debug" => Some(&["debug", "info"]),
        "omarchy-debug-idle" => Some(&["debug", "idle"]),
        // default
        "omarchy-default-agent" => Some(&["default", "agent"]),
        "omarchy-default-browser" => Some(&["default", "browser"]),
        "omarchy-default-editor" => Some(&["default", "editor"]),
        "omarchy-default-terminal" => Some(&["default", "terminal"]),
        // hook
        "omarchy-hook" => Some(&["hook", "run"]),
        "omarchy-hook-install" => Some(&["hook", "install"]),
        // hyprland
        "omarchy-hyprland-focus-app" => Some(&["hyprland", "focus-app"]),
        "omarchy-hyprland-monitor-clamshell" => Some(&["hyprland", "monitor-clamshell"]),
        "omarchy-hyprland-monitor-external-active" => Some(&["hyprland", "monitor-external-active"]),
        "omarchy-hyprland-monitor-focused" => Some(&["hyprland", "monitor-focused"]),
        "omarchy-hyprland-monitor-focused-apple" => Some(&["hyprland", "monitor-focused-apple"]),
        "omarchy-hyprland-monitor-internal" => Some(&["hyprland", "monitor-internal"]),
        "omarchy-hyprland-monitor-internal-mirror" => Some(&["hyprland", "monitor-internal-mirror"]),
        "omarchy-hyprland-monitor-laptop" => Some(&["hyprland", "monitor-laptop"]),
        "omarchy-hyprland-monitor-modeless" => Some(&["hyprland", "monitor-modeless"]),
        "omarchy-hyprland-monitor-scaling" => Some(&["hyprland", "monitor-scaling"]),
        "omarchy-hyprland-monitor-watch" => Some(&["hyprland", "monitor-watch"]),
        "omarchy-hyprland-reload-guard" => Some(&["hyprland", "reload-guard"]),
        "omarchy-hyprland-session-locked" => Some(&["hyprland", "session-locked"]),
        "omarchy-hyprland-toggle" => Some(&["hyprland", "toggle"]),
        "omarchy-hyprland-toggle-disabled" => Some(&["hyprland", "toggle-disabled"]),
        "omarchy-hyprland-toggle-enabled" => Some(&["hyprland", "toggle-enabled"]),
        "omarchy-hyprland-window-close-all" => Some(&["hyprland", "window-close-all"]),
        "omarchy-hyprland-window-gaps-toggle" => Some(&["hyprland", "window-gaps-toggle"]),
        "omarchy-hyprland-window-pop" => Some(&["hyprland", "window-pop"]),
        "omarchy-hyprland-window-single-square-aspect-toggle" => Some(&["hyprland", "window-single-square-aspect-toggle"]),
        "omarchy-hyprland-window-tiled-fullscreen-toggle" => Some(&["hyprland", "window-tiled-fullscreen-toggle"]),
        "omarchy-hyprland-window-transparency-toggle" => Some(&["hyprland", "window-transparency-toggle"]),
        "omarchy-hyprland-window-width" => Some(&["hyprland", "window-width"]),
        "omarchy-hyprland-workspace-layout-toggle" => Some(&["hyprland", "workspace-layout-toggle"]),
        // system
        "omarchy-system-factory-reset" => Some(&["system", "factory-reset"]),
        "omarchy-system-factory-reset-finish" => Some(&["system", "factory-reset-finish"]),
        "omarchy-system-lid-close" => Some(&["system", "lid-close"]),
        "omarchy-system-lock" => Some(&["system", "lock"]),
        "omarchy-system-logout" => Some(&["system", "logout"]),
        "omarchy-system-reboot" => Some(&["system", "reboot"]),
        "omarchy-system-shutdown" => Some(&["system", "shutdown"]),
        "omarchy-system-sleep-lock" => Some(&["system", "sleep-lock"]),
        "omarchy-system-sleep-monitor" => Some(&["system", "sleep-monitor"]),
        "omarchy-system-stats" => Some(&["system", "stats"]),
        "omarchy-system-wake" => Some(&["system", "wake"]),
        // launch
        "omarchy-launch-1password" => Some(&["launch", "1password"]),
        "omarchy-launch-about" => Some(&["launch", "about"]),
        "omarchy-launch-battlenet" => Some(&["launch", "battlenet"]),
        "omarchy-launch-browser" => Some(&["launch", "browser"]),
        "omarchy-launch-config-editor" => Some(&["launch", "config-editor"]),
        "omarchy-launch-discord-community" => Some(&["launch", "discord-community"]),
        "omarchy-launch-docker-tui" => Some(&["launch", "docker-tui"]),
        "omarchy-launch-editor" => Some(&["launch", "editor"]),
        "omarchy-launch-floating-terminal-with-presentation" => Some(&["launch", "floating-terminal-with-presentation"]),
        "omarchy-launch-nautilus" => Some(&["launch", "nautilus"]),
        "omarchy-launch-nautilus-cwd" => Some(&["launch", "nautilus-cwd"]),
        "omarchy-launch-openclaw" => Some(&["launch", "openclaw"]),
        "omarchy-launch-or-focus" => Some(&["launch", "or-focus"]),
        "omarchy-launch-or-focus-tui" => Some(&["launch", "or-focus-tui"]),
        "omarchy-launch-or-focus-webapp" => Some(&["launch", "or-focus-webapp"]),
        "omarchy-launch-screensaver" => Some(&["launch", "screensaver"]),
        "omarchy-launch-shell" => Some(&["launch", "shell"]),
        "omarchy-launch-signal" => Some(&["launch", "signal"]),
        "omarchy-launch-spotify" => Some(&["launch", "spotify"]),
        "omarchy-launch-terminal" => Some(&["launch", "terminal"]),
        "omarchy-launch-terminal-herdr" => Some(&["launch", "terminal-herdr"]),
        "omarchy-launch-terminal-tmux" => Some(&["launch", "terminal-tmux"]),
        "omarchy-launch-tui" => Some(&["launch", "tui"]),
        "omarchy-launch-webapp" => Some(&["launch", "webapp"]),
        // shell IPC
        "omarchy-shell" => Some(&["shell"]),
        // migrate
        "omarchy-migrate" => Some(&["migrate", "run"]),
        "omarchy-migrate-notify" => Some(&["migrate", "notify"]),
        // misc
        "omarchy-dns" => Some(&["dns"]),
        "omarchy-mise-install" => Some(&["mise-install"]),
        "omarchy-network-band" => Some(&["network", "band"]),
        "omarchy-network-password" => Some(&["network", "password"]),
        "omarchy-network-qr" => Some(&["network", "qr"]),
        "omarchy-network-speedtest" => Some(&["network", "speedtest"]),
        "omarchy-network-status" => Some(&["network", "status"]),
        // dev
        "omarchy-dev-add-migration" => Some(&["dev", "add-migration"]),
        "omarchy-dev-benchmark-cli" => Some(&["dev", "benchmark-cli"]),
        "omarchy-dev-benchmark-theme-switcher" => Some(&["dev", "benchmark-theme-switcher"]),
        "omarchy-dev-font" => Some(&["dev", "font"]),
        "omarchy-dev-theme-preview" => Some(&["dev", "theme-preview"]),
        "omarchy-dev-ui-preview" => Some(&["dev", "ui-preview"]),
        // drive
        "omarchy-drive-info" => Some(&["drive", "info"]),
        "omarchy-drive-password" => Some(&["drive", "password"]),
        "omarchy-drive-select" => Some(&["drive", "select"]),
        // file-select
        "omarchy-file-select" => Some(&["file-select"]),
        // menu
        "omarchy-menu" => Some(&["menu", "main"]),
        "omarchy-menu-clipboard" => Some(&["menu", "clipboard"]),
        "omarchy-menu-emoji" => Some(&["menu", "emoji"]),
        "omarchy-menu-emoji-insert" => Some(&["menu", "emoji-insert"]),
        "omarchy-menu-file" => Some(&["menu", "file"]),
        "omarchy-menu-herdr-keybindings" => Some(&["menu", "herdr-keybindings"]),
        "omarchy-menu-images" => Some(&["menu", "images"]),
        "omarchy-menu-input" => Some(&["menu", "input"]),
        "omarchy-menu-keybindings" => Some(&["menu", "keybindings"]),
        "omarchy-menu-plugin" => Some(&["menu", "plugin"]),
        "omarchy-menu-select" => Some(&["menu", "select"]),
        "omarchy-menu-share" => Some(&["menu", "share"]),
        "omarchy-menu-timezone" => Some(&["menu", "timezone"]),
        "omarchy-menu-tmux-keybindings" => Some(&["menu", "tmux-keybindings"]),
        // openclaw-onboard
        "omarchy-openclaw-onboard" => Some(&["openclaw-onboard"]),
        // plugin
        "omarchy-plugin-add" => Some(&["plugin", "add"]),
        "omarchy-plugin-catalog" => Some(&["plugin", "catalog"]),
        "omarchy-plugin-clone" => Some(&["plugin", "clone"]),
        "omarchy-plugin-disable" => Some(&["plugin", "disable"]),
        "omarchy-plugin-enable" => Some(&["plugin", "enable"]),
        "omarchy-plugin-list" => Some(&["plugin", "list"]),
        "omarchy-plugin-remove" => Some(&["plugin", "remove"]),
        "omarchy-plugin-update" => Some(&["plugin", "update"]),
        "omarchy-plugin-validate" => Some(&["plugin", "validate"]),
        // channel
        "omarchy-channel-current" => Some(&["channel", "current"]),
        "omarchy-channel-set" => Some(&["channel", "set"]),
        // hibernation
        "omarchy-hibernation-available" => Some(&["hibernation", "available"]),
        "omarchy-hibernation-remove" => Some(&["hibernation", "remove"]),
        "omarchy-hibernation-setup" => Some(&["hibernation", "setup"]),
        // plymouth
        "omarchy-plymouth-current" => Some(&["plymouth", "current"]),
        "omarchy-plymouth-list" => Some(&["plymouth", "list"]),
        "omarchy-plymouth-preview" => Some(&["plymouth", "preview"]),
        "omarchy-plymouth-reset" => Some(&["plymouth", "reset"]),
        "omarchy-plymouth-set" => Some(&["plymouth", "set"]),
        "omarchy-plymouth-set-by-theme" => Some(&["plymouth", "set-by-theme"]),
        "omarchy-plymouth-switcher" => Some(&["plymouth", "switcher"]),
        // refresh
        "omarchy-refresh-applications" => Some(&["refresh", "applications"]),
        "omarchy-refresh-chromium" => Some(&["refresh", "chromium"]),
        "omarchy-refresh-config" => Some(&["refresh", "config"]),
        "omarchy-refresh-herdr" => Some(&["refresh", "herdr"]),
        "omarchy-refresh-hyprland" => Some(&["refresh", "hyprland"]),
        "omarchy-refresh-hyprsunset" => Some(&["refresh", "hyprsunset"]),
        "omarchy-refresh-shell" => Some(&["refresh", "shell"]),
        "omarchy-refresh-tmux" => Some(&["refresh", "tmux"]),
        "omarchy-refresh-plymouth" => Some(&["refresh", "plymouth"]),
        "omarchy-refresh-sddm" => Some(&["refresh", "sddm"]),
        // theme
        "omarchy-theme-bg-cache" => Some(&["theme", "bg-cache"]),
        "omarchy-theme-bg-current" => Some(&["theme", "bg-current"]),
        "omarchy-theme-bg-install" => Some(&["theme", "bg-install"]),
        "omarchy-theme-bg-next" => Some(&["theme", "bg-next"]),
        "omarchy-theme-bg-set" => Some(&["theme", "bg-set"]),
        "omarchy-theme-bg-switcher" => Some(&["theme", "bg-switcher"]),
        "omarchy-theme-color" => Some(&["theme", "color"]),
        "omarchy-theme-colors-from-alacritty" => Some(&["theme", "colors-from-alacritty"]),
        "omarchy-theme-current" => Some(&["theme", "current"]),
        "omarchy-theme-dir" => Some(&["theme", "dir"]),
        "omarchy-theme-extras" => Some(&["theme", "extras"]),
        "omarchy-theme-install" => Some(&["theme", "install"]),
        "omarchy-theme-list" => Some(&["theme", "list"]),
        "omarchy-theme-osc" => Some(&["theme", "osc"]),
        "omarchy-theme-refresh" => Some(&["theme", "refresh"]),
        "omarchy-theme-remove" => Some(&["theme", "remove"]),
        "omarchy-theme-set" => Some(&["theme", "set"]),
        "omarchy-theme-set-browser" => Some(&["theme", "set-browser"]),
        "omarchy-theme-set-browser-policy" => Some(&["theme", "set-browser-policy"]),
        "omarchy-theme-set-claude" => Some(&["theme", "set-claude"]),
        "omarchy-theme-set-foot" => Some(&["theme", "set-foot"]),
        "omarchy-theme-set-gnome" => Some(&["theme", "set-gnome"]),
        "omarchy-theme-set-hermes" => Some(&["theme", "set-hermes"]),
        "omarchy-theme-set-keyboard" => Some(&["theme", "set-keyboard"]),
        "omarchy-theme-set-keyboard-asus-rog" => Some(&["theme", "set-keyboard-asus-rog"]),
        "omarchy-theme-set-keyboard-f16" => Some(&["theme", "set-keyboard-f16"]),
        "omarchy-theme-set-obsidian" => Some(&["theme", "set-obsidian"]),
        "omarchy-theme-set-pi" => Some(&["theme", "set-pi"]),
        "omarchy-theme-set-t3code" => Some(&["theme", "set-t3code"]),
        "omarchy-theme-set-templates" => Some(&["theme", "set-templates"]),
        "omarchy-theme-set-tmux" => Some(&["theme", "set-tmux"]),
        "omarchy-theme-set-vscode" => Some(&["theme", "set-vscode"]),
        "omarchy-theme-switcher" => Some(&["theme", "switcher"]),
        "omarchy-theme-update" => Some(&["theme", "update"]),
        // transcode
        "omarchy-transcode" => Some(&["transcode", "convert"]),
        "omarchy-transcode-ascii" => Some(&["transcode", "ascii"]),
        // voxtype
        "omarchy-voxtype-config" => Some(&["voxtype", "config"]),
        "omarchy-voxtype-model" => Some(&["voxtype", "model"]),
        "omarchy-voxtype-status" => Some(&["voxtype", "status"]),
        // webapp-handler
        "omarchy-webapp-handler-hey" => Some(&["webapp-handler", "hey"]),
        "omarchy-webapp-handler-zoom" => Some(&["webapp-handler", "zoom"]),
        // screenshot root alias
        "omarchy-screenshot" => Some(&["screenshot"]),
        // share root alias
        "omarchy-share" => Some(&["share"]),
        // apply (install-time plumbing)
        "omarchy-apply-hardware" => Some(&["apply", "hardware"]),
        "omarchy-apply-system" => Some(&["apply", "system"]),
        "omarchy-apply-lock" => Some(&["apply", "lock"]),
        _ => None,
    }
}

fn exec_hw(name: &str, args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/omarchy-hw-{}", omarchy_path, name);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

fn exec_bin(binary: &str, args: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
    let script = format!("{}/bin/{}", omarchy_path, binary);
    let err = std::process::Command::new(&script).args(args).exec();
    eprintln!("exec failed: {}", err);
    1
}

/// Recursively collect leaf command routes from a Clap subcommand tree.
/// Each entry is (binary_name, summary).
fn collect_leaf_routes(cmd: &clap::Command, stem: &str, out: &mut Vec<(String, String)>) {
    let subs: Vec<_> = cmd.get_subcommands()
        .filter(|c| !matches!(c.get_name(), "help"))
        .collect();

    if subs.is_empty() {
        // Leaf command
        let about = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();
        out.push((format!("omarchy-{}", stem), about));
    } else {
        for sub in subs {
            let sub_stem = format!("{}-{}", stem, sub.get_name());
            collect_leaf_routes(sub, &sub_stem, out);
        }
    }
}

/// Print the help for a named subcommand to stdout and return 0.
/// Used when a group command is invoked with no subcommand — mirrors the bash
/// router behaviour of showing group help to stdout rather than erroring.
fn print_group_help(group: &str) -> i32 {
    let mut cmd = Cli::command();
    if let Some(sub) = cmd.find_subcommand_mut(group) {
        let _ = sub.print_long_help();
    }
    0
}

fn handle_partial_prefix(args: &[String]) -> Option<i32> {
    // Need at least: [binary, group, partial-or-flag]
    if args.len() < 2 {
        return None;
    }

    let group = &args[1];

    // Skip pure flags at position 1 (e.g. `omarchy --help` should let Clap handle it)
    if group.starts_with('-') {
        return None;
    }

    // Get all subcommand names for the group from the Clap tree
    let root = Cli::command();

    if let Some(group_cmd) = root.get_subcommands().find(|c| c.get_name() == group.as_str()) {
        // Known Clap group — partial subcommand prefix matching
        if args.len() < 3 {
            return None;
        }
        let prefix = args[2..].iter()
            .filter(|a| !a.starts_with('-'))
            .cloned()
            .collect::<Vec<_>>()
            .join("-");
        if prefix.is_empty() {
            return None;
        }

        let matches: Vec<_> = group_cmd
            .get_subcommands()
            .filter(|c| c.get_name().starts_with(prefix.as_str()))
            .collect();

        if matches.is_empty() {
            return None;
        }

        // Print matching routes
        for sub in &matches {
            let sub_name = sub.get_name();
            // Convert "asus-rog" → "asus rog" for display
            let sub_words = sub_name.replace('-', " ");
            let route = format!("omarchy {} {}", group, sub_words);
            let about = sub.get_about().map(|s| s.to_string()).unwrap_or_default();
            if about.is_empty() {
                println!("{}", route);
            } else {
                println!("{}  {}", route, about);
            }
        }

        return Some(0);
    }

    // Shell-script-only group: scan $OMARCHY_PATH/bin and the argv[0] directory
    // for omarchy-<group>-* scripts and the parent omarchy-<group> binary.
    // Also checks alias metadata to handle e.g. `parenthelp-alias` → `omarchy-parenthelp`.
    let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();

    // Directories to scan (deduplicated).
    let mut scan_dirs: Vec<std::path::PathBuf> = Vec::new();
    if !omarchy_path.is_empty() {
        scan_dirs.push(std::path::Path::new(&omarchy_path).join("bin"));
    }
    // Also scan the directory of argv[0] for extension scripts (used in tests).
    if let Some(argv0) = std::env::args().next() {
        if let Some(parent) = std::path::Path::new(&argv0).parent() {
            let parent_buf = parent.to_path_buf();
            if !parent_buf.as_os_str().is_empty() && !scan_dirs.contains(&parent_buf) {
                scan_dirs.push(parent_buf);
            }
        }
    }

    if scan_dirs.is_empty() {
        return None;
    }

    /// Read the summary, aliases, and alias routes from a script's metadata header.
    fn read_script_meta(path: &std::path::Path) -> (String, Vec<String>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return (String::new(), Vec::new()),
        };
        let mut summary = String::new();
        let mut aliases: Vec<String> = Vec::new();
        let mut seen_non_meta = false;
        for (i, line) in content.lines().enumerate() {
            if i == 0 && line.starts_with("#!") { continue; }
            if line.trim().is_empty() { continue; }
            if !line.starts_with('#') { seen_non_meta = true; break; }
            if let Some(v) = line.strip_prefix("# omarchy:summary=") {
                summary = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("# omarchy:alias=") {
                aliases.push(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("# omarchy:aliases=") {
                aliases.push(v.trim().to_string());
            }
            let _ = seen_non_meta;
        }
        (summary, aliases)
    }

    let child_prefix = format!("omarchy-{}-", group);
    let parent_name = format!("omarchy-{}", group);

    // (binary_name, summary, full_path)
    let mut scripts: Vec<(String, String, std::path::PathBuf)> = Vec::new();
    // Parent binary path (if found)
    let mut parent_script: Option<(String, std::path::PathBuf)> = None; // (summary, path)
    // Alias resolution: if `group` is an alias for some other binary, record it.
    let mut alias_parent: Option<(String, String, std::path::PathBuf)> = None; // (real_name, summary, path)

    let alias_needle = format!("omarchy {}", group);

    for dir in &scan_dirs {
        let dir_entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(e) => e.flatten().collect(),
            Err(_) => continue,
        };
        for entry in dir_entries {
            let fname = match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let path = entry.path();
            if fname == parent_name {
                let (summary, _) = read_script_meta(&path);
                parent_script = Some((summary, path.clone()));
            } else if fname.starts_with(&child_prefix) {
                let (summary, _) = read_script_meta(&path);
                scripts.push((fname, summary, path));
            } else if alias_parent.is_none() {
                // Check if this script has an alias matching our group name
                let (summary, aliases) = read_script_meta(&path);
                if aliases.iter().any(|a| a == &alias_needle) {
                    alias_parent = Some((fname, summary, path));
                }
            }
        }
    }

    // Also scan root-level Rust Clap commands matching <group>-* or <group>
    let group_prefix = format!("{}-", group);
    let mut clap_routes: Vec<(String, String)> = Vec::new();
    for sub in root.get_subcommands() {
        let sub_name = sub.get_name();
        if sub_name.starts_with(group_prefix.as_str()) || sub_name == group.as_str() {
            collect_leaf_routes(sub, sub_name, &mut clap_routes);
        }
    }
    for (binary, summary) in clap_routes {
        if !scripts.iter().any(|(b, _, _)| b == &binary) {
            scripts.push((binary, summary, std::path::PathBuf::new()));
        }
    }

    // Resolve the canonical parent: direct parent > alias parent > nothing
    let resolved_parent: Option<(String, String, std::path::PathBuf)> = if let Some((summary, path)) = parent_script {
        Some((parent_name.clone(), summary, path))
    } else if let Some((name, summary, path)) = alias_parent.clone() {
        Some((name, summary, path))
    } else {
        None
    };

    // If no children AND no parent, there's nothing to do for this group.
    if scripts.is_empty() && resolved_parent.is_none() {
        return None;
    }

    // Helper: check if `--help` or `-h` appears before any `--` in the given slice.
    fn has_help_flag(tokens: &[String]) -> bool {
        for t in tokens {
            if t == "--" { break; }
            if t == "--help" || t == "-h" { return true; }
        }
        false
    }

    // Helper: check if `--json` appears before any `--` in the given slice.
    fn has_json_flag(tokens: &[String]) -> bool {
        for t in tokens {
            if t == "--" { break; }
            if t == "--json" { return true; }
        }
        false
    }

    /// Print the group listing for the current group.
    fn print_group_listing(group: &str, scripts: &[(String, String, std::path::PathBuf)], parent: Option<&(String, String, std::path::PathBuf)>) {
        let cap = format!("{}{}", &group[..1].to_uppercase(), &group[1..]);
        println!("{} commands", cap);
        println!();
        let mut display: Vec<(String, String)> = Vec::new();
        if let Some((name, summary, _)) = parent {
            display.push((name.clone(), summary.clone()));
        }
        for (b, s, _) in scripts {
            display.push((b.clone(), s.clone()));
        }
        display.sort_by(|a, b| a.0.cmp(&b.0));
        display.dedup_by(|a, b| a.0 == b.0);
        for (binary, summary) in &display {
            let stem = binary.strip_prefix("omarchy-").unwrap_or(binary);
            let route = format!("omarchy {}", stem.replace('-', " "));
            if summary.is_empty() {
                println!("  {}", route);
            } else {
                println!("  {}  {}", route, summary);
            }
        }
    }

    // Bare group invocation (args.len() < 3) or explicit --help at position 2
    let pos2_is_help = args.len() >= 3 && (args[2] == "--help" || args[2] == "-h");
    if args.len() < 3 || pos2_is_help {
        print_group_listing(group, &scripts, resolved_parent.as_ref());
        return Some(0);
    }

    // From here, args[2..] contains the subcommand words + flags.
    let remaining = &args[2..];

    // Try to match a specific child command from the remaining args.
    // Non-flag tokens are joined with hyphens to form the child suffix.
    // We stop accumulating words when we find an exact child match.
    let mut child_match: Option<(String, String, std::path::PathBuf, Vec<String>)> = None; // (name, summary, path, leftover_args)

    let word_tokens: Vec<String> = remaining.iter()
        .take_while(|t| t.as_str() != "--")
        .filter(|t| !t.starts_with('-'))
        .cloned()
        .collect();

    for word_count in (1..=word_tokens.len()).rev() {
        let joined = word_tokens[..word_count].join("-");
        let candidate = format!("omarchy-{}-{}", group, joined);
        if let Some((_, s, p)) = scripts.iter().find(|(b, _, _)| b == &candidate) {
            // Compute the leftover args: everything in remaining that isn't the matched words
            // (preserving flags and post-`--` tokens).
            let mut leftover: Vec<String> = Vec::new();
            let mut words_consumed = 0usize;
            let mut past_dashdash = false;
            for tok in remaining {
                if tok == "--" {
                    past_dashdash = true;
                    leftover.push(tok.clone());
                    continue;
                }
                if !past_dashdash && !tok.starts_with('-') && words_consumed < word_count {
                    words_consumed += 1;
                    continue; // consumed as part of the child name
                }
                leftover.push(tok.clone());
            }
            child_match = Some((candidate, s.clone(), p.clone(), leftover));
            break;
        }
    }

    if let Some((child_name, child_summary, child_path, leftover)) = child_match {
        // Check for --help flag in leftover (before --)
        if has_help_flag(&leftover) {
            let stem = child_name.strip_prefix("omarchy-").unwrap_or(&child_name);
            let route = format!("omarchy {}", stem.replace('-', " "));
            if has_json_flag(&leftover) {
                // JSON help mode
                let json = serde_json::json!({
                    "ok": true,
                    "route": route,
                    "binary": child_name,
                    "summary": child_summary,
                });
                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
            } else {
                println!("{}", route);
                if !child_summary.is_empty() {
                    println!();
                    println!("{}", child_summary);
                }
                if !child_path.as_os_str().is_empty() {
                    println!();
                    println!("Binary: {}", child_name);
                }
            }
            return Some(0);
        }

        if !child_path.as_os_str().is_empty() {
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new(&child_path).args(&leftover).exec();
            eprintln!("exec failed: {}", err);
            return Some(1);
        }
        // Rust Clap child (no path): fall through to print
        let stem = child_name.strip_prefix("omarchy-").unwrap_or(&child_name);
        let route = format!("omarchy {}", stem.replace('-', " "));
        if child_summary.is_empty() {
            println!("{}", route);
        } else {
            println!("{}  {}", route, child_summary);
        }
        return Some(0);
    }

    // No child matched.  Check if --help is in remaining (before --).
    // When a parent binary exists, show command help (binary name + related commands).
    // Otherwise fall back to the group listing.
    if has_help_flag(remaining) {
        if has_json_flag(remaining) {
            // JSON help: show the parent command's JSON (or a group listing in JSON).
            if let Some((parent_name, parent_summary, _)) = &resolved_parent {
                let stem = parent_name.strip_prefix("omarchy-").unwrap_or(parent_name);
                let route = format!("omarchy {}", stem.replace('-', " "));
                let json = serde_json::json!({
                    "ok": true,
                    "route": route,
                    "binary": parent_name,
                    "summary": parent_summary,
                });
                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
            } else {
                let json = serde_json::json!({"ok": false, "group": group});
                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
            }
            return Some(0);
        }
        if let Some((parent_name, parent_summary, _)) = &resolved_parent {
            // Show single-command help: summary + binary name + related commands
            let stem = parent_name.strip_prefix("omarchy-").unwrap_or(parent_name);
            let route = format!("omarchy {}", stem.replace('-', " "));
            println!("Usage:");
            println!("  {}", route);
            println!();
            println!("{}", parent_summary);
            println!();
            println!("Binary:");
            println!("  {}", parent_name);
            if !scripts.is_empty() {
                println!();
                println!("Related commands:");
                let mut related: Vec<_> = scripts.iter()
                    .map(|(b, s, _)| (b.clone(), s.clone()))
                    .collect();
                related.sort_by(|a, b| a.0.cmp(&b.0));
                for (binary, summary) in &related {
                    let child_stem = binary.strip_prefix("omarchy-").unwrap_or(binary);
                    let child_route = format!("omarchy {}", child_stem.replace('-', " "));
                    if summary.is_empty() {
                        println!("  {}", child_route);
                    } else {
                        println!("  {}  {}", child_route, summary);
                    }
                }
            }
        } else {
            // No parent binary — show full group listing
            print_group_listing(group, &scripts, None);
        }
        return Some(0);
    }

    // No child match, no help flag.  If there is a parent binary, exec it with all remaining args.
    if let Some((_, _, parent_path)) = &resolved_parent {
        if !parent_path.as_os_str().is_empty() {
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new(parent_path).args(remaining).exec();
            eprintln!("exec failed: {}", err);
            return Some(1);
        }
    }

    // Partial subcommand matching — show all children that start with the prefix.
    let sub_prefix = word_tokens.join("-");
    if !sub_prefix.is_empty() {
        let match_prefix = format!("omarchy-{}-{}", group, sub_prefix);
        let matching: Vec<_> = scripts.iter()
            .filter(|(b, _, _)| b.starts_with(&match_prefix))
            .collect();
        if !matching.is_empty() {
            for (binary, summary, _) in &matching {
                let stem = binary.strip_prefix("omarchy-").unwrap_or(binary);
                let route = format!("omarchy {}", stem.replace('-', " "));
                if summary.is_empty() {
                    println!("{}", route);
                } else {
                    println!("{}  {}", route, summary);
                }
            }
            return Some(0);
        }
    }

    None
}

/// Attempt to normalise multi-word subcommand invocations into the hyphenated
/// names Clap expects.  For example:
///
///   ["omarchy", "dev", "benchmark", "cli", "--repeat=1"]
///   → ["omarchy", "dev", "benchmark-cli", "--repeat=1"]
///
/// The function:
/// 1. Finds the group subcommand (args[1]).
/// 2. Greedily joins non-flag tokens from args[2..] with hyphens while at
///    least one subcommand of that group starts with the accumulated prefix.
/// 3. When the accumulated prefix exactly matches exactly one subcommand name,
///    it splices the normalised name in place of the consumed word tokens.
///
/// Returns the original slice unchanged when no normalisation is needed.
fn normalize_multiword_subcmd(args: Vec<String>) -> Vec<String> {
    // Need at least [binary, group, word, ...]
    if args.len() < 3 {
        return args;
    }
    let group = &args[1];
    if group.starts_with('-') {
        return args;
    }
    let root = Cli::command();
    let group_cmd = match root.get_subcommands().find(|c| c.get_name() == group.as_str()) {
        Some(c) => c,
        None => return args, // shell-script-only group, nothing to normalise
    };

    // Walk forward through args[2..], accumulating non-flag tokens into a
    // hyphenated prefix.  Stop when no subcommand starts with the prefix.
    let mut prefix = String::new();
    let mut word_count = 0usize;
    let mut exact_match: Option<String> = None;

    for tok in &args[2..] {
        if tok.starts_with('-') {
            // Flag — stop accumulating subcommand words.
            break;
        }
        let candidate = if prefix.is_empty() {
            tok.clone()
        } else {
            format!("{}-{}", prefix, tok)
        };

        // Check if any subcommand name still starts with this candidate.
        let still_viable = group_cmd
            .get_subcommands()
            .any(|c| c.get_name().starts_with(candidate.as_str()));
        if !still_viable {
            break;
        }
        prefix = candidate;
        word_count += 1;

        // Check for an exact match at this point.
        let n_exact = group_cmd
            .get_subcommands()
            .filter(|c| c.get_name() == prefix.as_str())
            .count();
        if n_exact == 1 {
            exact_match = Some(prefix.clone());
        }
        // If we've consumed more words than the longest subcommand name could
        // have, stop.
        let max_depth = group_cmd
            .get_subcommands()
            .map(|c| c.get_name().chars().filter(|&ch| ch == '-').count() + 1)
            .max()
            .unwrap_or(1);
        if word_count >= max_depth {
            break;
        }
    }

    // Only rewrite when we have an exact match and consumed more than 1 word
    // (single-word subcommands parse fine on their own).
    match exact_match {
        Some(name) if word_count > 1 => {
            let mut new_args = vec![args[0].clone(), args[1].clone(), name];
            // Remaining args: skip the word_count tokens from args[2..]
            new_args.extend_from_slice(&args[2 + word_count..]);
            new_args
        }
        _ => args,
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

    // Normalise multi-word subcommand calls (e.g. "dev benchmark cli") into
    // the hyphenated form Clap expects ("dev benchmark-cli").
    let args = normalize_multiword_subcmd(args);

    let cli = match Cli::try_parse_from(&args) {
        Ok(c) => c,
        Err(err) => {
            // DisplayHelp / DisplayVersion are intentional — let Clap handle them
            // directly so that e.g. `omarchy theme set --help` shows the command's
            // long_about rather than falling through to handle_partial_prefix.
            if matches!(err.kind(), clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion) {
                err.exit();
            }
            // When invoked with no subcommand, Clap would show help on stderr and
            // exit 2.  Mirror the bash router behaviour: print help to stdout and
            // exit 0 so callers can safely capture the output.
            if matches!(err.kind(), clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand) {
                let _ = Cli::command().print_long_help();
                process::exit(0);
            }
            if let Some(code) = handle_partial_prefix(&args) {
                process::exit(code);
            }
            err.exit();
        }
    };
    let code = match cli.cmd {
        Cmd::Pkg { subcmd } => match subcmd {
            PkgCmd::Add { name } => cmds::add::run(&name),
            PkgCmd::Drop { name } => cmds::drop::run(&name),
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
            None => print_group_help("install"),
            Some(InstallCmd::GamingSteam) => cmds::install::gaming::steam(),
            Some(InstallCmd::GamingHeroic) => cmds::install::gaming::heroic(),
            Some(InstallCmd::GamingLutris) => cmds::install::gaming::lutris(),
            Some(InstallCmd::GamingRetroarch) => cmds::install::gaming::retroarch(),
            Some(InstallCmd::GamingXboxControllers) => cmds::install::gaming::xbox_controllers(),
            Some(InstallCmd::GamingXboxCloud) => cmds::install::gaming::xbox_cloud(),
            Some(InstallCmd::GamingBattlenet) => cmds::install::gaming::battlenet(),
            Some(InstallCmd::GamingGeforceNow) => cmds::install::gaming::geforce_now(),
            Some(InstallCmd::GamingGpuLib32) => { cmds::install::gaming::gpu_lib32(); 0 },
            Some(InstallCmd::EditorHelix) => cmds::install::editor::helix(),
            Some(InstallCmd::EditorVscode) => cmds::install::editor::vscode(),
            Some(InstallCmd::EditorEmacs) => cmds::install::editor::emacs(),
            Some(InstallCmd::EditorZed) => cmds::install::editor::zed(),
            Some(InstallCmd::AiClaude) => cmds::install::ai::claude(),
            Some(InstallCmd::AiHermes) => cmds::install::ai::hermes(),
            Some(InstallCmd::AiT3code) => cmds::install::ai::t3code(),
            Some(InstallCmd::AiChatgpt) => cmds::install::ai::chatgpt(),
            Some(InstallCmd::Browser { name }) => cmds::install::browser::install(name),
            Some(InstallCmd::DevEnv { name }) => cmds::install::devenv::install(name),
            Some(InstallCmd::Terminal { name }) => cmds::install::terminal::install(name),
            Some(InstallCmd::ChromiumClaude) => cmds::install::chromium::claude(),
            Some(InstallCmd::ChromiumCopyUrl) => cmds::install::chromium::copy_url(),
            Some(InstallCmd::ChromiumYtdlp) => cmds::install::chromium::ytdlp(),
            Some(InstallCmd::ChromiumGoogleAccount) => cmds::install::chromium::google_account(),
            Some(InstallCmd::Font { name, package, family }) => cmds::install::font::install(&name, &package, &family),
            Some(InstallCmd::AndLaunch { name, packages, desktop_id }) => cmds::install::and_launch::run(&name, &packages, &desktop_id),
            Some(InstallCmd::App { name, packages }) => cmds::install::app::run(&name, &packages),
            Some(InstallCmd::OpenclawCli { check, .. }) => cmds::install::openclaw_cli::run(check),
            Some(InstallCmd::DockerDbs { dbs }) => cmds::install::docker_dbs::run(&dbs),
            Some(InstallCmd::Preinstalls) => cmds::install::preinstalls::run(),
            Some(InstallCmd::ServiceOnce) => cmds::install::service_once::run(),
            Some(InstallCmd::Voxtype) => cmds::install::voxtype::run(),
            Some(InstallCmd::Tui { name, command, window_style, icon }) => {
                cmds::install::tui::run(name.as_deref(), command.as_deref(), window_style.as_deref(), icon.as_deref())
            }
            Some(InstallCmd::Webapp { name, url, icon, custom_exec, mime_types }) => {
                cmds::install::webapp::run(name.as_deref(), url.as_deref(), icon.as_deref(), custom_exec.as_deref(), mime_types.as_deref())
            }
            Some(InstallCmd::AiOpenclaw) => cmds::install::ai::openclaw(),
            Some(InstallCmd::HermesCli { check, owns, remove }) => cmds::install::hermes_cli::run(check, owns, remove),
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
            HwCmd::AsusRog => exec_hw("asus-rog", &[]),
            HwCmd::AsusExpertbookB9406 => exec_hw("asus-expertbook-b9406", &[]),
            HwCmd::AsusZenbookUx5406aa => exec_hw("asus-zenbook-ux5406aa", &[]),
            HwCmd::Clamshell => exec_hw("clamshell", &[]),
            HwCmd::DellXps13SidecarAmps => exec_hw("dell-xps13-sidecar-amps", &[]),
            HwCmd::DellXpsHapticTouchpad => exec_hw("dell-xps-haptic-touchpad", &[]),
            HwCmd::DellXpsOled => exec_hw("dell-xps-oled", &[]),
            HwCmd::Display => exec_hw("display", &[]),
            HwCmd::ElegatoCamlink4k => exec_hw("elgato-camlink-4k", &[]),
            HwCmd::ExternalMonitors => exec_hw("external-monitors", &[]),
            HwCmd::Fingerprint => exec_hw("fingerprint", &[]),
            HwCmd::Framework16 => exec_hw("framework16", &[]),
            HwCmd::HybridGpu => exec_hw("hybrid-gpu", &[]),
            HwCmd::Intel => exec_hw("intel", &[]),
            HwCmd::IntelPtl => exec_hw("intel-ptl", &[]),
            HwCmd::IntelSof => exec_hw("intel-sof", &[]),
            HwCmd::Laptop => exec_hw("laptop", &[]),
            HwCmd::LaptopClosed => exec_hw("laptop-closed", &[]),
            HwCmd::Match { args } => exec_hw("match", &args),
            HwCmd::Nvidia => exec_hw("nvidia", &[]),
            HwCmd::NvidiaGsp => exec_hw("nvidia-gsp", &[]),
            HwCmd::NvidiaWithoutGsp => exec_hw("nvidia-without-gsp", &[]),
            HwCmd::Surface => exec_hw("surface", &[]),
            HwCmd::Touchpad => exec_hw("touchpad", &[]),
            HwCmd::Touchscreen => exec_hw("touchscreen", &[]),
            HwCmd::Vulkan => exec_hw("vulkan", &[]),
            HwCmd::Webcam => exec_hw("webcam", &[]),
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
        Cmd::Version { subcmd } => match subcmd {
            None | Some(VersionCmd::Show) => cmds::version_cmd::version(),
            Some(VersionCmd::Branch) => cmds::version_cmd::branch(),
        },
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
            None => print_group_help("toggle"),
            Some(ToggleCmd::Flag { flag_name, action }) => cmds::toggle::flag(&flag_name, &action),
            Some(ToggleCmd::Bar { action }) => cmds::toggle::bar(&action),
            Some(ToggleCmd::CrashCapture) => cmds::toggle::crash_capture(),
            Some(ToggleCmd::Enabled { flag_name }) => cmds::toggle::enabled(&flag_name),
            Some(ToggleCmd::FullscreenDesktop { action }) => cmds::toggle::fullscreen_desktop(&action),
            Some(ToggleCmd::HybridGpu) => cmds::toggle::hybrid_gpu(),
            Some(ToggleCmd::Idle { action }) => cmds::toggle::idle(&action),
            Some(ToggleCmd::InputDevice { kind, action }) => cmds::toggle::input_device(&kind, &action),
            Some(ToggleCmd::Nightlight { status }) => cmds::toggle::nightlight(status),
            Some(ToggleCmd::NotificationSilencing) => cmds::toggle::notification_silencing(),
            Some(ToggleCmd::Screensaver) => cmds::toggle::screensaver(),
            Some(ToggleCmd::Suspend) => cmds::toggle::suspend(),
            Some(ToggleCmd::Touchpad { action }) => cmds::toggle::touchpad(&action),
            Some(ToggleCmd::Touchscreen { action }) => cmds::toggle::touchscreen(&action),
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
        Cmd::Agent { subcmd } => match subcmd {
            AgentCmd::Run { inline, pick, prompt } => {
                cmds::agent::run(inline, pick, prompt.as_deref())
            }
            AgentCmd::Crash { pid, comm, exe, signal } => {
                cmds::agent::crash(&pid, comm.as_deref(), exe.as_deref(), signal.as_deref())
            }
            AgentCmd::Prompt { inline, prompt } => {
                cmds::agent::agent_prompt(inline, &prompt)
            }
            AgentCmd::UsageClaude { args } => cmds::agent::usage_claude(&args),
            AgentCmd::UsageCodex { args } => cmds::agent::usage_codex(&args),
            AgentCmd::UsageFireworks { args } => cmds::agent::usage_fireworks(&args),
            AgentCmd::UsageUpdate { force, limits_only, except, agents } => {
                cmds::agent::usage_update(force, limits_only, &except, &agents)
            }
        },
        Cmd::Branding { subcmd } => match subcmd {
            BrandingCmd::About { mode } => cmds::branding::about(&mode),
            BrandingCmd::Screensaver { mode } => cmds::branding::screensaver(&mode),
        },
        Cmd::Capture { subcmd } => match subcmd {
            CaptureCmd::Qr => cmds::capture::qr(),
            CaptureCmd::Region { mode, keep_freeze, match_monitor } => {
                cmds::capture::region(mode.as_deref(), keep_freeze, match_monitor)
            }
            CaptureCmd::Screenrecording { args } => cmds::capture::screenrecording(&args),
            CaptureCmd::ScreenrecordingWithWebcam => cmds::capture::screenrecording_with_webcam(),
            CaptureCmd::Screenshot { args } => cmds::capture::screenshot(&args),
            CaptureCmd::Text => cmds::capture::text(),
            CaptureCmd::WebcamList => cmds::capture::webcam_list(),
            CaptureCmd::WebcamResize { args } => cmds::capture::webcam_resize(&args),
        },
        Cmd::ChromiumCopyUrlHost => cmds::chromium_host::copy_url_host(),
        Cmd::ChromiumYtdlpHost { args } => cmds::chromium_host::ytdlp_host(&args),
        Cmd::Clipboard { subcmd } => match subcmd {
            ClipboardCmd::Open { history_index } => cmds::clipboard::open(history_index),
            ClipboardCmd::PasteFile { copy_only, mime_type, path } => {
                cmds::clipboard::paste_file(copy_only, &mime_type, &path)
            }
            ClipboardCmd::PasteText { shift_insert, copy_only, history_index, text } => {
                cmds::clipboard::paste_text(shift_insert, copy_only, history_index, &text)
            }
        },
        Cmd::Crash { subcmd } => match subcmd {
            CrashCmd::Mute { program, action } => {
                cmds::crash::mute(program.as_deref(), &action)
            }
            CrashCmd::Watch => cmds::crash::watch(),
        },
        Cmd::Debug { subcmd } => match subcmd {
            DebugCmd::Info { no_sudo, print } => cmds::debug::run(no_sudo, print),
            DebugCmd::Idle { log_lines } => cmds::debug::idle(log_lines),
        },
        Cmd::DefaultApp { subcmd } => match subcmd {
            DefaultCmd::Agent { install, name } => {
                cmds::default_cmd::agent(install, name.as_deref())
            }
            DefaultCmd::Browser { install, name } => {
                cmds::default_cmd::browser(install, name.as_deref())
            }
            DefaultCmd::Editor { install, name } => {
                cmds::default_cmd::editor(install, name.as_deref())
            }
            DefaultCmd::Terminal { install, name } => {
                cmds::default_cmd::terminal(install, name.as_deref())
            }
        },
        Cmd::Hook { subcmd } => match subcmd {
            HookCmd::Run { name, args } => cmds::hook::run(&name, &args),
            HookCmd::Install { hook_type, file } => cmds::hook::install(&hook_type, &file),
        },
        Cmd::Completions { shell } => {
            generate(shell, &mut Cli::command(), "omarchy", &mut std::io::stdout());
            0
        }
        // ── Batch 3 ──────────────────────────────────────────────────────────
        Cmd::Hyprland { subcmd } => match subcmd {
            HyprlandCmd::FocusApp { app } => cmds::hyprland::focus_app(&app),
            HyprlandCmd::MonitorClamshell => cmds::hyprland::monitor_clamshell(),
            HyprlandCmd::MonitorExternalActive => cmds::hyprland::monitor_external_active(),
            HyprlandCmd::MonitorFocused => cmds::hyprland::monitor_focused(),
            HyprlandCmd::MonitorFocusedApple { monitor } => {
                cmds::hyprland::monitor_focused_apple(monitor.as_deref())
            }
            HyprlandCmd::MonitorInternal { action } => cmds::hyprland::monitor_internal(&action),
            HyprlandCmd::MonitorInternalMirror { action } => {
                cmds::hyprland::monitor_internal_mirror(&action)
            }
            HyprlandCmd::MonitorLaptop => cmds::hyprland::monitor_laptop(),
            HyprlandCmd::MonitorModeless => cmds::hyprland::monitor_modeless(),
            HyprlandCmd::MonitorScaling { args } => cmds::hyprland::monitor_scaling(&args),
            HyprlandCmd::MonitorWatch => cmds::hyprland::monitor_watch(),
            HyprlandCmd::ReloadGuard { args } => cmds::hyprland::reload_guard(&args),
            HyprlandCmd::SessionLocked => cmds::hyprland::session_locked(),
            HyprlandCmd::Toggle { flag_name, action } => {
                cmds::hyprland::toggle(&flag_name, &action)
            }
            HyprlandCmd::ToggleDisabled { flag_name } => {
                cmds::hyprland::toggle_disabled(&flag_name)
            }
            HyprlandCmd::ToggleEnabled { flag_name } => {
                cmds::hyprland::toggle_enabled(&flag_name)
            }
            HyprlandCmd::WindowCloseAll => cmds::hyprland::window_close_all(),
            HyprlandCmd::WindowGapsToggle => cmds::hyprland::window_gaps_toggle(),
            HyprlandCmd::WindowPop { args } => cmds::hyprland::window_pop(&args),
            HyprlandCmd::WindowSingleSquareAspectToggle => {
                cmds::hyprland::window_single_square_aspect_toggle()
            }
            HyprlandCmd::WindowTiledFullscreenToggle => {
                cmds::hyprland::window_tiled_fullscreen_toggle()
            }
            HyprlandCmd::WindowTransparencyToggle => {
                cmds::hyprland::window_transparency_toggle()
            }
            HyprlandCmd::WindowWidth { args } => cmds::hyprland::window_width(&args),
            HyprlandCmd::WorkspaceLayoutToggle => cmds::hyprland::workspace_layout_toggle(),
        },
        Cmd::System { subcmd } => match subcmd {
            SystemCmd::FactoryReset { args } => cmds::system::factory_reset(&args),
            SystemCmd::FactoryResetFinish { args } => cmds::system::factory_reset_finish(&args),
            SystemCmd::LidClose => cmds::system::lid_close(),
            SystemCmd::Lock => cmds::system::lock(),
            SystemCmd::Logout => cmds::system::logout(),
            SystemCmd::Reboot => cmds::system::reboot(),
            SystemCmd::Shutdown => cmds::system::shutdown(),
            SystemCmd::SleepLock { args } => cmds::system::sleep_lock(&args),
            SystemCmd::SleepMonitor { args } => cmds::system::sleep_monitor(&args),
            SystemCmd::Stats { args } => cmds::system::stats(&args),
            SystemCmd::Wake => cmds::system::wake(),
        },
        Cmd::Launch { subcmd } => match subcmd {
            LaunchCmd::Onepassword => cmds::launch::onepassword(),
            LaunchCmd::About { args } => cmds::launch::about(&args),
            LaunchCmd::Battlenet { args } => cmds::launch::battlenet(&args),
            LaunchCmd::Browser { args } => cmds::launch::browser(&args),
            LaunchCmd::ConfigEditor { args } => cmds::launch::config_editor(&args),
            LaunchCmd::DiscordCommunity => cmds::launch::discord_community(),
            LaunchCmd::DockerTui => cmds::launch::docker_tui(),
            LaunchCmd::Editor { args } => cmds::launch::editor(&args),
            LaunchCmd::FloatingTerminalWithPresentation { args } => {
                cmds::launch::floating_terminal_with_presentation(&args)
            }
            LaunchCmd::Nautilus => cmds::launch::nautilus(),
            LaunchCmd::NautilusCwd => cmds::launch::nautilus_cwd(),
            LaunchCmd::Openclaw { args } => cmds::launch::openclaw(&args),
            LaunchCmd::OrFocus { args } => cmds::launch::or_focus(&args),
            LaunchCmd::OrFocusTui { args } => cmds::launch::or_focus_tui(&args),
            LaunchCmd::OrFocusWebapp { args } => cmds::launch::or_focus_webapp(&args),
            LaunchCmd::Screensaver { args } => cmds::launch::screensaver(&args),
            LaunchCmd::Shell { args } => cmds::launch::shell(&args),
            LaunchCmd::Signal => cmds::launch::signal(),
            LaunchCmd::Spotify => cmds::launch::spotify(),
            LaunchCmd::Terminal { args } => cmds::launch::terminal(&args),
            LaunchCmd::TerminalHerdr => cmds::launch::terminal_herdr(),
            LaunchCmd::TerminalTmux => cmds::launch::terminal_tmux(),
            LaunchCmd::Tui { args } => cmds::launch::tui(&args),
            LaunchCmd::Webapp { args } => cmds::launch::webapp(&args),
        },
        Cmd::ShellIpc { args } => cmds::shell_cmd::run(&args),
        Cmd::Migrate { subcmd } => match subcmd {
            MigrateCmd::Run { args } => cmds::migrate::run(&args),
            MigrateCmd::Notify => cmds::migrate::notify(),
        },
        Cmd::Dns { args } => cmds::dns_cmd::run(&args),
        Cmd::MiseInstall { args } => cmds::mise_install::run(&args),
        Cmd::Network { subcmd } => match subcmd {
            NetworkCmd::Band { args } => cmds::network::band(&args),
            NetworkCmd::Password { args } => cmds::network::password(&args),
            NetworkCmd::Qr { args } => cmds::network::qr(&args),
            NetworkCmd::Speedtest { args } => cmds::network::speedtest(&args),
            NetworkCmd::Status { args } => cmds::network::status(&args),
        },
        // ── Batch 4 ──────────────────────────────────────────────────────────
        Cmd::Dev { subcmd } => match subcmd {
            DevCmd::AddMigration { args } => cmds::dev::add_migration(&args),
            DevCmd::BenchmarkCli { args } => cmds::dev::benchmark_cli(&args),
            DevCmd::BenchmarkThemeSwitcher { args } => cmds::dev::benchmark_theme_switcher(&args),
            DevCmd::Font { args } => cmds::dev::font(&args),
            DevCmd::ThemePreview { args } => cmds::dev::theme_preview(&args),
            DevCmd::UiPreview { args } => cmds::dev::ui_preview(&args),
        },
        Cmd::Drive { subcmd } => match subcmd {
            DriveCmd::Info { args } => cmds::drive::info(&args),
            DriveCmd::Password { args } => cmds::drive::password(&args),
            DriveCmd::Select { args } => cmds::drive::select(&args),
        },
        Cmd::FileSelect { args } => cmds::file_select::run(&args),
        Cmd::Menu { subcmd } => match subcmd {
            MenuCmd::Main { args } => cmds::menu::main_menu(&args),
            MenuCmd::Clipboard { args } => cmds::menu::clipboard(&args),
            MenuCmd::Emoji { args } => cmds::menu::emoji(&args),
            MenuCmd::EmojiInsert { args } => cmds::menu::emoji_insert(&args),
            MenuCmd::File { args } => cmds::menu::file(&args),
            MenuCmd::HerdrKeybindings { args } => cmds::menu::herdr_keybindings(&args),
            MenuCmd::Images { args } => cmds::menu::images(&args),
            MenuCmd::Input { args } => cmds::menu::input(&args),
            MenuCmd::Keybindings { args } => cmds::menu::keybindings(&args),
            MenuCmd::Plugin { args } => cmds::menu::plugin(&args),
            MenuCmd::Select { args } => cmds::menu::select(&args),
            MenuCmd::Share { args } => cmds::menu::share(&args),
            MenuCmd::Timezone { args } => cmds::menu::timezone(&args),
            MenuCmd::TmuxKeybindings { args } => cmds::menu::tmux_keybindings(&args),
        },
        Cmd::OpenclawOnboard { args } => cmds::openclaw_onboard::run(&args),
        Cmd::Plugin { subcmd } => match subcmd {
            PluginCmd::Add { args } => cmds::plugin::add(&args),
            PluginCmd::Catalog { args } => cmds::plugin::catalog(&args),
            PluginCmd::Clone { args } => cmds::plugin::clone(&args),
            PluginCmd::Disable { args } => cmds::plugin::disable(&args),
            PluginCmd::Enable { args } => cmds::plugin::enable(&args),
            PluginCmd::List { args } => cmds::plugin::list(&args),
            PluginCmd::Remove { args } => cmds::plugin::remove(&args),
            PluginCmd::Update { args } => cmds::plugin::update(&args),
            PluginCmd::Validate { args } => cmds::plugin::validate(&args),
        },
        Cmd::Refresh { subcmd } => match subcmd {
            RefreshCmd::Applications { args } => cmds::refresh::applications(&args),
            RefreshCmd::Chromium { args } => cmds::refresh::chromium(&args),
            RefreshCmd::Config { args } => cmds::refresh::config(&args),
            RefreshCmd::Herdr { args } => cmds::refresh::herdr(&args),
            RefreshCmd::Hyprland { args } => cmds::refresh::hyprland(&args),
            RefreshCmd::Hyprsunset { args } => cmds::refresh::hyprsunset(&args),
            RefreshCmd::Shell { args } => cmds::refresh::shell(&args),
            RefreshCmd::Tmux { args } => cmds::refresh::tmux(&args),
            RefreshCmd::Plymouth { args } => exec_bin("omarchy-refresh-plymouth", &args),
            RefreshCmd::Sddm { args } => exec_bin("omarchy-refresh-sddm", &args),
        },
        Cmd::Channel { subcmd } => match subcmd {
            ChannelCmd::Current { args } => exec_bin("omarchy-channel-current", &args),
            ChannelCmd::Set { args } => exec_bin("omarchy-channel-set", &args),
        },
        Cmd::Hibernation { subcmd } => match subcmd {
            HibernationCmd::Available => exec_bin("omarchy-hibernation-available", &[]),
            HibernationCmd::Remove { args } => exec_bin("omarchy-hibernation-remove", &args),
            HibernationCmd::Setup { args } => exec_bin("omarchy-hibernation-setup", &args),
        },
        Cmd::Plymouth { subcmd } => match subcmd {
            PlymouthCmd::Current => exec_bin("omarchy-plymouth-current", &[]),
            PlymouthCmd::List => exec_bin("omarchy-plymouth-list", &[]),
            PlymouthCmd::Preview { args } => exec_bin("omarchy-plymouth-preview", &args),
            PlymouthCmd::Reset { args } => exec_bin("omarchy-plymouth-reset", &args),
            PlymouthCmd::Set { args } => exec_bin("omarchy-plymouth-set", &args),
            PlymouthCmd::SetByTheme { args } => exec_bin("omarchy-plymouth-set-by-theme", &args),
            PlymouthCmd::Switcher { args } => exec_bin("omarchy-plymouth-switcher", &args),
        },
        Cmd::Theme { subcmd } => match subcmd {
            ThemeCmd::BgCache { args } => cmds::theme::bg_cache(&args),
            ThemeCmd::BgCurrent { args } => cmds::theme::bg_current(&args),
            ThemeCmd::BgInstall { args } => cmds::theme::bg_install(&args),
            ThemeCmd::BgNext { args } => cmds::theme::bg_next(&args),
            ThemeCmd::BgSet { args } => cmds::theme::bg_set(&args),
            ThemeCmd::BgSwitcher { args } => cmds::theme::bg_switcher(&args),
            ThemeCmd::Color { args } => cmds::theme::color(&args),
            ThemeCmd::ColorsFromAlacritty { args } => cmds::theme::colors_from_alacritty(&args),
            ThemeCmd::Current { args } => cmds::theme::current(&args),
            ThemeCmd::Dir { args } => cmds::theme::dir(&args),
            ThemeCmd::Extras { args } => cmds::theme::extras(&args),
            ThemeCmd::Install { args } => cmds::theme::install(&args),
            ThemeCmd::List { args } => cmds::theme::list(&args),
            ThemeCmd::Osc { args } => cmds::theme::osc(&args),
            ThemeCmd::Refresh { args } => cmds::theme::refresh(&args),
            ThemeCmd::Remove { args } => cmds::theme::remove(&args),
            ThemeCmd::Set { args } => cmds::theme::set(&args),
            ThemeCmd::SetBrowser { args } => cmds::theme::set_browser(&args),
            ThemeCmd::SetBrowserPolicy { args } => cmds::theme::set_browser_policy(&args),
            ThemeCmd::SetClaude { args } => cmds::theme::set_claude(&args),
            ThemeCmd::SetFoot { args } => cmds::theme::set_foot(&args),
            ThemeCmd::SetGnome { args } => cmds::theme::set_gnome(&args),
            ThemeCmd::SetHermes { args } => cmds::theme::set_hermes(&args),
            ThemeCmd::SetKeyboard { args } => cmds::theme::set_keyboard(&args),
            ThemeCmd::SetKeyboardAsusRog { args } => cmds::theme::set_keyboard_asus_rog(&args),
            ThemeCmd::SetKeyboardF16 { args } => cmds::theme::set_keyboard_f16(&args),
            ThemeCmd::SetObsidian { args } => cmds::theme::set_obsidian(&args),
            ThemeCmd::SetPi { args } => cmds::theme::set_pi(&args),
            ThemeCmd::SetT3code { args } => cmds::theme::set_t3code(&args),
            ThemeCmd::SetTemplates { args } => cmds::theme::set_templates(&args),
            ThemeCmd::SetTmux { args } => cmds::theme::set_tmux(&args),
            ThemeCmd::SetVscode { args } => cmds::theme::set_vscode(&args),
            ThemeCmd::Switcher { args } => cmds::theme::switcher(&args),
            ThemeCmd::Update { args } => cmds::theme::update(&args),
        },
        Cmd::Transcode { subcmd } => match subcmd {
            TranscodeCmd::Convert { args } => cmds::transcode::convert(&args),
            TranscodeCmd::Ascii { args } => cmds::transcode::ascii(&args),
        },
        Cmd::Voxtype { subcmd } => match subcmd {
            VoxtypeCmd::Config { args } => cmds::voxtype::config(&args),
            VoxtypeCmd::Model { args } => cmds::voxtype::model(&args),
            VoxtypeCmd::Status { args } => cmds::voxtype::status(&args),
        },
        Cmd::WebappHandler { subcmd } => match subcmd {
            WebappHandlerCmd::Hey { args } => cmds::webapp_handler::hey(&args),
            WebappHandlerCmd::Zoom { args } => cmds::webapp_handler::zoom(&args),
        },
        Cmd::Commands { all, json, check } => cmds::commands::run(all, json, check),
        Cmd::Screenshot { args } => cmds::capture::screenshot(&args),
        Cmd::Share { args } => cmds::menu::share(&args),
        Cmd::Apply { subcmd } => match subcmd {
            ApplyCmd::Hardware { args } => {
                use std::os::unix::process::CommandExt;
                let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
                let script = format!("{}/bin/omarchy-apply-hardware", omarchy_path);
                let err = std::process::Command::new(&script).args(&args).exec();
                eprintln!("exec failed: {}", err);
                1
            }
            ApplyCmd::System { args } => {
                use std::os::unix::process::CommandExt;
                let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
                let script = format!("{}/bin/omarchy-apply-system", omarchy_path);
                let err = std::process::Command::new(&script).args(&args).exec();
                eprintln!("exec failed: {}", err);
                1
            }
            ApplyCmd::Lock { args } => {
                use std::os::unix::process::CommandExt;
                let omarchy_path = std::env::var("OMARCHY_PATH").unwrap_or_default();
                let script = format!("{}/bin/omarchy-apply-lock", omarchy_path);
                let err = std::process::Command::new(&script).args(&args).exec();
                eprintln!("exec failed: {}", err);
                1
            }
        },
    };
    process::exit(code);
}
