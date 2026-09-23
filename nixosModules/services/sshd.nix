{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.sshd;
in
{
  options.programs.omarchy.services.sshd = {
    enable = lib.mkEnableOption "OpenSSH server with key-only authentication";
  };

  config = lib.mkIf svc.enable {
    services.openssh = {
      enable = true;
      openFirewall = true;
      settings = {
        PasswordAuthentication = false;
        KbdInteractiveAuthentication = false;
      };
    };
  };
}
