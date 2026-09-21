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
        programs.hyprland = {
          enable = true;
          package = inputs.hyprland.packages.${system}.hyprland;
          portalPackage =
            inputs.hyprland.packages.${system}.xdg-desktop-portal-hyprland;
        };

        # OMARCHY_PATH in the session environment, not hardcoded in the package.
        environment.sessionVariables = {
          OMARCHY_PATH = "${omarchy}";
          # Cinque shims prepended so they shadow Arch-only commands.
          PATH = [ "${omarchy}/cinque-shims" ];
        };

        environment.systemPackages =
          with pkgs;
          [
            omarchy
            daemon
            # Runtime invariants called directly by omarchy-* scripts
            socat
            jq
            git
            curl
            wget
            foot
          ]
          ++ lib.optionals (builtins.hasAttr "quickshell" pkgs) [ pkgs.quickshell ];

        # omarchy-nix-daemon systemd service
        systemd.services.omarchy-nix-daemon = {
          description = "Omarchy Nix configuration daemon";
          wantedBy = [ "multi-user.target" ];
          after = [ "network.target" ];
          serviceConfig = {
            ExecStart = "${daemon}/bin/omarchy-nix-daemon";
            Restart = "on-failure";
            RestartSec = "2s";
            # Creates /run/omarchy (0755) and /var/lib/omarchy (0750)
            RuntimeDirectory = "omarchy";
            StateDirectory = "omarchy";
            # Socket is 0660; group omarchy; autologin user is added below.
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

        # Add the desktop user to the omarchy group so their scripts can reach
        # the daemon socket.
        users.users.${cfg.user}.extraGroups = [
          "omarchy"
          "video"
          "audio"
          "networkmanager"
          "wheel"
        ];
      };
    };
}
