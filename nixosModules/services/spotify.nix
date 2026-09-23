{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.spotify;
in
{
  options.programs.omarchy.services.spotify = {
    enable = lib.mkEnableOption "Spotify music player";
  };

  config = lib.mkIf svc.enable {
    environment.systemPackages = [ pkgs.spotify ];
  };
}
