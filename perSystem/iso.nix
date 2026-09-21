{ inputs, ... }:
{
  perSystem =
    {
      system,
      lib,
      config,
      ...
    }:
    lib.mkIf (system == "x86_64-linux") {
      packages.iso = inputs.self.nixosConfigurations.iso.config.system.build.isoImage;
      packages.vm = inputs.self.nixosConfigurations.vm.config.system.build.vm;

      apps.run-vm = {
        type = "app";
        program = lib.getExe config.packages.vm;
        meta.description = "Run the Omarchy Cinque QEMU VM";
      };
    };
}
