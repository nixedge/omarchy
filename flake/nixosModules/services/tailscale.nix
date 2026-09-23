{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.tailscale;
in
{
  options.programs.omarchy.services.tailscale = {
    enable = lib.mkEnableOption "Tailscale mesh VPN";
  };

  config = lib.mkIf svc.enable {
    services.tailscale.enable = true;

    # Taildrop file receiver — delivers incoming files to ~/Downloads.
    systemd.user.services.omarchy-tailscale-receive = {
      description = "Receive Taildrop files into Downloads";
      wantedBy = [ "default.target" ];
      after = [ "tailscaled.service" ];
      serviceConfig = {
        ExecStart = "${pkgs.tailscale}/bin/tailscale file get --loop --verbose /home/${cfg.user}/Downloads";
        Restart = "on-failure";
        RestartSec = "5s";
      };
    };
  };
}
