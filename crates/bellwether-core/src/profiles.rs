use crate::catalog;
use crate::model::AppDef;

/// A named bundle of catalog app ids, so `bellwether install --profile
/// server` installs a whole curated set in one shot instead of listing
/// every id by hand every time.
pub struct Profile {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub app_ids: &'static [&'static str],
}

pub const PROFILES: &[Profile] = &[
    Profile {
        id: "home",
        name: "Home",
        description: "Everyday desktop use: monitoring, power tuning, your 3D printer, browser, games, and a friendlier shell.",
        app_ids: &[
            "btop",
            "fastfetch",
            "powertop",
            "cpupower",
            "auto-cpufreq",
            "bambustudio",
            "steam",
            "zen-browser",
            "fish",
            "starship",
        ],
    },
    Profile {
        id: "advanced",
        name: "Advanced",
        description: "Home, plus a fuller terminal toolkit for hands-on tinkering.",
        app_ids: &[
            "btop",
            "fastfetch",
            "powertop",
            "cpupower",
            "auto-cpufreq",
            "bambustudio",
            "steam",
            "zen-browser",
            "fish",
            "starship",
            "tmux",
            "neovim",
            "ripgrep",
            "fzf",
            "bat",
        ],
    },
    Profile {
        id: "server",
        name: "Server",
        description:
            "Headless-box essentials: containers, firewall, intrusion prevention, monitoring, and a solid terminal toolkit.",
        app_ids: &[
            "btop",
            "docker",
            "docker-compose",
            "ufw",
            "fail2ban",
            "tmux",
            "neovim",
            "ripgrep",
            "fzf",
        ],
    },
];

pub fn find_profile(id: &str) -> Option<&'static Profile> {
    PROFILES.iter().find(|p| p.id == id)
}

/// Resolves a profile's app ids against the live catalog (community + your
/// own apps). Ids that don't (or no longer) exist in the catalog are
/// silently skipped rather than erroring — keeps profiles resilient to
/// catalog edits.
pub fn profile_apps(profile: &Profile) -> Vec<&'static AppDef> {
    profile
        .app_ids
        .iter()
        .filter_map(|id| catalog::find(id))
        .collect()
}
