{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.fido2;
in
{
  options.programs.omarchy.services.fido2 = {
    enable = lib.mkEnableOption "FIDO2 hardware key authentication for sudo and polkit";
  };

  config = lib.mkIf svc.enable {
    security.pam.u2f = {
      enable = true;
      control = "sufficient";
      cue = true;
      authFile = "/etc/fido2/fido2";
    };

    security.pam.services.sudo.u2fAuth = true;
    security.pam.services.login.u2fAuth = true;
    security.pam.services.polkit-1.u2fAuth = true;

    environment.systemPackages = with pkgs; [ libfido2 ];
  };
}
