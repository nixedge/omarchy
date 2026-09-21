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
              # --severity=error: catch real errors but not pre-existing style warnings
              # in the upstream Arch-era scripts.
              for f in ${config.packages.omarchy}/bin/omarchy-*; do
                read -r shebang < "$f"
                # SC1087: false positive on ImageMagick's image.png[0] frame syntax
                [[ $shebang == "#!/"*"bash"* ]] && shellcheck --severity=error --exclude=SC1087 "$f"
              done
              touch $out
            '';

        vm-smoke = import ../tests/vm-smoke.nix { inherit inputs pkgs lib; };
      };
    };
}
