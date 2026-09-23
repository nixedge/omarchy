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

      rustSrc = pkgs.lib.fileset.toSource {
        root = ../rust;
        fileset = pkgs.lib.fileset.unions [
          ../rust/Cargo.toml
          ../rust/Cargo.lock
          ../rust/lib
          ../rust/daemon
          ../rust/cli
        ];
      };

      common = {
        inherit craneLib muslToolchain rustSrc;
      };
    in
    {
      _module.args.common = common;
    };
}
