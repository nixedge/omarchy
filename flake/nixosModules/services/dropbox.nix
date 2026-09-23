{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.dropbox;
in
{
  options.programs.omarchy.services.dropbox = {
    enable = lib.mkEnableOption "Dropbox cloud storage";
  };

  config = lib.mkIf svc.enable {
    environment.systemPackages = with pkgs; [ dropbox dropbox-cli ];

    # Run Dropbox as a user service so it starts with the desktop session.
    systemd.user.services.dropbox = {
      description = "Dropbox";
      wantedBy = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig = {
        ExecStart = "${pkgs.dropbox}/bin/dropbox";
        Restart = "on-failure";
        RestartSec = "5s";
      };
    };
  };
}
