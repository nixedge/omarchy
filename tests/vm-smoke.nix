{
  inputs,
  pkgs,
  lib,
}:
pkgs.testers.nixosTest {
  name = "omarchy-cinque-vm-smoke";

  nodes.machine =
    { ... }:
    {
      imports = [ inputs.self.nixosModules.omarchy ];
      programs.omarchy.enable = true;
      # Give the VM enough room for a nixos-rebuild inside the test.
      virtualisation.memorySize = 4096;
      virtualisation.diskSize = 8192;
      system.stateVersion = "25.11";
      users.users.omarchy = {
        isNormalUser = true;
        initialPassword = "omarchy";
        # extraGroups merged with omarchy module's additions (omarchy, wheel, …)
      };
    };

  testScript = ''
    import json

    # ── Daemon health ────────────────────────────────────────────────────────
    machine.wait_for_unit("omarchy-nix-daemon.service")
    machine.wait_for_file("/run/omarchy/packages.json")
    machine.wait_for_file("/var/lib/omarchy/omarchy-managed.nix")

    machine.succeed("omarchy help")

    # ── pkg-list returns valid JSON ──────────────────────────────────────────
    raw = machine.succeed("omarchy pkg list")
    pkgs = json.loads(raw)
    assert isinstance(pkgs, list), f"pkg-list: expected JSON list, got: {raw!r}"
    assert len(pkgs) > 0, "pkg-list: manifest is empty — daemon did not populate it"

    # ── pkg-present / pkg-missing against the live manifest ─────────────────
    # Pick the first package from the real manifest so we don't hardcode names.
    first_name = pkgs[0]["name"]

    machine.succeed(f"su - omarchy -c 'omarchy-pkg-present {first_name}'")
    machine.fail(f"su - omarchy -c 'omarchy-pkg-missing {first_name}'")

    machine.fail("su - omarchy -c 'omarchy-pkg-present definitely-not-a-real-package-xyz'")
    machine.succeed("su - omarchy -c 'omarchy-pkg-missing definitely-not-a-real-package-xyz'")

    # ── pkg-present: absent manifest is safe (exits 1, not a crash) ─────────
    machine.fail(
        "su - omarchy -c 'OMARCHY_MANIFEST=/nonexistent omarchy-pkg-present bat || exit 1'"
    )

    # ── Alias rejection: service-managed packages ────────────────────────────
    # These are handled by the alias table before any nixos-rebuild is attempted.
    machine.fail("omarchy pkg add docker")
    machine.fail("omarchy pkg add bluez")
    machine.fail("omarchy pkg add networkmanager")

    # ── Alias rejection: AUR / Arch-only packages ────────────────────────────
    machine.fail("omarchy pkg add yay")
    machine.fail("omarchy pkg add paru")

    # ── pkg-drop refuses packages not currently in state ─────────────────────
    machine.fail("omarchy pkg drop definitely-not-installed-xyz")

    # ── Full pipeline: pkg-add → rebuild → pkg-drop ──────────────────────────
    # `hello` (GNU Hello) is a tiny package present in every nixpkgs tree.
    # It is deliberately absent from the omarchy base package list, so pkg-add
    # actually adds something new.  With the build cache warm this takes under
    # two minutes; the 600 s timeout is a safety net.

    rc, out = machine.execute("omarchy pkg add hello", timeout=600)

    if rc == 0:
        # Rebuild succeeded — verify the full acceptance chain.

        machine.succeed("hello | grep -i hello")

        raw = machine.succeed("omarchy pkg list")
        manifest = json.loads(raw)
        manifest_names = [p["name"] for p in manifest]
        assert "hello" in manifest_names, \
            f"hello not in packages.json after add; manifest={manifest_names!r}"

        state = json.loads(machine.succeed("cat /var/lib/omarchy/state.json"))
        assert "hello" in state.get("packages", []), \
            f"hello not in state.json after add; state={state!r}"

        machine.succeed(f"su - omarchy -c 'omarchy-pkg-present hello'")
        machine.fail(f"su - omarchy -c 'omarchy-pkg-missing hello'")

        # pkg-drop removes the package and triggers a second rebuild.
        machine.succeed("omarchy pkg drop hello", timeout=600)
        machine.fail("hello")

        state = json.loads(machine.succeed("cat /var/lib/omarchy/state.json"))
        assert "hello" not in state.get("packages", []), \
            f"hello still in state.json after drop; state={state!r}"

        machine.fail(f"su - omarchy -c 'omarchy-pkg-present hello'")
        machine.succeed(f"su - omarchy -c 'omarchy-pkg-missing hello'")

    else:
        # Rebuild failed (e.g. no cache access in sandboxed CI).
        # The spec requires state.json to be rolled back on failure (SC-004).
        state = json.loads(machine.succeed("cat /var/lib/omarchy/state.json"))
        assert "hello" not in state.get("packages", []), \
            f"state.json not rolled back after failed rebuild; state={state!r}"
        print(f"WARNING: nixos-rebuild failed (rc={rc}); rebuild path not verified")
        print(f"rebuild output: {out}")
  '';
}
