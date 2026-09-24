{ ... }: {
  imports = [
    ./1password.nix
    ./dropbox.nix
    ./fido2.nix
    ./fingerprint.nix
    ./hermes.nix
    ./hibernation.nix
    ./hybrid-gpu.nix
    ./nordvpn.nix
    ./signal.nix
    ./spotify.nix
    ./sshd.nix
    ./sudoless-docker.nix
    ./sunshine.nix
    ./tailscale.nix
  ];
}
