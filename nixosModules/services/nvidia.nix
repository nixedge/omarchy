{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.nvidia;
in
{
  options.programs.omarchy.services.nvidia = {
    enable = lib.mkEnableOption "NVIDIA GPU support";

    open = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Use the NVIDIA open kernel module (RTX 20xx and newer). Set to false for older GPUs.";
    };

    videoAcceleration = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable NVIDIA VDPAU/VAAPI video acceleration via libva-nvidia-driver.";
    };

    powerManagement = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Enable NVIDIA runtime power management (experimental; may cause suspend issues).";
    };
  };

  config = lib.mkIf svc.enable {
    hardware.nvidia = {
      modesetting.enable = true;
      open = svc.open;
      nvidiaSettings = true;
      powerManagement.enable = svc.powerManagement;
    };

    hardware.graphics = {
      enable = true;
      extraPackages = lib.optionals svc.videoAcceleration [ pkgs.libva-nvidia-driver ];
    };

    services.xserver.videoDrivers = [ "nvidia" ];

    # Required for Wayland/Hyprland with NVIDIA
    environment.sessionVariables = {
      LIBVA_DRIVER_NAME = "nvidia";
      __GLX_VENDOR_LIBRARY_NAME = "nvidia";
      NVD_BACKEND = "direct";
    };
  };
}
