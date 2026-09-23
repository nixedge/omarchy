{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.sunshine;
in
{
  options.programs.omarchy.services.sunshine = {
    enable = lib.mkEnableOption "Sunshine remote desktop streaming (Moonlight-compatible)";
  };

  config = lib.mkIf svc.enable {
    services.sunshine = {
      enable = true;
      autoStart = true;
      openFirewall = true;
      capSysAdmin = true;
    };
  };
}
