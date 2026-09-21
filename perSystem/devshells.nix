{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      common,
      config,
      ...
    }:
    {
      devShells.default = pkgs.mkShell {
        packages = [
          common.muslToolchain
          pkgs.rust-analyzer
          pkgs.shellcheck
          pkgs.nix
          pkgs.jq
          pkgs.socat
          pkgs.nixfmt
        ];
        # Make the omarchy commands available in the dev shell
        OMARCHY_PATH = config.packages.omarchy;
        shellHook = ''
          export PATH="$OMARCHY_PATH/bin:$PATH"
        '';
      };
    };
}
