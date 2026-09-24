{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.hermes;
in
{
  options.programs.omarchy.services.hermes = {
    enable = lib.mkEnableOption "Hermes AI agent standalone CLI";
  };

  config = lib.mkIf svc.enable {
    environment.systemPackages = [ pkgs.hermes-agent ];
  };
}
