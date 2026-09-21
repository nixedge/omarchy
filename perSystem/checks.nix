{ inputs, ... }:
{
  perSystem =
    {
      system,
      pkgs,
      lib,
      config,
      ...
    }:
    lib.mkIf (system == "x86_64-linux") {
      checks = {
        shellcheck = pkgs.runCommand "shellcheck" { nativeBuildInputs = [ pkgs.shellcheck ]; } ''
          shellcheck ${config.packages.omarchy}/bin/omarchy-*
          touch $out
        '';

        vm-smoke = import ../tests/vm-smoke.nix { inherit inputs pkgs lib; };
      };
    };
}
