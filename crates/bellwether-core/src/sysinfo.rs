use crate::model::InstallMethod;

fn exists(bin: &str) -> bool {
    which::which(bin).is_ok()
}

/// What the current machine has available. Detected once and reused for
/// every install decision in a run.
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub has_apt: bool,
    pub has_pacman: bool,
    pub has_dnf: bool,
    pub has_flatpak: bool,
    pub aur_helper: Option<&'static str>, // "yay" or "paru"
    pub is_root: bool,
}

impl SystemInfo {
    pub fn detect() -> Self {
        let aur_helper = if exists("yay") {
            Some("yay")
        } else if exists("paru") {
            Some("paru")
        } else {
            None
        };

        SystemInfo {
            has_apt: exists("apt-get"),
            has_pacman: exists("pacman"),
            has_dnf: exists("dnf"),
            has_flatpak: exists("flatpak"),
            aur_helper,
            is_root: unsafe { libc_geteuid() == 0 },
        }
    }

    pub fn supports(&self, method: InstallMethod) -> bool {
        match method {
            InstallMethod::Native => self.has_apt || self.has_pacman || self.has_dnf,
            InstallMethod::Aur => self.has_pacman && self.aur_helper.is_some(),
            InstallMethod::Flatpak => self.has_flatpak,
            InstallMethod::Direct => true,
            InstallMethod::Script => true,
        }
    }

    pub fn distro_summary(&self) -> String {
        let mut bits = Vec::new();
        if self.has_apt {
            bits.push("apt");
        }
        if self.has_pacman {
            bits.push("pacman");
        }
        if self.has_dnf {
            bits.push("dnf");
        }
        if self.has_flatpak {
            bits.push("flatpak");
        }
        if bits.is_empty() {
            "unknown package manager".to_string()
        } else {
            bits.join(" + ")
        }
    }
}

// Tiny local shim so we don't pull in the full `libc` crate just for geteuid.
extern "C" {
    #[link_name = "geteuid"]
    fn libc_geteuid_raw() -> u32;
}
unsafe fn libc_geteuid() -> u32 {
    libc_geteuid_raw()
}
