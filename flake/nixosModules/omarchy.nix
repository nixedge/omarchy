{ inputs, ... }:
{
  flake.nixosModules.omarchy =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      cfg = config.programs.omarchy;
      system = pkgs.stdenv.hostPlatform.system;
      omarchy = inputs.self.packages.${system}.omarchy;
      daemon = inputs.self.packages.${system}.omarchy-nix-daemon;
    in
    {
      options.programs.omarchy = {
        enable = lib.mkEnableOption "Omarchy Cinque desktop environment";
        user = lib.mkOption {
          type = lib.types.str;
          default = "omarchy";
          description = "Primary user that runs the Omarchy desktop session.";
        };
      };

      config = lib.mkIf cfg.enable {

        # ── Hyprland compositor ────────────────────────────────────────────────
        programs.hyprland = {
          enable = true;
          package = inputs.hyprland.packages.${system}.hyprland;
          portalPackage = inputs.hyprland.packages.${system}.xdg-desktop-portal-hyprland;
        };

        # ── Display manager ────────────────────────────────────────────────────
        # Register the omarchy.desktop wayland session file so SDDM discovers it.
        services.displayManager = {
          defaultSession = "omarchy";
          sessionPackages = [ omarchy ];
          sddm = {
            enable = true;
            wayland.enable = true;
          };
        };

        # ── Audio (PipeWire) ───────────────────────────────────────────────────
        security.rtkit.enable = true;
        services.pipewire = {
          enable = true;
          alsa = {
            enable = true;
            support32Bit = true;
          };
          pulse.enable = true;
        };

        # ── Networking ─────────────────────────────────────────────────────────
        networking.networkmanager = {
          enable = true;
          wifi.powersave = false;
          dns = "systemd-resolved";
        };
        services.resolved = {
          enable = true;
          settings.Resolve = {
            LLMNR = "no";
            MulticastDNS = "no";
          };
        };
        # Don't let network-online.target stall graphical.target for DHCP.
        systemd.services.NetworkManager-wait-online.enable = false;

        # ── mDNS/Bonjour ───────────────────────────────────────────────────────
        services.avahi = {
          enable = true;
          nssmdns4 = true;
        };

        # ── Printing ───────────────────────────────────────────────────────────
        services.printing.enable = true;

        # ── Power management ───────────────────────────────────────────────────
        services.power-profiles-daemon.enable = true;

        # ── Bluetooth ──────────────────────────────────────────────────────────
        hardware.bluetooth = {
          enable = true;
          powerOnBoot = true;
        };

        # ── Docker ─────────────────────────────────────────────────────────────
        virtualisation.docker = {
          enable = true;
          daemon.settings = {
            "log-driver" = "json-file";
            "log-opts" = {
              "max-size" = "10m";
              "max-file" = "5";
            };
            "dns" = [ "172.17.0.1" ];
            "bip" = "172.17.0.1/16";
          };
        };

        # ── OOM daemon ─────────────────────────────────────────────────────────
        systemd.oomd.enable = true;
        environment.etc."systemd/oomd.conf.d/10-omarchy.conf".text = ''
          [OOM]
          DefaultMemoryPressureDurationSec=20s
          DefaultMemoryPressureLimit=50%
        '';

        # ── Secrets / polkit ───────────────────────────────────────────────────
        services.gnome.gnome-keyring.enable = true;
        security.pam.services.login.enableGnomeKeyring = true;
        security.polkit.enable = true;

        # ── XDG portals ────────────────────────────────────────────────────────
        xdg.portal = {
          enable = true;
          extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
          config.common.default = "*";
        };

        # ── GPU / graphics ─────────────────────────────────────────────────────
        hardware.graphics.enable = true;

        # ── Session environment ────────────────────────────────────────────────
        environment.sessionVariables = {
          OMARCHY_PATH = "${omarchy}";
          # Cinque shims prepended so they shadow Arch-only commands.
          PATH = [ "${omarchy}/cinque-shims" ];
        };

        # ── Kernel modules for networking tuning ───────────────────────────────
        boot.kernelModules = [ "tcp_bbr" ];

        # ── sysctl tuning ──────────────────────────────────────────────────────
        boot.kernel.sysctl = {
          # Raise inotify ceiling for dev tools (VS Code, webpack, …)
          "fs.inotify.max_user_watches" = 524288;
          # Probe MTU; helps SSH on flaky/VPN links
          "net.ipv4.tcp_mtu_probing" = 1;
          # BBR + fq for lower bufferbloat on fast links
          "net.core.default_qdisc" = "fq";
          "net.ipv4.tcp_congestion_control" = "bbr";
          # zram-friendly VM pressure tuning
          "vm.swappiness" = 150;
          "vm.vfs_cache_pressure" = 50;
          "vm.page-cluster" = 0;
          "vm.watermark_boost_factor" = 0;
          "vm.watermark_scale_factor" = 125;
          "vm.dirty_background_bytes" = 67108864;
          "vm.dirty_bytes" = 268435456;
          "vm.dirty_writeback_centisecs" = 1500;
        };

        # ── logind ─────────────────────────────────────────────────────────────
        services.logind.settings.Login = {
          HandlePowerKey = "ignore";
          InhibitDelayMaxSec = 15;
        };

        # ── systemd timeouts and file-descriptor limits ─────────────────────────
        systemd.settings.Manager = {
          DefaultTimeoutStopSec = "5s";
          DefaultLimitNOFILE = "65536:524288";
        };
        systemd.user.settings.Manager = {
          DefaultLimitNOFILE = "65536:524288";
        };

        # ── USB autosuspend off (Intel BE200/BE211 drops link when it naps) ────
        boot.extraModprobeConfig = "options usbcore autosuspend=-1";

        # ── Disable zswap (double-compresses pages in front of zram) ───────────
        boot.kernelParams = [ "zswap.enabled=0" ];

        # ── omarchy-nix-daemon service ─────────────────────────────────────────
        systemd.services.omarchy-nix-daemon = {
          description = "Omarchy Nix configuration daemon";
          wantedBy = [ "multi-user.target" ];
          after = [ "network.target" ];
          serviceConfig = {
            ExecStart = "${daemon}/bin/omarchy-nix-daemon";
            Restart = "on-failure";
            RestartSec = "2s";
            RuntimeDirectory = "omarchy";
            StateDirectory = "omarchy";
            UMask = "0117";
            User = "omarchy-daemon";
            Group = "omarchy";
          };
        };

        users.users.omarchy-daemon = {
          isSystemUser = true;
          group = "omarchy";
        };
        users.groups.omarchy = { };

        users.users.${cfg.user}.extraGroups = [
          "omarchy"
          "video"
          "audio"
          "networkmanager"
          "wheel"
          "docker"
        ];

        # ── Package set ────────────────────────────────────────────────────────
        environment.systemPackages =
          with pkgs;
          [
            omarchy
            daemon

            # Session manager (launches Hyprland under a proper systemd user session)
            uwsm

            # Hyprland ecosystem
            hyprpaper
            hypridle
            hyprlock
            hyprsunset
            hyprpicker

            # Core Wayland desktop tools
            wl-clipboard
            grim
            slurp
            wtype
            xdg-terminal-exec
            gnome-keyring
            libsecret
            polkit_gnome
            udiskie

            # Terminal / shell
            foot
            fzf
            ripgrep
            bat
            eza
            starship
            zoxide
            fd
            gum
            btop
            tmux
            lazygit
            lazydocker
            less
            inotify-tools
            fastfetch
            imagemagick
            qrencode
            zbar
            yt-dlp
            tldr
            tree-sitter

            # Runtime invariants called directly by omarchy-* scripts
            socat
            jq
            git
            curl
            wget

            # File management
            nautilus
            gvfs
            gnome-disk-utility
            sushi
            dosfstools
            exfatprogs
            unzip

            # Audio utilities
            pamixer
            alsa-utils

            # Media
            mpv
            imv
            obs-studio
            ffmpegthumbnailer

            # Hardware
            brightnessctl
            ddcutil
            bolt
            inxi
            wireless-regdb

            # Networking
            whois

            # Browser
            chromium

            # Productivity
            evince
            libreoffice
            xournalpp
            pinta
            localsend

            # Dev
            mise
            ruby
            lua5_1

            # Fonts
            nerd-fonts.jetbrains-mono
            noto-fonts
            noto-fonts-cjk-sans
            noto-fonts-color-emoji
            font-awesome

            # Theme / icons
            gnome-themes-extra
            yaru-theme

            # Qt integration
            qt6.qtimageformats
          ]
          ++ lib.optionals (builtins.hasAttr "quickshell" pkgs) [ pkgs.quickshell ];

        # ── Copy default configs to the user's home on first boot ──────────────
        # Copies $OMARCHY_PATH/config/* → ~/.config/* if not already present,
        # so Hyprland and other tools find their configs without manual setup.
        system.activationScripts.omarchyUserConfig = lib.stringAfter [ "users" ] ''
          src="${omarchy}/config"
          userhome="/home/${cfg.user}"
          if [[ -d "$userhome" ]]; then
            while IFS= read -r -d "" f; do
              rel="''${f#$src/}"
              target="$userhome/.config/$rel"
              if [[ ! -e "$target" ]]; then
                mkdir -p "$(dirname "$target")"
                install -m 644 -o "${cfg.user}" "$f" "$target"
              fi
            done < <(find "$src" -type f -print0)
          fi
        '';
      };
    };
}
