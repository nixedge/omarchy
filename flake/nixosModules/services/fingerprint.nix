{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.fingerprint;
in
{
  options.programs.omarchy.services.fingerprint = {
    enable = lib.mkEnableOption "fingerprint authentication for sudo, polkit, and lock screen";
  };

  config = lib.mkIf svc.enable {
    services.fprintd.enable = true;

    security.pam.services.sudo.fprintAuth = true;
    security.pam.services.login.fprintAuth = true;
    security.pam.services.polkit-1.fprintAuth = true;

    # Custom PAM service for the omarchy lock screen fingerprint path.
    security.pam.services.omarchy-lock-fingerprint = {
      text = ''
        #%PAM-1.0
        auth       required  pam_fprintd.so
        account    include   system-local-login
      '';
    };
  };
}
