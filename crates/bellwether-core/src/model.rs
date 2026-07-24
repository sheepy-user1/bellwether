/// Category groups apps in listings and TUI panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Monitoring,
    Power,
    Creative,
    Gaming,
    Browser,
    /// Apps you build yourself, hosted on your own GitHub Releases.
    Custom,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::Monitoring => "Monitoring",
            Category::Power => "Power Management",
            Category::Creative => "Creative / 3D Printing",
            Category::Gaming => "Gaming",
            Category::Browser => "Browsers",
            Category::Custom => "My Apps",
        }
    }
}

/// The concrete method used to fetch a package, in rough preference order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMethod {
    Native, // apt / pacman / dnf
    Aur,    // Arch User Repository via yay/paru
    Flatpak,
    Direct, // .deb / AppImage / tarball download
}

impl InstallMethod {
    pub fn label(&self) -> &'static str {
        match self {
            InstallMethod::Native => "native package manager",
            InstallMethod::Aur => "AUR",
            InstallMethod::Flatpak => "Flatpak",
            InstallMethod::Direct => "direct download",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectKind {
    Deb,
    AppImage,
    TarGz,
}

#[derive(Debug, Clone, Copy)]
pub struct DirectInstall {
    pub url: &'static str,
    pub kind: DirectKind,
    /// Final name to place under ~/.local/bin or ~/Applications for AppImages.
    pub install_name: &'static str,
}

/// Per-package-manager package names / identifiers for one app.
#[derive(Debug, Clone, Copy, Default)]
pub struct InstallSpec {
    pub apt: Option<&'static str>,
    pub pacman: Option<&'static str>,
    pub dnf: Option<&'static str>,
    pub aur: Option<&'static str>,
    pub flatpak: Option<&'static str>,
    pub direct: Option<DirectInstall>,
    /// Order in which to try methods. First available + present-on-system wins.
    pub preference: &'static [InstallMethod],
}

/// A step applied after the package itself is installed, to get it into a
/// sane, "just works" configuration instead of defaults.
#[derive(Debug, Clone, Copy)]
pub enum PostInstallStep {
    /// Write a config file for the *invoking* (non-root) user, creating
    /// parent directories as needed. Will not overwrite an existing file
    /// unless `force` is true.
    WriteUserFile {
        rel_path: &'static str, // relative to $HOME
        content: &'static str,
        force: bool,
    },
    /// Run a shell command as root (via the same privilege the installer runs under).
    RootShell(&'static str),
    /// Run a shell command as the normal invoking user.
    UserShell(&'static str),
    /// Enable (and optionally start) a systemd unit.
    EnableService { unit: &'static str, start_now: bool },
    /// Informational note shown to the user, no action taken.
    Note(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct AppDef {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub description: &'static str,
    pub install: InstallSpec,
    pub post_install: &'static [PostInstallStep],
    /// Binary name to look for on PATH when checking whether this is
    /// already installed. Defaults to `id` when `None` — set this
    /// explicitly if the package name and the binary name differ
    /// (e.g. package "linux-tools-generic" but binary "cpupower").
    pub bin_name: Option<&'static str>,
}

impl AppDef {
    pub fn bin_name(&self) -> &'static str {
        self.bin_name.unwrap_or(self.id)
    }
}
