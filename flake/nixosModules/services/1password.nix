{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services."1password";
in
{
  options.programs.omarchy.services."1password" = {
    enable = lib.mkEnableOption "1Password password manager";
  };

  config = lib.mkIf svc.enable {
    programs._1password.enable = true;
    programs._1password-gui = {
      enable = true;
      polkitPolicyOwners = [ cfg.user ];
    };

    # Chromium extension — auto-installs via update URL on next browser launch.
    environment.etc."chromium/extensions/aeblfdkhhhdcdjpifhhbdiojplfjncoa.json".text =
      ''{ "external_update_url": "https://clients2.google.com/service/update2/crx" }'';
  };
}
