{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.nordvpn;
in
{
  options.programs.omarchy.services.nordvpn = {
    enable = lib.mkEnableOption "NordVPN";
  };

  config = lib.mkIf svc.enable {
    environment.systemPackages = [ pkgs.nordvpn ];

    # Install the nordvpnd systemd unit shipped by the package and start it.
    systemd.packages = [ pkgs.nordvpn ];
    systemd.services.nordvpnd.wantedBy = [ "multi-user.target" ];

    users.groups.nordvpn = { };
    users.users.${cfg.user}.extraGroups = [ "nordvpn" ];
  };
}
