{ inputs, ... }:
{
  perSystem =
    { pkgs, system, ... }:
    let
      fenixPkgs = inputs.fenix.packages.${system};

      muslToolchain = fenixPkgs.combine [
        fenixPkgs.stable.cargo
        fenixPkgs.stable.rustc
        fenixPkgs.targets.x86_64-unknown-linux-musl.stable.rust-std
      ];

      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain muslToolchain;

      daemonSrc = pkgs.lib.fileset.toSource {
        root = ../daemon;
        fileset = pkgs.lib.fileset.unions [
          ../daemon/Cargo.toml
          ../daemon/Cargo.lock
          ../daemon/src
        ];
      };

      common = {
        inherit craneLib muslToolchain daemonSrc;
      };
    in
    {
      _module.args.common = common;
    };
}
