{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      common,
      system,
      ...
    }:
    {
      packages = {
        omarchy-nix-daemon = common.craneLib.buildPackage {
          src = common.daemonSrc;
          pname = "omarchy-nix-daemon";
          version = "0.1.0";
          strictDeps = true;
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs.pkgsStatic.stdenv.cc}/bin/${pkgs.pkgsStatic.stdenv.cc.targetPrefix}cc";
          # Tests require a running system; skip in the build sandbox.
          doCheck = false;
        };

        omarchy = pkgs.stdenv.mkDerivation {
          pname = "omarchy";
          version = inputs.self.shortRev or "dev";
          src = inputs.self;

          nativeBuildInputs = [ pkgs.bash ];

          installPhase = ''
            runHook preInstall

            mkdir -p $out
            for d in bin config default shell themes migrations manual; do
              [ -d "$d" ] && cp -r "$d" $out/
            done

            # Cinque-not-implemented shims for eliminated Arch-only commands.
            # These are prepended to PATH via environment.sessionVariables so
            # they shadow the originals without modifying bin/.
            mkdir -p $out/cinque-shims
            for cmd in \
              omarchy-pkg-add-aur \
              omarchy-update-aur-pkgs \
              omarchy-update-keyring \
              omarchy-update-pacman-guard \
              omarchy-update-mise \
              omarchy-dev-pkg-test \
              omarchy-upgrade-to-quattro \
              omarchy-refresh-pacman; do
              install -m 0755 ${../pkgs/omarchy/cinque-not-implemented.sh} $out/cinque-shims/$cmd
            done

            runHook postInstall
          '';

          # Leave #!/bin/bash shebangs alone — NixOS provides /bin/bash and
          # rewriting to store paths would break the header-grep in bin/omarchy
          # which scans for # omarchy:summary= lines in the installed files.
          dontPatchShebangs = true;
          dontFixup = true;
        };

        default = inputs.self.packages.${system}.omarchy;
      };
    };
}
