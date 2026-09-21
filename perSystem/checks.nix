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
        shellcheck =
          pkgs.runCommand "shellcheck"
            { nativeBuildInputs = [ pkgs.shellcheck pkgs.bash ]; }
            ''
              # Only check bash scripts; Python and other-shebang files are skipped.
              for f in ${config.packages.omarchy}/bin/omarchy-*; do
                read -r shebang < "$f"
                [[ $shebang == "#!/"*"bash"* ]] && shellcheck "$f"
              done
              touch $out
            '';

        vm-smoke = import ../tests/vm-smoke.nix { inherit inputs pkgs lib; };
      };
    };
}
