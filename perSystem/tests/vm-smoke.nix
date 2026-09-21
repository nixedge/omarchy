{ inputs, pkgs, lib }:
pkgs.testers.nixosTest {
  name = "omarchy-cinque-vm-smoke";

  nodes.machine =
    { ... }:
    {
      imports = [ inputs.self.nixosModules.omarchy ];
      programs.omarchy.enable = true;
      virtualisation.memorySize = 2048;
    };

  testScript = ''
    machine.wait_for_unit("omarchy-nix-daemon.service")
    machine.wait_for_file("/run/omarchy/packages.json")
    machine.succeed("omarchy help")
    out = machine.succeed("omarchy pkg list")
    assert len(out.strip()) > 0, f"pkg list output was empty: {out!r}"
  '';
}
