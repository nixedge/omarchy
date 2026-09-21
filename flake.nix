{
  description = "Omarchy Cinque — Nix-native desktop environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    hyprland.url = "github:hyprwm/Hyprland";
    hyprland.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      flake-parts,
      nixpkgs,
      ...
    }@inputs:
    let
      inherit ((import ./flake/lib/recursive-imports.nix { inherit inputs; }).flake.lib)
        recursiveImports
        ;
    in
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports =
        recursiveImports [
          ./flake
          ./perSystem
        ]
        ++ [ inputs.treefmt-nix.flakeModule ];

      systems = [ "x86_64-linux" ];

      flake = {
        defaultPackage = builtins.mapAttrs (_: a: a.default) self.outputs.packages;
      };
    };

  nixConfig = {
    extra-substituters = [ "https://hyprland.cachix.org" ];
    extra-trusted-public-keys = [
      "hyprland.cachix.org-1:a7pgxzMz7+chwVL3/pzj6jIITemDosxrE9/Kb+PfYvE="
    ];
  };
}
