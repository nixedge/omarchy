/// Resolve an Arch Linux package name to the corresponding nixpkgs attribute.
///
/// Returns the nixpkgs attribute name to use in `with pkgs; [ ... ]`.  If the
/// name is not in the alias table, the original name is returned unchanged
/// (many packages have identical names in both ecosystems).
///
/// Returns `Err(ResolveError::ServiceManaged)` for packages that are
/// controlled by NixOS module options rather than `environment.systemPackages`,
/// so the caller can print a helpful message instead of attempting an install.
///
/// Returns `Err(ResolveError::Eliminated)` for Arch packages that have no
/// NixOS equivalent and should not be installed.
pub fn resolve(name: &str) -> Result<&str, ResolveError> {
    match name {
        // ── Arch name differs from nixpkgs attribute ───────────────────────
        "dua-cli" => Ok("dua"),
        "gvfs-mtp" | "gvfs-nfs" | "gvfs-smb" => Ok("gvfs"),
        "libvips" => Ok("vips"),
        "libreoffice-fresh" => Ok("libreoffice"),
        "lua51" => Ok("lua5_1"),
        "mariadb-libs" => Ok("mariadb"),
        "mise-bin" => Ok("mise"),
        "mpv-mpris" => Ok("mpvScripts.mpris"),
        "noto-fonts-cjk" => Ok("noto-fonts-cjk-sans"),
        "noto-fonts-emoji" => Ok("noto-fonts-color-emoji"),
        "nvim" => Ok("neovim"),
        "omarchy-nvim" => Ok("neovim"),
        "postgresql-libs" => Ok("libpq"),
        "python-gobject" => Ok("python3Packages.pygobject3"),
        "python-poetry-core" => Ok("python3Packages.poetry-core"),
        "qt6-imageformats" => Ok("qt6.qtimageformats"),
        "tesseract-data-eng" => Ok("tesseract"),
        "tree-sitter-cli" => Ok("tree-sitter"),
        "ttf-ia-writer" => Ok("ia-writer-duospace"),
        "ttf-jetbrains-mono-nerd-basic" => Ok("nerd-fonts.jetbrains-mono"),
        "vi" => Ok("vim"),
        "woff2-font-awesome" => Ok("font-awesome"),
        "xdg-terminal-exec" => Ok("xdg-terminal-exec"),
        "xorg-xwayland" => Ok("xwayland"),
        "yaru-icon-theme" => Ok("yaru-theme"),
        "yt-dlp" => Ok("yt-dlp"),

        // ── Managed by NixOS module options, not systemPackages ────────────
        "bluez" | "bluez-tools" | "bluez-utils" => {
            Err(ResolveError::ServiceManaged("hardware.bluetooth.enable"))
        }
        "cups" | "cups-filters" | "cups-pk-helper" => {
            Err(ResolveError::ServiceManaged("services.printing.enable"))
        }
        "avahi" | "nss-mdns" => {
            Err(ResolveError::ServiceManaged("services.avahi.enable"))
        }
        "docker" | "docker-buildx" | "docker-compose" => {
            Err(ResolveError::ServiceManaged("virtualisation.docker.enable"))
        }
        "networkmanager" => {
            Err(ResolveError::ServiceManaged("networking.networkmanager.enable"))
        }
        "pipewire" | "wireplumber" => {
            Err(ResolveError::ServiceManaged("services.pipewire.enable"))
        }
        "power-profiles-daemon" => {
            Err(ResolveError::ServiceManaged("services.power-profiles-daemon.enable"))
        }
        "sddm" => Err(ResolveError::ServiceManaged("services.displayManager.sddm.enable")),
        "ufw" | "ufw-docker" => {
            Err(ResolveError::ServiceManaged("networking.firewall.enable"))
        }
        "hyprland" | "xdg-desktop-portal-hyprland" => {
            Err(ResolveError::ServiceManaged("programs.hyprland.enable"))
        }
        "plymouth" => Err(ResolveError::ServiceManaged("boot.plymouth.enable")),

        // ── Arch-only / AUR packages with no NixOS equivalent ─────────────
        "yay" | "paru" | "expac" | "fakeroot" | "makepkg" => {
            Err(ResolveError::Eliminated("AUR tooling does not exist on NixOS"))
        }
        "kernel-modules-hook" | "linux-modules-cleanup" => {
            Err(ResolveError::Eliminated("Arch kernel hook, not needed on NixOS"))
        }
        "limine" | "limine-entry-tool" => {
            Err(ResolveError::Eliminated("Limine is the Arch bootloader; NixOS uses systemd-boot"))
        }
        "mkinitcpio" | "mkinitcpio-openswap" => {
            Err(ResolveError::Eliminated("Arch initramfs tool, not needed on NixOS"))
        }
        "pacman-contrib" => {
            Err(ResolveError::Eliminated("pacman helper, not needed on NixOS"))
        }

        // ── Pass through unchanged ─────────────────────────────────────────
        name => Ok(name),
    }
}

#[derive(Debug)]
pub enum ResolveError {
    /// Package is managed by a NixOS module option; `option` is the relevant path.
    ServiceManaged(&'static str),
    /// Package is Arch-specific and has no NixOS equivalent; `reason` explains why.
    Eliminated(&'static str),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServiceManaged(opt) => write!(
                f,
                "this package is managed by the NixOS option `{opt}`;\n\
                 use the Omarchy settings panel or omarchy-setup to toggle it"
            ),
            Self::Eliminated(reason) => write!(f, "not available on NixOS: {reason}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_unchanged() {
        assert_eq!(resolve("bat").unwrap(), "bat");
        assert_eq!(resolve("ripgrep").unwrap(), "ripgrep");
        assert_eq!(resolve("chromium").unwrap(), "chromium");
    }

    #[test]
    fn known_aliases() {
        assert_eq!(resolve("libreoffice-fresh").unwrap(), "libreoffice");
        assert_eq!(resolve("lua51").unwrap(), "lua5_1");
        assert_eq!(resolve("noto-fonts-emoji").unwrap(), "noto-fonts-color-emoji");
        assert_eq!(resolve("dua-cli").unwrap(), "dua");
        assert_eq!(resolve("vi").unwrap(), "vim");
    }

    #[test]
    fn service_managed() {
        assert!(matches!(resolve("docker"), Err(ResolveError::ServiceManaged(_))));
        assert!(matches!(resolve("bluez"), Err(ResolveError::ServiceManaged(_))));
        assert!(matches!(resolve("networkmanager"), Err(ResolveError::ServiceManaged(_))));
    }

    #[test]
    fn eliminated() {
        assert!(matches!(resolve("yay"), Err(ResolveError::Eliminated(_))));
        assert!(matches!(resolve("paru"), Err(ResolveError::Eliminated(_))));
    }
}
