{ inputs, ... }:
let
  nixpkgs = inputs.nixpkgs;

  baseModules = [
    inputs.self.nixosModules.omarchy
    {
      programs.omarchy.enable = true;
      networking.hostName = "omarchy-cinque";
    }
    {
      users.users.omarchy = {
        isNormalUser = true;
        initialPassword = "omarchy";
        # extraGroups merged with those added by the omarchy module
        extraGroups = [
          "omarchy"
          "wheel"
          "video"
          "audio"
          "networkmanager"
        ];
      };
    }
  ];

  mkSystem =
    extraModules:
    nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      specialArgs = { inherit inputs; };
      modules = baseModules ++ extraModules;
    };
in
{
  flake.nixosConfigurations = {
    # Production install — hostname omarchy-cinque, no VM or ISO extras.
    # Used by omarchy-nix-daemon when calling nixos-rebuild switch.
    "omarchy-cinque" = mkSystem [
      {
        system.stateVersion = "25.11";
      }
    ];

    iso = mkSystem [
      {
        imports = [
          "${nixpkgs}/nixos/modules/installer/cd-dvd/installation-cd-graphical-base.nix"
        ];
        isoImage.isoBaseName = nixpkgs.lib.mkForce "omarchy-cinque";
        isoImage.squashfsCompression = "zstd -Xcompression-level 6";
        isoImage.appendToMenuLabel = " Omarchy Cinque";
        services.displayManager.autoLogin = {
          enable = true;
          user = "omarchy";
        };
      }
    ];

    vm = mkSystem [
      (
        { modulesPath, ... }:
        {
          imports = [ "${modulesPath}/virtualisation/qemu-vm.nix" ];
          system.stateVersion = "25.11";
          virtualisation.memorySize = 4096;
          virtualisation.diskSize = 8192;
        }
      )
    ];
  };
}
