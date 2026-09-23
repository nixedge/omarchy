{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      common,
      system,
      ...
    }:
    let
      muslLinker = "${pkgs.pkgsStatic.stdenv.cc}/bin/${pkgs.pkgsStatic.stdenv.cc.targetPrefix}cc";
      muslArgs = {
        src = common.rustSrc;
        strictDeps = true;
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = muslLinker;
        doCheck = false;
      };
    in
    {
      packages = {
        omarchy-nix-daemon = common.craneLib.buildPackage (muslArgs // {
          pname = "omarchy-nix-daemon";
          version = "0.1.0";
          cargoExtraArgs = "--package omarchy-nix-daemon";
        });

        omarchy-cli = common.craneLib.buildPackage (muslArgs // {
          pname = "omarchy-cli";
          version = "0.1.0";
          cargoExtraArgs = "--package omarchy-cli";
          postInstall = ''
            install -d $out/share/bash-completion/completions \
                       $out/share/zsh/site-functions \
                       $out/share/fish/vendor_completions.d
            $out/bin/omarchy-cli completions bash \
              > $out/share/bash-completion/completions/omarchy-cli
            $out/bin/omarchy-cli completions zsh \
              > $out/share/zsh/site-functions/_omarchy-cli
            $out/bin/omarchy-cli completions fish \
              > $out/share/fish/vendor_completions.d/omarchy-cli.fish

            # Backward-compatible symlinks — argv[0] dispatch routes each name
            # to the right subcommand without wrapper scripts.
            for name in \
              omarchy-pkg-add \
              omarchy-pkg-drop \
              omarchy-pkg-install \
              omarchy-pkg-list \
              omarchy-pkg-missing \
              omarchy-pkg-present \
              omarchy-pkg-resolve \
              omarchy-pkg-search \
              omarchy-pkg-sync; do
              ln -s $out/bin/omarchy-cli $out/bin/$name
            done
          '';
        });

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
            # icon.png is referenced by default/chromium/extensions/copy-url/icon.png
            # via a ../../../../ symlink that resolves to the package root.
            [ -f icon.png ] && cp icon.png $out/

            # omarchy bash-router completions (for `omarchy pkg …` etc.)
            install -Dm644 completions/bash/omarchy \
              $out/share/bash-completion/completions/omarchy
            install -Dm644 completions/zsh/_omarchy \
              $out/share/zsh/site-functions/_omarchy
            install -Dm644 completions/fish/omarchy.fish \
              $out/share/fish/vendor_completions.d/omarchy.fish

            # Wayland session entry so SDDM discovers the omarchy session.
            mkdir -p $out/share/wayland-sessions
            install -m 0644 default/wayland-sessions/omarchy.desktop \
              $out/share/wayland-sessions/omarchy.desktop

            # Cinque-not-implemented shims for eliminated Arch-only commands.
            # These are prepended to PATH via environment.sessionVariables so
            # they shadow the originals without modifying bin/.
            mkdir -p $out/cinque-shims
            for cmd in \
              omarchy-pkg-add-aur \
              omarchy-pkg-remove \
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

          # patchShebangs rewrites #!/bin/bash to the nix-store bash path.
          # bin/omarchy's register_command already skips line 1 when it starts
          # with #!, so the header-grep is unaffected by shebang rewriting.

          # Required by services.displayManager.sessionPackages type check.
          passthru.providedSessions = [ "omarchy" ];
        };

        default = inputs.self.packages.${system}.omarchy;
      };
    };
}
