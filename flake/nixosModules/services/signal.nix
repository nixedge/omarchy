{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.signal;
in
{
  options.programs.omarchy.services.signal = {
    enable = lib.mkEnableOption "Signal messenger";
  };

  config = lib.mkIf svc.enable {
    environment.systemPackages = [ pkgs.signal-desktop ];
  };
}
