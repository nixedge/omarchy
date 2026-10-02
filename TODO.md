# Omarchy Nix Port — Remaining Work

## Source-only bash libraries (no Rust dispatch needed)

These files are sourced by other scripts and don't need symlinks in `perSystem/packages.nix`:

- `omarchy-security-functions` — sourced library
- `omarchy-shell-config` — sourced library
- `omarchy-branding-about-animation` — sourced library

## Router cleanup done

- `bin/omarchy` (bash router) deleted — Rust `omarchy` binary is now the entrypoint
- `completions/` (bash-router completions) deleted — clap completions used instead
- `perSystem/packages.nix` updated to remove bash-router completions install block

## Commands ported in this session

All previously unported commands now have Rust CLI dispatch + packages.nix symlinks:

- `omarchy-agent-account-add/home/list/mode/remove/rename/state/use` — exec_script delegates
- `omarchy-agent-usage-grok` — exec_script delegate
- `omarchy-hw-nvidia-display` — Rust sysfs impl (boot_vga scan)
- `omarchy-hw-vm` — Rust impl (systemd-detect-virt)
- `omarchy-cmd-browser-handoff` — exec_script delegate
- `omarchy-cmd-default-browser` — exec_script delegate
- `omarchy-provision-first-run` — exec_script delegate
- `omarchy-provision-user` — exec_script delegate
- `omarchy-remove-service-ssh-agent` — exec_script delegate
- `omarchy-setup-security-ssh-agent` — exec_script delegate
- `omarchy-theme-set-herdr-machines` — exec_theme delegate
- `omarchy-theme-set-hunk` — exec_theme delegate
- `omarchy-toggle-animations` — exec_script delegate
- `omarchy-toggle-theme-sync` — exec_script delegate
- `omarchy-update` — exec_script delegate (complex bash; Rust is the argv0 entrypoint)
