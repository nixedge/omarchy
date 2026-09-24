{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      common,
      system,
      ...
    }:
    let
      muslLinker = "${pkgs.pkgsStatic.stdenv.cc}/bin/${pkgs.pkgsStatic.stdenv.cc.targetPrefix}cc";
      muslArgs = {
        src = common.rustSrc;
        strictDeps = true;
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = muslLinker;
        doCheck = false;
      };
    in
    {
      packages = {
        omarchy-nix-daemon = common.craneLib.buildPackage (muslArgs // {
          pname = "omarchy-nix-daemon";
          version = "0.1.0";
          cargoExtraArgs = "--package omarchy-nix-daemon";
        });

        omarchy-cli = common.craneLib.buildPackage (muslArgs // {
          pname = "omarchy-cli";
          version = "0.1.0";
          cargoExtraArgs = "--package omarchy-cli";
          postInstall = ''
            install -d $out/share/bash-completion/completions \
                       $out/share/zsh/site-functions \
                       $out/share/fish/vendor_completions.d
            # clap_complete splits hyphenated binary names differently in the
            # state-machine loop vs the opts handler section, producing mismatched
            # state names. Normalize by replacing the broken opts-section prefix.
            $out/bin/omarchy-cli completions bash \
              | sed 's/omarchy__subcmd__cli/omarchy__cli/g' \
              > $out/share/bash-completion/completions/omarchy-cli
            $out/bin/omarchy-cli completions zsh \
              > $out/share/zsh/site-functions/_omarchy-cli
            $out/bin/omarchy-cli completions fish \
              > $out/share/fish/vendor_completions.d/omarchy-cli.fish

            # Backward-compatible symlinks — argv[0] dispatch routes each name
            # to the right subcommand without wrapper scripts.
            for name in \
              omarchy-pkg-add \
              omarchy-pkg-drop \
              omarchy-pkg-install \
              omarchy-pkg-list \
              omarchy-pkg-missing \
              omarchy-pkg-present \
              omarchy-pkg-resolve \
              omarchy-pkg-search \
              omarchy-pkg-sync \
              omarchy-install-service-tailscale \
              omarchy-install-service-signal \
              omarchy-install-service-spotify \
              omarchy-install-service-1password \
              omarchy-install-service-dropbox \
              omarchy-install-service-nordvpn \
              omarchy-install-service-sunshine \
              omarchy-remove-service-tailscale \
              omarchy-remove-service-1password \
              omarchy-remove-service-dropbox \
              omarchy-remove-service-sunshine \
              omarchy-setup-security-fingerprint \
              omarchy-setup-security-fido2 \
              omarchy-setup-security-sshd \
              omarchy-setup-security-sudoless-docker \
              omarchy-remove-security-fingerprint \
              omarchy-remove-security-fido2 \
              omarchy-remove-security-sshd \
              omarchy-remove-security-sudoless-docker \
              omarchy-install-gaming-steam \
              omarchy-install-gaming-heroic \
              omarchy-install-gaming-lutris \
              omarchy-install-gaming-retroarch \
              omarchy-install-gaming-xbox-controllers \
              omarchy-install-gaming-xbox-cloud \
              omarchy-install-gaming-battlenet \
              omarchy-install-gaming-geforce-now \
              omarchy-install-gaming-gpu-lib32 \
              omarchy-remove-gaming-steam \
              omarchy-remove-gaming-heroic \
              omarchy-remove-gaming-lutris \
              omarchy-remove-gaming-retroarch \
              omarchy-remove-gaming-minecraft \
              omarchy-remove-gaming-xbox-controllers \
              omarchy-remove-gaming-xbox-cloud \
              omarchy-remove-gaming-battlenet \
              omarchy-remove-gaming-geforce-now \
              omarchy-install-editor-helix \
              omarchy-install-editor-vscode \
              omarchy-install-editor-emacs \
              omarchy-install-editor-zed \
              omarchy-install-ai-claude \
              omarchy-install-ai-hermes \
              omarchy-install-ai-t3-code \
              omarchy-install-ai-chatgpt \
              omarchy-remove-ai-claude \
              omarchy-remove-ai-hermes \
              omarchy-remove-ai-t3-code \
              omarchy-remove-ai-ollama \
              omarchy-remove-ai-chatgpt \
              omarchy-remove-ai-lm-studio \
              omarchy-remove-ai-grok-bot \
              omarchy-remove-ai-perplexity \
              omarchy-install-chromium-claude \
              omarchy-install-chromium-copy-url \
              omarchy-install-chromium-ytdlp \
              omarchy-install-chromium-google-account \
              omarchy-install-browser \
              omarchy-remove-browser \
              omarchy-install-dev-env \
              omarchy-remove-dev-env \
              omarchy-install-terminal \
              omarchy-install-font \
              omarchy-install-and-launch \
              omarchy-install-app \
              omarchy-install-openclaw-cli \
              omarchy-install-docker-dbs \
              omarchy-install-preinstalls \
              omarchy-install-service-once \
              omarchy-voxtype-install \
              omarchy-tui-install \
              omarchy-webapp-install \
              omarchy-remove-preinstalls \
              omarchy-voxtype-remove \
              omarchy-tui-remove \
              omarchy-tui-remove-all \
              omarchy-webapp-remove \
              omarchy-webapp-remove-all \
              omarchy-remove-launcher-entry \
              omarchy-install-hermes-cli \
              omarchy-remove-ai-openclaw \
              omarchy-install-ai-openclaw \
              omarchy-installed-service-tailscale \
              omarchy-installed-service-dropbox \
              omarchy-hw-asus-rog \
              omarchy-hw-asus-expertbook-b9406 \
              omarchy-hw-asus-zenbook-ux5406aa \
              omarchy-hw-clamshell \
              omarchy-hw-dell-xps13-sidecar-amps \
              omarchy-hw-dell-xps-haptic-touchpad \
              omarchy-hw-dell-xps-oled \
              omarchy-hw-display \
              omarchy-hw-elgato-camlink-4k \
              omarchy-hw-external-monitors \
              omarchy-hw-fingerprint \
              omarchy-hw-framework16 \
              omarchy-hw-hybrid-gpu \
              omarchy-hw-intel \
              omarchy-hw-intel-ptl \
              omarchy-hw-intel-sof \
              omarchy-hw-laptop \
              omarchy-hw-laptop-closed \
              omarchy-hw-match \
              omarchy-hw-nvidia \
              omarchy-hw-nvidia-gsp \
              omarchy-hw-nvidia-without-gsp \
              omarchy-hw-recover-internal-monitor \
              omarchy-hw-surface \
              omarchy-hw-touchpad \
              omarchy-hw-touchscreen \
              omarchy-hw-vulkan \
              omarchy-hw-webcam \
              omarchy-restart-app \
              omarchy-restart-audio \
              omarchy-restart-bluetooth \
              omarchy-restart-btop \
              omarchy-restart-gum \
              omarchy-restart-helix \
              omarchy-restart-herdr \
              omarchy-restart-hyprctl \
              omarchy-restart-hyprsunset \
              omarchy-restart-opencode \
              omarchy-restart-shell \
              omarchy-restart-terminal \
              omarchy-restart-tmux \
              omarchy-restart-trackpad \
              omarchy-restart-wifi \
              omarchy-restart-xcompose \
              omarchy-notification-battery \
              omarchy-notification-time \
              omarchy-notification-weather \
              omarchy-notification-dismiss \
              omarchy-notification-wait \
              omarchy-notification-send \
              omarchy-battery-present \
              omarchy-battery-low \
              omarchy-battery-status \
              omarchy-power-present \
              omarchy-cmd-missing \
              omarchy-cmd-present \
              omarchy-cmd-terminal-cwd \
              omarchy-state \
              omarchy-done \
              omarchy-show-done \
              omarchy-show-logo \
              omarchy-version \
              omarchy-version-branch \
              omarchy-powerprofiles-init \
              omarchy-powerprofiles-list \
              omarchy-powerprofiles-set \
              omarchy-osd \
              omarchy-windows-key \
              omarchy-windows-vm \
              omarchy-tailscale-receive \
              omarchy-tailscale-send \
              omarchy-update-dev \
              omarchy-update-firmware \
              omarchy-update-lock \
              omarchy-update-requires-free-space \
              omarchy-update-status \
              omarchy-update-stay-awake \
              omarchy-update-time \
              omarchy-update-user-notify \
              omarchy-setup-direct-boot \
              omarchy-sudo-docker \
              omarchy-sudo-keepalive \
              omarchy-sudo-passwordless \
              omarchy-screensaver \
              omarchy-git-url-check \
              omarchy-games-retro-cores \
              omarchy-games-retro-install \
              omarchy-monitor-state \
              omarchy-disk-speedtest \
              omarchy-reminder \
              omarchy-display-text-size \
              omarchy-audio-output-volume \
              omarchy-audio-output-switch \
              omarchy-audio-input-mute \
              omarchy-audio-output-sink \
              omarchy-audio-sink-availability \
              omarchy-audio-source-switch \
              omarchy-audio-input-set-default \
              omarchy-audio-output-set-default \
              omarchy-audio-tuning \
              omarchy-brightness-display \
              omarchy-brightness-display-apple \
              omarchy-brightness-display-ddc \
              omarchy-brightness-keyboard \
              omarchy-brightness-keyboard-mute \
              omarchy-toggle \
              omarchy-toggle-bar \
              omarchy-toggle-crash-capture \
              omarchy-toggle-enabled \
              omarchy-toggle-fullscreen-desktop \
              omarchy-toggle-hybrid-gpu \
              omarchy-toggle-idle \
              omarchy-toggle-input-device \
              omarchy-toggle-nightlight \
              omarchy-toggle-notification-silencing \
              omarchy-toggle-screensaver \
              omarchy-toggle-suspend \
              omarchy-toggle-touchpad \
              omarchy-toggle-touchscreen \
              omarchy-bluetooth-device \
              omarchy-bluetooth-power \
              omarchy-bar \
              omarchy-bar-text-color \
              omarchy-ascii \
              omarchy-font-current \
              omarchy-font-list \
              omarchy-font-set \
              omarchy-weather-icon \
              omarchy-weather-location \
              omarchy-weather-status \
              omarchy-agent \
              omarchy-agent-crash \
              omarchy-agent-prompt \
              omarchy-agent-usage-claude \
              omarchy-agent-usage-codex \
              omarchy-agent-usage-fireworks \
              omarchy-agent-usage-update \
              omarchy-branding-about \
              omarchy-branding-screensaver \
              omarchy-capture-qr \
              omarchy-capture-region \
              omarchy-capture-screenrecording \
              omarchy-capture-screenrecording-with-webcam \
              omarchy-capture-screenshot \
              omarchy-capture-text \
              omarchy-capture-webcam-list \
              omarchy-capture-webcam-resize \
              omarchy-chromium-copy-url-host \
              omarchy-chromium-ytdlp-host \
              omarchy-clipboard-open \
              omarchy-clipboard-paste-file \
              omarchy-clipboard-paste-text \
              omarchy-crash-mute \
              omarchy-crash-watch \
              omarchy-debug \
              omarchy-debug-idle \
              omarchy-default-agent \
              omarchy-default-browser \
              omarchy-default-editor \
              omarchy-default-terminal \
              omarchy-hook \
              omarchy-hook-install; do
              ln -s $out/bin/omarchy-cli $out/bin/$name
            done
          '';
        });

        omarchy = pkgs.stdenv.mkDerivation {
          pname = "omarchy";
          version = inputs.self.shortRev or "dev";
          src = inputs.self;

          nativeBuildInputs = [ pkgs.bash ];

          installPhase = ''
            runHook preInstall

            mkdir -p $out
            for d in bin config default shell themes migrations manual; do
              [ -d "$d" ] && cp -r "$d" $out/
            done
            # icon.png is referenced by default/chromium/extensions/copy-url/icon.png
            # via a ../../../../ symlink that resolves to the package root.
            [ -f icon.png ] && cp icon.png $out/

            # omarchy bash-router completions (for `omarchy pkg …` etc.)
            install -Dm644 completions/bash/omarchy \
              $out/share/bash-completion/completions/omarchy
            install -Dm644 completions/zsh/_omarchy \
              $out/share/zsh/site-functions/_omarchy
            install -Dm644 completions/fish/omarchy.fish \
              $out/share/fish/vendor_completions.d/omarchy.fish

            # Wayland session entry so SDDM discovers the omarchy session.
            mkdir -p $out/share/wayland-sessions
            install -m 0644 default/wayland-sessions/omarchy.desktop \
              $out/share/wayland-sessions/omarchy.desktop

            # Cinque-not-implemented shims for eliminated Arch-only commands.
            # These are prepended to PATH via environment.sessionVariables so
            # they shadow the originals without modifying bin/.
            mkdir -p $out/cinque-shims
            for cmd in \
              omarchy-pkg-add-aur \
              omarchy-pkg-aur-accessible \
              omarchy-pkg-aur-add \
              omarchy-pkg-aur-install \
              omarchy-pkg-install \
              omarchy-pkg-remove \
              omarchy-refresh-limine \
              omarchy-refresh-pacman \
              omarchy-reinstall \
              omarchy-reinstall-configs \
              omarchy-reinstall-pkgs \
              omarchy-snapshot \
              omarchy-update \
              omarchy-update-analyze-logs \
              omarchy-update-available \
              omarchy-update-aur-pkgs \
              omarchy-update-confirm \
              omarchy-update-keyring \
              omarchy-update-mise \
              omarchy-update-orphan-pkgs \
              omarchy-update-pacman-guard \
              omarchy-update-pkg-prune \
              omarchy-update-restart \
              omarchy-update-system-pkgs \
              omarchy-update-system-pkgs-when-conflicted \
              omarchy-update-pacman \
              omarchy-version-channel \
              omarchy-version-pkgs \
              omarchy-dev-pkg-test \
              omarchy-upgrade-to-quattro \
              omarchy-apply-hardware \
              omarchy-apply-system \
              omarchy-apply-lock \
              omarchy-channel-current \
              omarchy-channel-set \
              omarchy-hibernation-available \
              omarchy-hibernation-remove \
              omarchy-hibernation-setup; do
              install -m 0755 ${../pkgs/omarchy/cinque-not-implemented.sh} $out/cinque-shims/$cmd
            done


            runHook postInstall
          '';

          # patchShebangs rewrites #!/bin/bash to the nix-store bash path.
          # bin/omarchy's register_command already skips line 1 when it starts
          # with #!, so the header-grep is unaffected by shebang rewriting.

          # Required by services.displayManager.sessionPackages type check.
          passthru.providedSessions = [ "omarchy" ];
        };

        default = inputs.self.packages.${system}.omarchy;
      };
    };
}
