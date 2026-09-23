{ config, lib, ... }:
let
  cfg = config.programs.omarchy;
  svc = cfg.services.hibernation;
in
{
  options.programs.omarchy.services.hibernation = {
    enable = lib.mkEnableOption "system hibernation";

    swapDevice = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Block device path for swap (e.g. /dev/disk/by-uuid/...).";
      example = "/dev/disk/by-uuid/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
    };

    resumeOffset = lib.mkOption {
      type = lib.types.int;
      default = 0;
      description = "Btrfs swapfile physical offset (from btrfs inspect-internal map-swapfile -r).";
    };
  };

  config = lib.mkIf svc.enable {
    boot.resumeDevice = lib.mkIf (svc.swapDevice != "") svc.swapDevice;
    boot.kernelParams = lib.optional (svc.resumeOffset != 0)
      "resume_offset=${toString svc.resumeOffset}";
    swapDevices = lib.optional (svc.swapDevice != "") { device = svc.swapDevice; };
    boot.initrd.systemd.enableHibernation = true;
  };
}
