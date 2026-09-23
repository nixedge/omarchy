{ config, lib, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services."hybrid-gpu";
in
{
  options.programs.omarchy.services."hybrid-gpu" = {
    enable = lib.mkEnableOption "NVIDIA hybrid GPU (offload mode)";

    intelBusId = lib.mkOption {
      type = lib.types.str;
      default = "PCI:0:2:0";
      description = "PCI bus ID of the Intel integrated GPU (from lspci).";
      example = "PCI:0:2:0";
    };

    nvidiaBusId = lib.mkOption {
      type = lib.types.str;
      default = "PCI:1:0:0";
      description = "PCI bus ID of the NVIDIA discrete GPU (from lspci).";
      example = "PCI:1:0:0";
    };
  };

  config = lib.mkIf svc.enable {
    hardware.nvidia = {
      modesetting.enable = true;
      prime = {
        offload = {
          enable = true;
          enableOffloadCmd = true;
        };
        intelBusId = svc.intelBusId;
        nvidiaBusId = svc.nvidiaBusId;
      };
    };

    services.xserver.videoDrivers = [ "nvidia" ];
  };
}
