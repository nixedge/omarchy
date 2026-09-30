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
          # test suite dependencies
          pkgs.nodejs
          pkgs.lua
          pkgs.imagemagick
          pkgs.util-linux
          pkgs.glib
          pkgs.mise
          pkgs.plocate
          pkgs.python3
          pkgs.procps
          pkgs.perl
          pkgs.ffmpeg
          pkgs.libxkbcommon
          pkgs.desktop-file-utils
          pkgs.diffutils
          pkgs.gawk
          pkgs.tzdata
          pkgs.shadow
        ];
        # Make the omarchy commands available in the dev shell
        OMARCHY_PATH = config.packages.omarchy;
        shellHook = ''
          export PATH="$OMARCHY_PATH/bin:$PATH"
        '';
      };
    };
}
