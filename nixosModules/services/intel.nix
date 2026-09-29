{ config, lib, pkgs, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.intel;
in
{
  options.programs.omarchy.services.intel = {
    enable = lib.mkEnableOption "Intel CPU/GPU optimizations";

    mediaDriver = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable Intel media driver for hardware video acceleration (Broadwell and newer).";
    };

    thermald = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable thermald thermal management daemon (Sandy Bridge and newer with battery).";
    };

    lowPowerMode = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Enable Intel Low Power Mode Daemon (lpmd) for efficiency core management.";
    };
  };

  config = lib.mkIf svc.enable {
    hardware.graphics = {
      enable = true;
      extraPackages = lib.optionals svc.mediaDriver [
        pkgs.intel-media-driver
        pkgs.intel-compute-runtime
        pkgs.vpl-gpu-rt
      ];
    };

    services.thermald.enable = lib.mkIf svc.thermald true;

    # lpmd is not yet packaged in nixpkgs; placeholder for when it lands
    # services.intel-lpmd.enable = lib.mkIf svc.lowPowerMode true;

    environment.sessionVariables = lib.mkIf svc.mediaDriver {
      LIBVA_DRIVER_NAME = "iHD";
    };
  };
}
