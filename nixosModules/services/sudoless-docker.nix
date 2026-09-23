{ config, lib, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services."sudoless-docker";
in
{
  options.programs.omarchy.services."sudoless-docker" = {
    enable = lib.mkEnableOption "sudoless Docker (adds user to docker group — root-equivalent!)";
  };

  # Docker itself is already enabled in the core omarchy module.
  config = lib.mkIf svc.enable {
    users.users.${cfg.user}.extraGroups = [ "docker" ];
  };
}
